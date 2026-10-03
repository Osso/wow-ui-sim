"""Behavioral source/export contract; fake transport never runs Docker or a client."""

import gzip
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

HELPER = Path(__file__).resolve().parents[1] / "build-host.py"
COMMON = Path("/syncthing/Sync/Projects/world-of-osso/game-engine/scripts")

FAKE_TRANSPORT = r"""#!/usr/bin/env python3
import gzip
import json
import os
from pathlib import Path
import sys
context, output, key, target, args, host = sys.argv[1:]
context, output = Path(context), Path(output)
args = json.loads(args)
values = dict(value.split("=", 1) for value in args[1::2])
files = {str(p.relative_to(context)): p.read_text() for p in context.rglob("*") if p.is_file() and p.suffix != ".gz"}
request = dict(files=files, key=key, target=target, args=values, host=host)
Path(os.environ["FAKE_RECORD"]).write_text(json.dumps(request))
if os.environ.get("FAKE_FAIL"):
    sys.exit(9)
output.mkdir(exist_ok=True)
binary = values["BIN"]
libs = ["libiced_dynamic.so", "libstd-fixture.so"] if os.environ.get("FAKE_LIBS") else []
manifest = {"directory": binary + ".libs/fixture" if libs else None, "libraries": libs}
(output / "runtime-libs.json").write_text(json.dumps(manifest))
for name in [binary, *libs]:
    with gzip.open(output / (name + ".gz"), "wb") as stream:
        stream.write(("new " + name).encode())
if os.environ.get("FAKE_BAD_EXPORT"):
    (output / (libs[-1] + ".gz")).write_bytes(b"not gzip")
"""


