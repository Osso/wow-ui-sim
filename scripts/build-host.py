#!/usr/bin/env python3
"""Build one Linux executable on the shared desktop/local Docker build host.

Requires game-engine scripts at
/syncthing/Sync/Projects/world-of-osso/game-engine/scripts.
BUILD_HOST_SCRIPTS overrides that single dependency directory; no path search or
host fallback. The saved default is shared with game-engine. Runtime libraries
live in target/<profile>/<bin>.libs/<content-id>, referenced by the ELF RUNPATH;
old generations remain usable by running processes.
"""

import argparse
import gzip
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

COMMON_SCRIPTS = Path("/syncthing/Sync/Projects/world-of-osso/game-engine/scripts")
SOURCE_DIRS = (
    "src",
    "build",
    "native",
    "iced-dynamic",
    "iced-wgpu-patched",
    "xtask",
    "tests",
)
ROOT_FILES = {"Cargo.toml", "Cargo.lock", "build.rs", ".cargo/config.toml"}
EXCLUDED = {
    ".git",
    "target",
    "cache",
    "__pycache__",
    "SavedVariables",
    "WTF",
    "node_modules",
}
SECRET_SUFFIXES = {".key", ".pem", ".p12", ".pfx"}
LITERAL_INCLUDE = re.compile(r'\binclude(?:_str|_bytes)?!\s*\(\s*"([^"\n]+)"')
MANIFEST_INCLUDE = re.compile(
    r"\binclude(?:_str|_bytes)?!\s*\(\s*concat!\s*\(\s*"
    r'env!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*,\s*"([^"\n]+)"\s*\)\s*\)',
    re.MULTILINE,
)
PATH_ATTRIBUTE = re.compile(r'#\[[^\]]*?\bpath\s*=\s*"([^"\n]+)"', re.MULTILINE)
# Glibc and its loader must come from the caller, not the Debian build image.
SYSTEM_LIBRARIES = {
    "libc.so.6",
    "libm.so.6",
    "libpthread.so.0",
    "libdl.so.2",
    "librt.so.1",
    "libresolv.so.2",
    "libutil.so.1",
    "ld-linux-x86-64.so.2",
}


def load_common():
    directory = Path(os.environ.get("BUILD_HOST_SCRIPTS", COMMON_SCRIPTS)).resolve()
    for name in ("depot-build.py", "build_hosts.py"):
        if not (directory / name).is_file():
            raise FileNotFoundError(
                f"missing shared build dependency: {directory / name}"
            )
    sys.path.insert(0, str(directory))
    spec = importlib.util.spec_from_file_location(
        "engine_build_helper", directory / "depot-build.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git_files(root, paths):
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            *paths,
        ],
        check=True,
        capture_output=True,
    )
    return {Path(os.fsdecode(value)) for value in result.stdout.split(b"\0") if value}


def safe_source(relative):
    return (
        not relative.is_absolute()
        and ".." not in relative.parts
        and not any(part in EXCLUDED for part in relative.parts)
        and not any(part.startswith(".") for part in relative.parts if part != ".cargo")
        and relative.suffix not in SECRET_SUFFIXES
    )


def validate_source(root, relative):
    if not safe_source(relative):
        raise ValueError(f"excluded compile input: {relative}")
    source = root / relative
    if source.is_symlink() or any(
        (root / parent).is_symlink() for parent in relative.parents
    ):
        raise ValueError(f"unsupported source symlink: {source}")
    if not source.is_file():
        raise FileNotFoundError(f"missing compile input: {source}")
    return source


def compile_references(root, relative):
    """Follow literal Rust includes/path attributes, including cfg_attr variants.

    OUT_DIR includes are Cargo-generated, not snapshot inputs. No runtime path
    strings are followed. Data/assets/Interface are admitted only through these
    compile references, never as whole directories.
    """
    text = (root / relative).read_text()
    references = []
    references.extend(
        (root / relative).parent / value for value in LITERAL_INCLUDE.findall(text)
    )
    # Source directories are already selected in full. Inline-module #[path]
    # resolution belongs to rustc; follow only attributes into external data.
    for value in PATH_ATTRIBUTE.findall(text):
        path = Path(os.path.abspath((root / relative).parent / value))
        if (
            not path.is_relative_to(root)
            or path.relative_to(root).parts[0] not in SOURCE_DIRS
        ):
            references.append(path)
    references.extend(
        root / value.lstrip("/") for value in MANIFEST_INCLUDE.findall(text)
    )
    result = set()
    for path in references:
        # Normalize '..' without following symlinks; validation must see them.
        normalized = Path(os.path.abspath(path))
        if not normalized.is_relative_to(root):
            raise ValueError(f"compile input escapes checkout: {relative}: {path}")
        result.add(normalized.relative_to(root))
    return result


