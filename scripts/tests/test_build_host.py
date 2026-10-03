"""Behavioral source/export contract; fake transport never runs Docker or a client."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

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
        self.invoke("--build-host", "local", "--features", "fast-build")
        libraries = self.root / "target/debug/wow-sim.libs/fixture"
        self.assertEqual(
            (libraries / "libiced_dynamic.so").read_text(), "new libiced_dynamic.so"
        )
        self.assertEqual(
            (libraries / "libstd-fixture.so").read_text(), "new libstd-fixture.so"
        )
        self.env.pop("FAKE_LIBS")
        self.invoke("--build-host", "desktop", "--bin", "wow-cli")
        self.assertTrue((libraries / "libiced_dynamic.so").is_file())

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


if __name__ == "__main__":
    unittest.main()