class BuildHostTests(unittest.TestCase):
    def setUp(self):
        self.work = tempfile.TemporaryDirectory()
        self.addCleanup(self.work.cleanup)
        self.base = Path(self.work.name)
        self.root = self.base / "checkout with spaces"
        self.root.mkdir()
        self.common = self.base / "common"
        self.common.mkdir()
        shutil.copy2(COMMON / "depot-build.py", self.common / "depot-build.py")
        self.transport = self.base / "transport"
        self.transport.write_text(FAKE_TRANSPORT)
        self.transport.chmod(0o755)
        (self.common / "build_hosts.py").write_text(
            "import json, os, subprocess\n"
            "def execute(context, output, checkout_key, target, build_args, host):\n"
            "    subprocess.run([os.environ['FAKE_TRANSPORT'], str(context), str(output), checkout_key, target, json.dumps(build_args), host], check=True)\n"
        )
        self.record = self.base / "request.json"
        self.env = {
            **os.environ,
            "HOME": str(self.base / "home"),
            "XDG_CACHE_HOME": str(self.base / "cache"),
            "BUILD_HOST_SCRIPTS": str(self.common),
            "FAKE_RECORD": str(self.record),
            "FAKE_TRANSPORT": str(self.transport),
        }
        self.env.pop("FAKE_FAIL", None)
        self.env.pop("FAKE_LIBS", None)
        self.git("init", "-q")
        self.put("Cargo.toml", '[package]\nname="fixture"\nversion="0.1.0"\n')
        self.put("Cargo.lock", "lock fixture")
        self.put(".cargo/config.toml", "[build]\njobs=4\n")
        self.put(".gitignore", "target/\ncache/\n*.ignored\n")
        self.put("build.rs", "fn main() {}")
        self.put("src/lib.rs", '#[path = "../data/owned.rs"] mod owned;\n')
        self.put("src/deleted.rs", "old deleted")
        self.put("src/bootstrap.lua", "owned Lua")
        self.put("data/owned.rs", 'const TEXT: &str = include_str!("profile.txt");')
        self.put("data/profile.txt", "owned manifest")
        self.put("data/runtime.csv", "runtime not compile input")
        self.put("Interface/AddOns/User/User.lua", "private addon")
        self.git("add", ".")
        self.git(
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.org",
            "commit",
            "-qm",
            "fixture",
        )
        self.put(
            "src/lib.rs",
            '#[path = "../data/owned.rs"] mod owned;\nconst NEW: &str = include_str!("new.lua");\n',
        )
        self.put("src/new.lua", "working untracked Lua")
        self.put("native/shim.c", "working untracked native")
        self.put("src/.env", "SECRET")
        self.put("src/private.key", "SECRET")
        self.put("src/cache/result.rs", "SECRET")
        self.put("src/ignored.ignored", "SECRET")
        self.put("target/debug/wow-sim", "old binary")
        (self.root / "src/deleted.rs").unlink()

    def git(self, *args):
        subprocess.run(
            ["git", "-C", str(self.root), *args], check=True, capture_output=True
        )

    def put(self, relative, contents):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)
        return path

    def invoke(self, *args, success=True):
        result = subprocess.run(
            [sys.executable, str(HELPER), "--root", str(self.root), *args],
            env=self.env,
            text=True,
            capture_output=True,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def request(self):
        return json.loads(self.record.read_text())

    def test_snapshot_preserves_working_inputs_deletions_and_excludes_runtime_and_secrets(
        self,
    ):
        self.invoke("--build-host", "desktop")
        files = self.request()["files"]
        for relative in (
            "src/lib.rs",
            "src/new.lua",
            "native/shim.c",
            ".cargo/config.toml",
            "data/owned.rs",
            "data/profile.txt",
            "Cargo.lock",
            "build.rs",
        ):
            self.assertEqual(
                files["source/" + relative], (self.root / relative).read_text()
            )
        for relative in (
            "src/deleted.rs",
            "data/runtime.csv",
            "Interface/AddOns/User/User.lua",
            "src/.env",
            "src/private.key",
            "src/cache/result.rs",
            "src/ignored.ignored",
            "target/debug/wow-sim",
        ):
            self.assertNotIn("source/" + relative, files)
        self.assertEqual(
            (self.root / "target/debug/wow-sim").read_text(), "new wow-sim"
        )
        self.assertTrue(os.access(self.root / "target/debug/wow-sim", os.X_OK))

    def test_inline_module_path_uses_whole_source_directory_and_manifest_data_include(
        self,
    ):
        self.put(
            "src/inline.rs",
            '#[cfg(test)] mod tests { #[path = "fixture.rs"] mod fixture; }\nconst DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/new.csv"));',
        )
        self.put("src/inline/tests/fixture.rs", "fn fixture() {}")
        self.put("data/new.csv", "compile CSV")
        self.invoke("--build-host", "local")
        files = self.request()["files"]
        self.assertEqual(files["source/data/new.csv"], "compile CSV")
        self.assertEqual(files["source/src/inline/tests/fixture.rs"], "fn fixture() {}")

    def test_default_features_originating_path_and_release_cli_contract(self):
        self.invoke("--build-host", "local")
        first = self.request()
        self.assertEqual(first["args"]["BUILD_ROOT"], str(self.root))
        self.assertEqual(first["args"]["NO_DEFAULT_FEATURES"], "0")
        self.assertEqual(first["args"]["FEATURES"], "")
        self.assertEqual(first["host"], "local")
        self.invoke(
            "--build-host",
            "desktop",
            "--bin",
            "wow-cli",
            "--release",
            "--no-default-features",
            "--features",
            "sound,gui,client-mists,fast-build",
        )
        second = self.request()
        self.assertEqual(second["args"]["BIN"], "wow-cli")
        self.assertEqual(second["args"]["RELEASE"], "1")
        self.assertEqual(second["args"]["NO_DEFAULT_FEATURES"], "1")
        self.assertEqual(
            second["args"]["FEATURES"], "sound,gui,client-mists,fast-build"
        )
        self.assertEqual(first["key"], second["key"])
        self.assertEqual(
            (self.root / "target/release/wow-cli").read_text(), "new wow-cli"
        )

    def test_save_only_shared_default_explicit_override_and_failed_build_preserve_state(
        self,
    ):
        setting = Path(self.env["HOME"]) / ".config/game-engine/build-host"
        self.invoke("--save-build-host", "desktop")
        self.assertEqual(setting.read_text(), "desktop\n")
        self.assertFalse(self.record.exists())
        self.invoke()
        self.assertEqual(self.request()["host"], "desktop")
        self.invoke("--build-host", "local")
        self.assertEqual(self.request()["host"], "local")
        self.put("target/debug/wow-sim", "last good")
        self.env["FAKE_FAIL"] = "1"
        self.invoke(success=False)
        self.assertEqual(self.request()["host"], "desktop")
        self.assertEqual((self.root / "target/debug/wow-sim").read_text(), "last good")
        self.assertEqual(setting.read_text(), "desktop\n")

    def test_missing_setting_invalid_setting_and_save_with_build_options_fail(self):
        self.invoke(success=False)
        self.assertFalse(self.record.exists())
        setting = Path(self.env["HOME"]) / ".config/game-engine/build-host"
        setting.parent.mkdir(parents=True)
        setting.write_text("unknown\n")
        self.invoke(success=False)
        self.assertFalse(self.record.exists())
        self.invoke(
            "--save-build-host", "local", "--features", "fast-build", success=False
        )
        self.assertEqual(setting.read_text(), "unknown\n")

    def test_runtime_libraries_export_before_executable_and_static_mode_keeps_other_bins(
        self,
    ):
        self.env["FAKE_LIBS"] = "1"
        result = self.invoke("--build-host", "local", "--features", "fast-build")
        libraries = self.root / "target/debug/wow-sim.libs/fixture"
        self.assertIn(f"Runtime libraries: {libraries}", result.stdout)
        self.assertEqual(
            (libraries / "libiced_dynamic.so").read_text(), "new libiced_dynamic.so"
        )
        self.assertEqual(
            (libraries / "libstd-fixture.so").read_text(), "new libstd-fixture.so"
        )
        self.env.pop("FAKE_LIBS")
        self.invoke("--build-host", "desktop", "--bin", "wow-cli")
        self.assertTrue((libraries / "libiced_dynamic.so").is_file())

    def test_incomplete_library_export_preserves_previous_binary_and_libraries(self):
        self.env["FAKE_LIBS"] = "1"
        self.invoke("--build-host", "desktop", "--features", "fast-build")
        self.put("target/debug/wow-sim", "last good executable")
        self.env["FAKE_BAD_EXPORT"] = "1"
        self.invoke(
            "--build-host", "desktop", "--features", "fast-build", success=False
        )
        self.assertEqual(
            (self.root / "target/debug/wow-sim").read_text(), "last good executable"
        )
        self.assertEqual(
            (
                self.root / "target/debug/wow-sim.libs/fixture/libstd-fixture.so"
            ).read_text(),
            "new libstd-fixture.so",
        )

    def test_compile_include_missing_or_secret_and_source_symlink_fail_before_transport(
        self,
    ):
        for reference in ("../data/missing.csv", ".env"):
            self.put("src/new.rs", f'const INPUT: &str = include_str!("{reference}");')
            self.invoke("--build-host", "local", success=False)
            self.assertFalse(self.record.exists())
        (self.root / "src/new.rs").unlink()
        (self.root / "src/link.rs").symlink_to(self.root / "src/lib.rs")
        self.invoke("--build-host", "local", success=False)
        self.assertFalse(self.record.exists())


FAKE_NATIVE = r"""#!/usr/bin/env python3
import json, os
from pathlib import Path
import sys
name = Path(sys.argv[0]).name
root = Path(os.environ["BUILD_ROOT"])
if name == "cargo":
    args = sys.argv[1:]
    profile = "release" if "--release" in args else "debug"
    binary = args[args.index("--bin") + 1]
    features = args[args.index("--features") + 1].split(",") if "--features" in args else []
    if "--no-default-features" not in args:
        features += ["sound", "gui", "casc", "client-retail"]
    artifact = root / "target" / profile / binary
    artifact.parent.mkdir(parents=True, exist_ok=True)
    payload = dict(manifest=str(Path.cwd()), features=sorted(set(features)), profile=profile,
                   mtimes={str(p.relative_to(root)): p.stat().st_mtime_ns for p in root.rglob("*") if p.is_file() and "target" not in p.relative_to(root).parts})
    artifact.write_text(json.dumps(payload))
elif name == "rustc":
    print(root / "toolchain")
elif name == "ldd":
    for library in ["libc.so.6", "libicui18n.so.72", "libicuuc.so.72", "libicudata.so.72", "libiced_dynamic.so", "libstd-matching.so"]:
        if os.environ.get("FAKE_UNRESOLVED") and library == "libstd-matching.so":
            print(library + " => not found")
        else:
            print(library + " => " + str(root / "deps" / library) + " (0x123)")
elif name == "patchelf":
    path = Path(sys.argv[-1])
    path.write_text(path.read_text() + "\nRUNPATH=" + sys.argv[2])
else:
    sys.exit(2)
"""


class ContainerExportTests(unittest.TestCase):
    def setUp(self):
        self.work = tempfile.TemporaryDirectory()
        self.addCleanup(self.work.cleanup)
        self.root = Path(self.work.name) / "origin with spaces"
        self.root.mkdir()
        self.output = self.root.parent / "output"
        self.commands = self.root.parent / "commands"
        self.commands.mkdir()
        for name in ("cargo", "rustc", "ldd", "patchelf"):
            path = self.commands / name
            path.write_text(FAKE_NATIVE)
            path.chmod(0o755)
        self.libraries = [
            "libc.so.6",
            "libicui18n.so.72",
            "libicuuc.so.72",
            "libicudata.so.72",
            "libiced_dynamic.so",
            "libstd-matching.so",
        ]
        for name in self.libraries:
            path = self.root / "deps" / name
            path.parent.mkdir(exist_ok=True)
            path.write_text("original " + name)
        for relative in (
            "src/lib.rs",
            "data/owned.csv",
            ".cargo/config.toml",
            "target/cached",
        ):
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("input")
            os.utime(path, ns=(1_000_000_000, 1_000_000_000))
        spec = importlib.util.spec_from_file_location("wow_build", HELPER)
        self.helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.helper)
        self.environment = {
            "PATH": str(self.commands) + os.pathsep + os.environ["PATH"],
            "BUILD_ROOT": str(self.root),
            "BIN": "wow-cli",
            "RELEASE": "1",
            "NO_DEFAULT_FEATURES": "1",
            "FEATURES": "gui,client-mists,fast-build",
        }

    def test_container_compile_and_export_features_embedded_paths_and_only_needed_libraries(
        self,
    ):
        with patch.dict(os.environ, self.environment):
            self.helper.container_build(self.output)
        manifest = json.loads((self.output / "runtime-libs.json").read_text())
        expected = set(self.libraries) - {"libc.so.6"}
        self.assertEqual(set(manifest["libraries"]), expected)
        binary = gzip.decompress((self.output / "wow-cli.gz").read_bytes()).decode()
        payload, runpath = binary.split("\nRUNPATH=")
        payload = json.loads(payload)
        self.assertEqual(payload["manifest"], str(self.root))
        self.assertEqual(payload["features"], ["client-mists", "fast-build", "gui"])
        self.assertEqual(payload["profile"], "release")
        for relative in ("src/lib.rs", "data/owned.csv", ".cargo/config.toml"):
            self.assertGreater(payload["mtimes"][relative], 1_000_000_000)
        self.assertEqual(
            (self.root / "target/cached").stat().st_mtime_ns, 1_000_000_000
        )
        self.assertEqual(runpath, "$ORIGIN/" + manifest["directory"])
        for name in expected:
            self.assertEqual(
                gzip.decompress((self.output / (name + ".gz")).read_bytes()).decode(),
                "original " + name + "\nRUNPATH=$ORIGIN",
            )
            self.assertEqual(
                (self.root / "deps" / name).read_text(), "original " + name
            )
        original = json.loads((self.root / "target/release/wow-cli").read_text())
        self.assertEqual(original["features"], payload["features"])

    def test_container_default_features_and_missing_runtime_dependency_fail_explicitly(
        self,
    ):
        self.environment.update(RELEASE="0", NO_DEFAULT_FEATURES="0", FEATURES="")
        with patch.dict(os.environ, self.environment):
            self.helper.container_build(self.output)
        binary = gzip.decompress((self.output / "wow-cli.gz").read_bytes()).decode()
        payload = json.loads(binary.split("\nRUNPATH=")[0])
        self.assertEqual(payload["features"], ["casc", "client-retail", "gui", "sound"])
        self.assertEqual(payload["profile"], "debug")
        self.environment["FAKE_UNRESOLVED"] = "1"
        previous = (self.output / "wow-cli.gz").read_bytes()
        with patch.dict(os.environ, self.environment):
            with self.assertRaisesRegex(ValueError, "libstd-matching.so.*not found"):
                self.helper.container_build(self.output)
        self.assertEqual((self.output / "wow-cli.gz").read_bytes(), previous)


if __name__ == "__main__":
    unittest.main()