def snapshot(root, context):
    candidates = git_files(root, [*SOURCE_DIRS, *sorted(ROOT_FILES)])
    selected = {
        path for path in candidates if safe_source(path) and (root / path).exists()
    }
    pending = list(selected)
    while pending:
        relative = pending.pop()
        validate_source(root, relative)
        if relative.suffix != ".rs":
            continue
        for dependency in compile_references(root, relative):
            validate_source(root, dependency)
            if dependency in selected:
                continue
            if dependency not in git_files(root, [str(dependency)]):
                raise ValueError(f"compile input is ignored/unowned: {dependency}")
            selected.add(dependency)
            pending.append(dependency)
    for required in ("Cargo.toml", "Cargo.lock", "build.rs"):
        if Path(required) not in selected:
            raise FileNotFoundError(f"missing build dependency: {root / required}")
    source_root = context / "source"
    for relative in sorted(selected):
        destination = source_root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(validate_source(root, relative), destination)
    # build.rs reads these directories even when building a bin, not tests.
    for directory in ("tests", "tests/perf"):
        (source_root / directory).mkdir(parents=True, exist_ok=True)
    scripts = Path(__file__).resolve().parent
    shutil.copy2(scripts / "build-host/Dockerfile", context / "Dockerfile")
    shutil.copy2(Path(__file__).resolve(), context / "builder.py")


def install_export(common, output, destination):
    manifest = json.loads((output / "runtime-libs.json").read_text())
    libraries = manifest["libraries"]
    directory = manifest["directory"]
    if not isinstance(libraries, list) or len(set(libraries)) != len(libraries):
        raise ValueError("invalid exported library list")
    if any(
        not isinstance(name, str)
        or Path(name).name != name
        or not re.fullmatch(r"lib[A-Za-z0-9_.+-]+", name)
        for name in libraries
    ):
        raise ValueError("unsafe exported library name")
    if libraries:
        if not isinstance(directory, str) or not re.fullmatch(
            re.escape(destination.name) + r"\.libs/[A-Za-z0-9_-]+", directory
        ):
            raise ValueError("unsafe exported runtime directory")
    elif directory is not None:
        raise ValueError("runtime directory without libraries")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=f".{destination.name}-", dir=destination.parent
    ) as work:
        staging = Path(work)
        executable = staging / destination.name
        common.install_artifact(
            output / (destination.name + ".gz"), executable, executable=True
        )
        if libraries:
            staged_libraries = staging / "libs"
            staged_libraries.mkdir()
            for name in libraries:
                common.install_artifact(
                    output / (name + ".gz"), staged_libraries / name
                )
            installed = destination.parent / directory
            installed.parent.mkdir(parents=True, exist_ok=True)
            if installed.exists():
                if installed.is_symlink() or {
                    p.name for p in installed.iterdir()
                } != set(libraries):
                    raise ValueError(f"runtime generation collision: {installed}")
                for name in libraries:
                    if (installed / name).is_symlink() or (
                        installed / name
                    ).read_bytes() != (staged_libraries / name).read_bytes():
                        raise ValueError(
                            f"runtime generation collision: {installed / name}"
                        )
            else:
                os.replace(staged_libraries, installed)
        os.replace(executable, destination)
    return destination.parent / directory if libraries else None


def build(
    root,
    binary="wow-sim",
    release=False,
    no_default_features=False,
    features="",
    host=None,
):
    common = load_common()
    host = common.select_build_host(host)
    lock, cache, checkout_key = common.locked_checkout(root)
    with lock:
        profile = "release" if release else "debug"
        destination = root / "target" / profile / binary
        for path in (
            root / "target",
            destination.parent,
            destination.parent / (binary + ".libs"),
        ):
            if path.is_symlink():
                raise ValueError(f"artifact destination must be checkout-local: {path}")
        with (
            tempfile.TemporaryDirectory(prefix="wow-build-", dir=cache) as work,
            common.stable_context(cache, "wow-build", checkout_key) as context,
        ):
            snapshot(root, context)
            output = Path(work) / "output"
            output.mkdir()
            values = {
                "BUILD_ROOT": str(root),
                "BIN": binary,
                "RELEASE": str(int(release)),
                "NO_DEFAULT_FEATURES": str(int(no_default_features)),
                "FEATURES": features,
            }
            arguments = [
                item
                for key, value in values.items()
                for item in ("--build-arg", f"{key}={value}")
            ]
            common.execute(
                context,
                output,
                "wow-ui-sim-" + checkout_key,
                "artifact",
                arguments,
                host,
            )
            libraries = install_export(common, output, destination)
    if libraries is not None:
        print(f"Runtime libraries: {libraries}", flush=True)
    print(destination, flush=True)
    return destination


