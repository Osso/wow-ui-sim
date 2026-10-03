#!/usr/bin/env python3
"""Build and optionally run natively on the shared desktop/local build host.

Requires game-engine scripts at
/syncthing/Sync/Projects/world-of-osso/game-engine/scripts.
BUILD_HOST_SCRIPTS overrides that dependency directory; no host fallback.
The saved default is shared with game-engine. Desktop artifacts stay on desktop.
"""

import argparse
import importlib.util
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

COMMON_SCRIPTS = Path("/syncthing/Sync/Projects/world-of-osso/game-engine/scripts")
PROJECT_NAME = "wow-ui-sim"
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
ADDON_SUFFIXES = {".lua", ".xml", ".toc"}
LITERAL_INCLUDE = re.compile(r'\binclude(?:_str|_bytes)?!\s*\(\s*"([^"\n]+)"')
MANIFEST_INCLUDE = re.compile(
    r"\binclude(?:_str|_bytes)?!\s*\(\s*concat!\s*\(\s*"
    r'env!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*,\s*"([^"\n]+)"\s*\)\s*\)',
    re.MULTILINE,
)
PATH_ATTRIBUTE = re.compile(r'#\[[^\]]*?\bpath\s*=\s*"([^"\n]+)"', re.MULTILINE)


def load_common():
    directory = Path(os.environ.get("BUILD_HOST_SCRIPTS", COMMON_SCRIPTS)).resolve()
    for name in ("depot-build.py", "build_hosts.py", "native_build_hosts.py"):
        if not (directory / name).is_file():
            raise FileNotFoundError(
                f"missing shared build dependency: {directory / name}"
            )
    sys.path.insert(0, str(directory))
    modules = []
    for name, filename in (
        ("engine_build_helper", "depot-build.py"),
        ("engine_native_build_helper", "native_build_hosts.py"),
    ):
        spec = importlib.util.spec_from_file_location(name, directory / filename)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        modules.append(module)
    return tuple(modules)


def git_files(root, paths, tracked_only=False):
    selection = ["--cached"]
    if not tracked_only:
        selection.extend(["--others", "--exclude-standard"])
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", *selection, "--", *paths],
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


def has_symlink(root, relative):
    return (root / relative).is_symlink() or any(
        (root / parent).is_symlink() for parent in relative.parents
    )


def validate_source(root, relative):
    if not safe_source(relative):
        raise ValueError(f"excluded compile input: {relative}")
    source = root / relative
    if has_symlink(root, relative):
        raise ValueError(f"unsupported source symlink: {source}")
    if not source.is_file():
        raise FileNotFoundError(f"missing compile input: {source}")
    return source


def compile_references(root, relative):
    """Follow literal Rust includes/path attributes, not runtime path strings."""
    text = (root / relative).read_text()
    references = [
        (root / relative).parent / value for value in LITERAL_INCLUDE.findall(text)
    ]
    # Inline-module path resolution belongs to rustc; source dirs are copied whole.
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
        normalized = Path(os.path.abspath(path))
        if not normalized.is_relative_to(root):
            raise ValueError(f"compile input escapes checkout: {relative}: {path}")
        result.add(normalized.relative_to(root))
    return result


def runtime_sources(root):
    tracked = git_files(
        root, ["Interface", "data/blizzard-ui-files"], tracked_only=True
    )
    owned_addons = {
        path.parts[2]
        for path in tracked
        if path.parts[:2] == ("Interface", "AddOns") and len(path.parts) > 3
    }
    candidates = set(tracked)
    for path in git_files(root, ["Interface/AddOns"]):
        if (
            len(path.parts) > 3
            and path.parts[2] in owned_addons
            and path.suffix.lower() in ADDON_SUFFIXES
        ):
            candidates.add(path)
    return {
        path
        for path in candidates
        if safe_source(path)
        and path.parts[:2] != ("Interface", "BlizzardUI")
        and not has_symlink(root, path)
        and (root / path).is_file()
        and (
            path.parts[0] == "Interface"
            or (len(path.parts) == 3 and path.suffix == ".txt")
        )
    }


