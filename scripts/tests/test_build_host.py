"""Native adapter process fixtures; no Cargo, remote hosts, or GUI execution."""

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

FAKE_MODULE = """import json, os, subprocess, sys
from pathlib import Path

def execute(context, checkout_key, project_name, cargo_args, host,
            runtime_args=None, binary=None, release=False, environment=None, root=None):
    request = dict(context=str(context), key=checkout_key, project=project_name,
                   cargo_args=cargo_args, host=host, runtime_args=runtime_args,
                   binary=binary, release=release, environment=environment, root=str(root))
    return subprocess.run([sys.executable, os.environ["FAKE_CHILD"], json.dumps(request)]).returncode
"""

FAKE_CHILD = """import json, os, shutil, sys
from pathlib import Path
request = json.loads(sys.argv[1])
context = Path(request.pop("context"))
output = Path(os.environ["FAKE_OUTPUT"])
if output.exists():
    shutil.rmtree(output)
shutil.copytree(context, output)
Path(os.environ["FAKE_RECORD"]).write_text(json.dumps(request))
print("native build " + request["host"], flush=True)
status = int(os.environ.get("FAKE_BUILD_STATUS", "0"))
if status:
    sys.exit(status)
print("native artifact=/worker/target/" + ("release" if request["release"] else "debug") + "/" + request["binary"], flush=True)
if request["runtime_args"] is not None:
    Path(os.environ["FAKE_RUNTIME"]).write_text(json.dumps(dict(args=request["runtime_args"], root=request["root"], host=request["host"])))
    print("native runtime", flush=True)
    sys.exit(int(os.environ.get("FAKE_RUN_STATUS", "0")))
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
        # depot-build imports the old backend for unrelated engine entry points.
        (self.common / "build_hosts.py").write_text(
            "def execute(*args):\n    raise RuntimeError('Docker forbidden')\n"
        )
        (self.common / "native_build_hosts.py").write_text(FAKE_MODULE)
        child = self.base / "child.py"
        child.write_text(FAKE_CHILD)
        self.record = self.base / "request.json"
        self.output = self.base / "snapshot"
        self.runtime = self.base / "runtime.json"
        self.env = {
            **os.environ,
            "HOME": str(self.base / "home"),
            "XDG_CACHE_HOME": str(self.base / "cache"),
            "BUILD_HOST_SCRIPTS": str(self.common),
            "FAKE_CHILD": str(child),
            "FAKE_RECORD": str(self.record),
            "FAKE_OUTPUT": str(self.output),
            "FAKE_RUNTIME": str(self.runtime),
        }
        for name in ("FAKE_BUILD_STATUS", "FAKE_RUN_STATUS", "FAKE_FAIL"):
            self.env.pop(name, None)
        self.git("init", "-q")
        inputs = {
            "Cargo.toml": '[package]\nname="fixture"\nversion="0.1.0"\n',
            "Cargo.lock": "lock fixture",
            ".cargo/config.toml": "[build]\njobs=4\n",
            ".gitignore": "target/\ncache/\n*.ignored\n",
            "build.rs": "fn main() {}",
            "src/lib.rs": '#[path = "../data/owned.rs"] mod owned;\n',
            "src/deleted.rs": "old deleted",
            "src/bootstrap.lua": "owned Lua",
            "data/owned.rs": 'const TEXT: &str = include_str!("profile.txt");',
            "data/profile.txt": "owned compile manifest",
            "data/runtime.csv": "not compile input",
            "data/blizzard-ui-files/mists.txt": "Blizzard_Test/Test.lua",
            "Interface/AddOns/Admin/Admin.lua": "tracked addon",
            "Interface/AddOns/Admin/Admin.toc": "Admin.lua",
            "Interface/AddOns/Admin/icon.webp": "tracked texture",
            "Interface/TestAddOns/Wowless/tests/owned.lua": "tracked test overlay",
        }
        for name, value in inputs.items():
            self.put(name, value)
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
        self.put("Interface/AddOns/Admin/new.xml", "working untracked XML")
        self.put("Interface/AddOns/Admin/new.lua", "working untracked addon Lua")
        self.put("Interface/AddOns/Admin/new.toc", "new.lua")
        self.put("Interface/AddOns/Private/private.lua", "unowned addon")
        for name in (
            "src/.env",
            "src/private.key",
            "src/cache/result.rs",
            "src/ignored.ignored",
            "Interface/AddOns/Admin/SavedVariables/state.lua",
            "Interface/AddOns/Admin/.env",
            "Interface/AddOns/Admin/unknown.bin",
            "data/blizzard-ui-files/private.txt",
            "SavedVariables/session.lua",
        ):
            self.put(name, "SECRET")
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

    def invoke(self, *args, status=0):
        result = subprocess.run(
            [sys.executable, str(HELPER), "--root", str(self.root), *args],
            env=self.env,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, status, result.stdout + result.stderr)
        return result

    def request(self):
        return json.loads(self.record.read_text())

    def test_snapshot_compile_and_runtime_inputs_preserve_bytes_without_exports(self):
        external = self.base / "external"
        external.mkdir()
        (external / "private.lua").write_text("SECRET")
        (self.root / "Interface/BlizzardUI").symlink_to(external)
        (self.root / "Interface/TestAddOns/Wowless/external").symlink_to(external)
        self.invoke("--build-host", "desktop")
        for relative in (
            "src/lib.rs",
            "src/new.lua",
            "native/shim.c",
            ".cargo/config.toml",
            "data/owned.rs",
            "data/profile.txt",
            "Cargo.lock",
            "build.rs",
            "Interface/AddOns/Admin/Admin.lua",
            "Interface/AddOns/Admin/icon.webp",
            "Interface/AddOns/Admin/new.xml",
            "Interface/AddOns/Admin/new.lua",
            "Interface/AddOns/Admin/new.toc",
            "Interface/TestAddOns/Wowless/tests/owned.lua",
            "data/blizzard-ui-files/mists.txt",
        ):
            self.assertEqual(
                (self.output / "wow-ui-sim" / relative).read_bytes(),
                (self.root / relative).read_bytes(),
            )
        for relative in (
            "src/deleted.rs",
            "data/runtime.csv",
            "src/.env",
            "src/private.key",
            "src/cache/result.rs",
            "src/ignored.ignored",
            "target/debug/wow-sim",
            "Interface/AddOns/Private/private.lua",
            "Interface/AddOns/Admin/SavedVariables/state.lua",
            "Interface/AddOns/Admin/.env",
            "Interface/AddOns/Admin/unknown.bin",
            "data/blizzard-ui-files/private.txt",
            "Interface/BlizzardUI/private.lua",
            "Interface/TestAddOns/Wowless/external/private.lua",
            "SavedVariables/session.lua",
        ):
            self.assertFalse((self.output / "wow-ui-sim" / relative).exists(), relative)
        self.assertEqual((self.root / "target/debug/wow-sim").read_text(), "old binary")
        self.assertEqual({p.name for p in self.output.iterdir()}, {"wow-ui-sim"})

    def test_inline_module_paths_and_manifest_compile_include(self):
        self.put(
            "src/inline.rs",
            '#[cfg(test)] mod tests { #[path = "fixture.rs"] mod fixture; }\nconst DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/new.csv"));',
        )
        self.put("src/inline/tests/fixture.rs", "fn fixture() {}")
        self.put("data/new.csv", "compile CSV")
        self.invoke("--build-host", "desktop")
        self.assertEqual(
            (self.output / "wow-ui-sim/data/new.csv").read_text(), "compile CSV"
        )
        self.assertEqual(
            (self.output / "wow-ui-sim/src/inline/tests/fixture.rs").read_text(),
            "fn fixture() {}",
        )

    def test_default_build_and_explicit_release_features(self):
        result = self.invoke("--build-host", "desktop")
        first = self.request()
        self.assertEqual(first["cargo_args"], ["build", "--bin", "wow-sim"])
        self.assertEqual(first["project"], "wow-ui-sim")
        self.assertIsNone(first["runtime_args"])
        self.assertEqual(first["root"], str(self.root))
        self.assertIn("native artifact=/worker/target/debug/wow-sim", result.stdout)
        self.invoke(
            "--build-host",
            "local",
            "--bin",
            "wow-cli",
            "--release",
            "--no-default-features",
            "--features",
            "sound,gui,client-mists,fast-build",
        )
        second = self.request()
        self.assertEqual(
            second["cargo_args"],
            [
                "build",
                "--bin",
                "wow-cli",
                "--no-default-features",
                "--features",
                "sound,gui,client-mists,fast-build",
            ],
        )
        self.assertTrue(second["release"])
        self.assertEqual(first["key"], second["key"])
        self.assertEqual(second["host"], "local")

    def test_runtime_arguments_empty_and_passthrough_and_exit_status(self):
        for host in ("desktop", "local"):
            self.invoke("--build-host", host, "--run")
            self.assertEqual(json.loads(self.runtime.read_text())["args"], [])
            self.env["FAKE_RUN_STATUS"] = "23"
            self.invoke(
                "--build-host",
                host,
                "--bin",
                "wow-cli",
                "--run",
                "--",
                "--no-saved-vars",
                "lua",
                "return 'two words'",
                status=23,
            )
            runtime = json.loads(self.runtime.read_text())
            self.assertEqual(
                runtime,
                {
                    "args": ["--no-saved-vars", "lua", "return 'two words'"],
                    "root": str(self.root),
                    "host": host,
                },
            )
            self.env.pop("FAKE_RUN_STATUS")

    def test_failed_build_does_not_run_and_preserves_status(self):
        self.env["FAKE_BUILD_STATUS"] = "9"
        self.invoke("--build-host", "desktop", "--run", "--", "self-test", status=9)
        self.assertFalse(self.runtime.exists())
        self.assertEqual((self.root / "target/debug/wow-sim").read_text(), "old binary")

    def test_shared_saved_default_and_explicit_override(self):
        setting = Path(self.env["HOME"]) / ".config/game-engine/build-host"
        self.invoke("--save-build-host", "desktop")
        self.assertEqual(setting.read_text(), "desktop\n")
        self.assertFalse(self.record.exists())
        self.invoke()
        self.assertEqual(self.request()["host"], "desktop")
        self.invoke("--build-host", "local", "--run")
        self.assertEqual(json.loads(self.runtime.read_text())["host"], "local")
        self.assertEqual(setting.read_text(), "desktop\n")

    def test_invalid_cli_or_default_does_not_execute(self):
        self.invoke(status=1)
        setting = Path(self.env["HOME"]) / ".config/game-engine/build-host"
        setting.parent.mkdir(parents=True)
        setting.write_text("unknown\n")
        self.invoke(status=1)
        for args in (
            ("--save-build-host", "local", "--run"),
            ("--save-build-host", "local", "--features", "fast-build"),
            ("--build-host", "local", "--", "self-test"),
        ):
            self.invoke(*args, status=2)
        self.assertFalse(self.record.exists())
        self.assertEqual(setting.read_text(), "unknown\n")

    def test_missing_native_dependency_fails_before_execution(self):
        (self.common / "native_build_hosts.py").unlink()
        result = self.invoke("--build-host", "local", "--run", status=1)
        self.assertIn("native_build_hosts.py", result.stderr)
        self.assertFalse(self.record.exists())
        self.assertFalse(self.runtime.exists())

    def test_invalid_compile_inputs_fail_before_execution(self):
        for reference in ("../data/missing.csv", ".env"):
            self.put("src/new.rs", f'const INPUT: &str = include_str!("{reference}");')
            self.invoke("--build-host", "desktop", status=1)
            self.assertFalse(self.record.exists())
        (self.root / "src/new.rs").unlink()
        (self.root / "src/link.rs").symlink_to(self.root / "src/lib.rs")
        self.invoke("--build-host", "desktop", status=1)
        self.assertFalse(self.record.exists())


if __name__ == "__main__":
    unittest.main()