def refresh_sources(root):
    # Called inside the BuildKit sharing=locked target mount. Touch compile data
    # and .cargo config too, so a preserved Git timestamp cannot hide an edit.
    for directory, dirs, files in os.walk(root):
        dirs[:] = [name for name in dirs if name not in EXCLUDED]
        for name in files:
            os.utime(Path(directory) / name, None, follow_symlinks=False)


def cargo_arguments(binary, release, no_default_features, features):
    arguments = ["cargo", "build", "--locked", "--bin", binary, "-j", "8"]
    if release:
        arguments.append("--release")
    if no_default_features:
        arguments.append("--no-default-features")
    if features:
        arguments.extend(["--features", features])
    return arguments


def resolve_runtime_libraries(executable, profile_dir):
    sysroot = Path(
        subprocess.run(
            ["rustc", "--print", "sysroot"], check=True, text=True, capture_output=True
        ).stdout.strip()
    )
    rust_libs = sysroot / "lib/rustlib/x86_64-unknown-linux-gnu/lib"
    environment = {
        **os.environ,
        "LD_LIBRARY_PATH": os.pathsep.join(
            map(str, (profile_dir, profile_dir / "deps", rust_libs))
        ),
    }
    result = subprocess.run(
        ["ldd", str(executable)],
        check=True,
        text=True,
        capture_output=True,
        env=environment,
    )
    libraries = {}
    for line in result.stdout.splitlines():
        if "not found" in line:
            raise ValueError(f"unresolved runtime dependency: {line.strip()}")
        match = re.match(r"\s*(\S+)\s+=>\s+(/.+?)\s+\(", line)
        if match and match[1] not in SYSTEM_LIBRARIES:
            libraries[match[1]] = Path(match[2])
    return libraries


def compress_artifact(source, destination):
    with (
        source.open("rb") as input_file,
        gzip.open(destination, "wb", compresslevel=1) as output,
    ):
        shutil.copyfileobj(input_file, output)


def export_binary(executable, profile_dir, output):
    libraries = resolve_runtime_libraries(executable, profile_dir)
    output.mkdir(parents=True, exist_ok=True)
    # Work on copies: never mutate Cargo cache artifacts or system/toolchain libs.
    with tempfile.TemporaryDirectory(prefix="wow-export-") as work:
        staging = Path(work)
        copied_binary = staging / executable.name
        shutil.copy2(executable, copied_binary)
        digest = hashlib.sha256()
        for name, source in sorted(libraries.items()):
            digest.update(name.encode())
            digest.update(source.read_bytes())
        directory = (
            executable.name + ".libs/" + digest.hexdigest()[:20] if libraries else None
        )
        if libraries:
            subprocess.run(
                ["patchelf", "--set-rpath", "$ORIGIN/" + directory, str(copied_binary)],
                check=True,
            )
            for name, source in sorted(libraries.items()):
                copied = staging / name
                shutil.copy2(source, copied)
                subprocess.run(
                    ["patchelf", "--set-rpath", "$ORIGIN", str(copied)], check=True
                )
                compress_artifact(copied, output / (name + ".gz"))
        compress_artifact(copied_binary, output / (executable.name + ".gz"))
    (output / "runtime-libs.json").write_text(
        json.dumps({"directory": directory, "libraries": sorted(libraries)})
    )


def container_build(output=Path("/out")):
    root = Path(os.environ["BUILD_ROOT"])
    binary = os.environ["BIN"]
    release = os.environ["RELEASE"] == "1"
    refresh_sources(root)
    subprocess.run(
        cargo_arguments(
            binary,
            release,
            os.environ["NO_DEFAULT_FEATURES"] == "1",
            os.environ["FEATURES"],
        ),
        cwd=root,
        check=True,
    )
    profile_dir = root / "target" / ("release" if release else "debug")
    export_binary(profile_dir / binary, profile_dir, output)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--build-host", choices=("desktop", "local"))
    parser.add_argument(
        "--save-build-host",
        choices=("desktop", "local"),
        help="save shared default and exit without building",
    )
    parser.add_argument("--bin", choices=("wow-sim", "wow-cli"), default=None)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument(
        "--features",
        default=None,
        help="Cargo comma-separated feature list; defaults remain enabled unless explicitly disabled",
    )
    args = parser.parse_args(argv)
    try:
        if args.save_build_host:
            if (
                args.build_host
                or args.bin
                or args.release
                or args.no_default_features
                or args.features is not None
            ):
                parser.error("--save-build-host cannot combine with build options")
            load_common().save_build_host(args.save_build_host)
        else:
            build(
                args.root.resolve(),
                args.bin or "wow-sim",
                args.release,
                args.no_default_features,
                args.features or "",
                args.build_host,
            )
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"Build failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["_container-build"]:
        container_build()
    else:
        sys.exit(main())