def test_addon_sources(root):
    tracked = git_files(root, ["test_addons"], tracked_only=True)
    owned = {
        path.parts[1]
        for path in tracked
        if len(path.parts) > 2
        and safe_source(path)
        and path.suffix.lower() in ADDON_SUFFIXES
    }
    return {
        path
        for path in git_files(root, ["test_addons"])
        if len(path.parts) > 2
        and path.parts[1] in owned
        and path.suffix.lower() in ADDON_SUFFIXES
        and safe_source(path)
        and not has_symlink(root, path)
        and (root / path).is_file()
    }


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
    source_root = context / PROJECT_NAME
    for relative in sorted(selected | runtime_sources(root) | test_addon_sources(root)):
        destination = source_root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(validate_source(root, relative), destination)
    # build.rs reads these directories even when building a bin, not tests.
    for directory in ("tests", "tests/perf"):
        (source_root / directory).mkdir(parents=True, exist_ok=True)


def cargo_arguments(binary, no_default_features, features, mode="build", test_args=()):
    arguments = [mode]
    if mode != "test":
        arguments.extend(["--bin", binary])
    if no_default_features:
        arguments.append("--no-default-features")
    if features:
        arguments.extend(["--features", features])
    arguments.extend(test_args)
    return arguments


def build(
    root,
    binary="wow-sim",
    release=False,
    no_default_features=False,
    features="",
    host=None,
    runtime_args=None,
    mode="build",
    test_args=(),
    build=True,
):
    common, native = load_common()
    host = common.select_build_host(host)
    cache, checkout_key = common.prepare_checkout_cache(root)
    with tempfile.TemporaryDirectory(prefix="wow-native-", dir=cache) as work:
        context = Path(work)
        snapshot(root, context)
        return native.execute(
            context,
            checkout_key,
            PROJECT_NAME,
            cargo_arguments(binary, no_default_features, features, mode, test_args),
            host,
            runtime_args=runtime_args,
            binary=binary,
            release=release,
            root=root,
            build=build,
        )


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
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="run an existing binary without Cargo; requires --run",
    )
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument(
        "--features",
        default=None,
        help="Cargo comma-separated feature list; defaults remain enabled unless explicitly disabled",
    )
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument(
        "--run",
        action="store_true",
        help="run on the build host after a successful build",
    )
    modes.add_argument(
        "--test",
        action="store_true",
        help="run native cargo test; last helper flag, all following Cargo arguments preserved",
    )
    modes.add_argument("--check", action="store_true", help="run native cargo check")
    arguments = list(sys.argv[1:] if argv is None else argv)
    runtime_args = []
    test_args = []
    helper_end = arguments.index("--") if "--" in arguments else len(arguments)
    if "--test" in arguments[:helper_end]:
        split = arguments.index("--test") + 1
        test_args = arguments[split:]
        args = parser.parse_args(arguments[:split])
    elif "--" in arguments:
        separator = arguments.index("--")
        runtime_args = arguments[separator + 1 :]
        arguments = arguments[:separator]
        args = parser.parse_args(arguments)
        if not args.run:
            parser.error("runtime arguments require --run")
    else:
        args = parser.parse_args(arguments)
    if args.no_build and (not args.run or args.save_build_host):
        parser.error(
            "--no-build requires --run and cannot combine with --save-build-host"
        )
    try:
        if args.save_build_host:
            if (
                args.build_host
                or args.bin
                or args.release
                or args.no_default_features
                or args.features is not None
                or args.run
                or args.test
                or args.check
            ):
                parser.error("--save-build-host cannot combine with build options")
            common, _ = load_common()
            common.save_build_host(args.save_build_host)
            return 0
        return build(
            args.root.resolve(),
            args.bin or "wow-sim",
            args.release,
            args.no_default_features,
            args.features or "",
            args.build_host,
            runtime_args if args.run else None,
            "test" if args.test else "check" if args.check else "build",
            test_args,
            build=not args.no_build,
        )
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"Build failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
