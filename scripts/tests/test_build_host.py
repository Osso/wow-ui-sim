"""Native adapter process fixtures; no Cargo, remote hosts, or GUI execution."""

import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

HELPER = Path(__file__).resolve().parents[1] / "build-host.py"
COMMON = Path("/syncthing/Sync/Projects/world-of-osso/game-engine/scripts")

FAKE_MODULE = """import json, os, subprocess, sys
from pathlib import Path

def execute(context, checkout_key, project_name, cargo_args, host,
            runtime_args=None, binary=None, release=False, environment=None, root=None, build=True):
    request = dict(context=str(context), key=checkout_key, project=project_name,
                   cargo_args=cargo_args, host=host, runtime_args=runtime_args,
                   binary=binary, release=release, environment=environment, root=str(root), build=build)
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
if request["cargo_args"][0] == "test":
    print("running 1 test\\ntest addon_filter ... FAILED", flush=True)
status = int(os.environ.get("FAKE_BUILD_STATUS", "0"))
if status:
    sys.exit(status)
print("native artifact=/worker/target/" + ("release" if request["release"] else "debug") + "/" + request["binary"], flush=True)
if request["runtime_args"] is not None:
    Path(os.environ["FAKE_RUNTIME"]).write_text(json.dumps(dict(args=request["runtime_args"], root=request["root"], host=request["host"])))
    print("native runtime", flush=True)
    sys.exit(int(os.environ.get("FAKE_RUN_STATUS", "0")))
"""


CONCURRENT_NATIVE = '''import json, os, subprocess, sys
from pathlib import Path

CHILD = """
import json, os, sys, time
from pathlib import Path
context, marker, release = map(Path, sys.argv[1:])
marker.write_text(json.dumps(dict(context=str(context), pid=os.getpid())))
while not release.exists():
    time.sleep(0.02)
"""

def execute(context, checkout_key, project_name, cargo_args, host, **kwargs):
    marker = Path(os.environ['CONCURRENT_MARKER'])
    if kwargs.get('runtime_args') is None:
        marker.write_text(json.dumps(dict(context=str(context))))
        return 0
    return subprocess.run([sys.executable, '-c', CHILD, str(context),
                           str(marker), os.environ['CONCURRENT_RELEASE']]).returncode
'''


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

    def test_running_app_does_not_block_check_or_delete_another_context(self):
        (self.common / "native_build_hosts.py").write_text(CONCURRENT_NATIVE)
        for host in ("desktop", "local"):
            with self.subTest(host=host):
                self.exercise_concurrent_helpers(host)

    def exercise_concurrent_helpers(self, host):
        command = [
            sys.executable,
            str(HELPER),
            "--root",
            str(self.root),
            "--build-host",
            host,
        ]
        processes = []
        releases = []
        try:
            first, first_marker, first_release = self.start_running_helper(
                command, host + "-first", processes, releases
            )
            first_request = self.wait_for_runtime(first, first_marker)
            check_marker = self.base / (host + "-check.json")
            try:
                check = subprocess.run(
                    command + ["--check"],
                    env=dict(self.env, CONCURRENT_MARKER=str(check_marker)),
                    capture_output=True,
                    text=True,
                    timeout=3,
                )
            except subprocess.TimeoutExpired:
                self.fail("check blocked behind a running app's caller lock")
            self.assertEqual(check.returncode, 0, check.stdout + check.stderr)
            self.assertIsNone(first.poll())
            os.kill(first_request["pid"], 0)
            second, second_marker, _ = self.start_running_helper(
                command, host + "-second", processes, releases
            )
            second_request = self.wait_for_runtime(second, second_marker)
            first_context = Path(first_request["context"])
            second_context = Path(second_request["context"])
            self.assertNotEqual(first_context, second_context)
            sentinel = second_context / "runtime-sentinel"
            sentinel.write_bytes(b"second context still owned")
            first_release.touch()
            stdout, stderr = first.communicate(timeout=3)
            self.assertEqual(first.returncode, 0, stdout + stderr)
            self.assertFalse(first_context.exists())
            self.assertEqual(sentinel.read_bytes(), b"second context still owned")
            self.assertIsNone(second.poll())
            os.kill(second_request["pid"], 0)
        finally:
            for release in releases:
                release.touch()
            for process in processes:
                try:
                    process.communicate(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.communicate(timeout=3)

    def start_running_helper(self, command, name, processes, releases):
        marker = self.base / (name + ".json")
        release = self.base / (name + ".release")
        releases.append(release)
        process = subprocess.Popen(
            command + ["--run"],
            env=dict(
                self.env, CONCURRENT_MARKER=str(marker), CONCURRENT_RELEASE=str(release)
            ),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        processes.append(process)
        return process, marker, release

    def wait_for_runtime(self, process, marker):
        deadline = time.monotonic() + 3
        while not marker.exists():
            if process.poll() is not None:
                stdout, stderr = process.communicate()
                self.fail("helper exited before runtime: " + stdout + stderr)
            if time.monotonic() >= deadline:
                self.fail("runtime did not start within bounded timeout")
            time.sleep(0.02)
        return json.loads(marker.read_text())

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

    def test_no_build_requires_run_and_rejects_other_modes(self):
        for args in (
            ("--no-build",),
            ("--no-build", "--check"),
            ("--no-build", "--test"),
            ("--no-build", "--save-build-host", "desktop"),
        ):
            with self.subTest(args=args):
                result = self.invoke(*args, status=2)
                self.assertIn("--no-build", result.stderr)
                self.assertFalse(self.record.exists())
        self.assertFalse(
            (Path(self.env["HOME"]) / ".config/game-engine/build-host").exists()
        )

    def test_no_build_desktop_request_preserves_runtime_snapshot(self):
        self.invoke("--build-host", "desktop", "--no-build", "--run", "--", "two words")
        self.assertFalse(self.request()["build"])
        self.assertEqual(self.request()["runtime_args"], ["two words"])
        self.assertEqual(
            (self.output / "wow-ui-sim/Interface/AddOns/Admin/new.lua").read_text(),
            "working untracked addon Lua",
        )

    def test_no_build_real_native_existing_and_missing_profiles_without_cargo(self):
        for name in ("native_build_hosts.py", "build_hosts.py"):
            shutil.copy2(COMMON / name, self.common / name)
        spec = importlib.util.spec_from_file_location(
            "native_fixture", COMMON / "tests/test_native_build_hosts.py"
        )
        fixture = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(fixture)
        cargo_bin = Path(self.env["HOME"]) / ".cargo/bin"
        cargo_bin.mkdir(parents=True)
        rustup = cargo_bin / "rustup"
        rustup.write_text(fixture.RUSTUP)
        rustup.chmod(0o755)
        wrapper = self.common / "agent/agent-run"
        wrapper.parent.mkdir()
        wrapper.write_text(
            "#!/usr/bin/env python3\nimport os, sys\nassert sys.argv[1] == 'native-build'\nos.execvp(sys.argv[2], sys.argv[2:])\n"
        )
        wrapper.chmod(0o755)
        self.env["FIXTURE_ROOT"] = str(self.base)
        for release in (False, True):
            profile = "release" if release else "debug"
            flags = ["--release"] if release else []
            app = self.put(
                f"target/{profile}/wow-sim",
                "#!/usr/bin/env python3\nimport json, os, pathlib, sys\n"
                "pathlib.Path(os.environ['FAKE_RUNTIME']).write_text(json.dumps(dict(args=sys.argv[1:], loader=os.environ['LD_LIBRARY_PATH'], cwd=os.getcwd())))\n"
                "sys.exit(23)\n",
            )
            app.chmod(0o755)
            self.invoke(
                "--build-host",
                "local",
                *flags,
                "--no-build",
                "--run",
                "--",
                "two words",
                status=23,
            )
            report = json.loads(self.runtime.read_text())
            self.assertEqual(report["args"], ["two words"])
            self.assertEqual(report["cwd"], str(self.root))
            self.assertIn(str(self.base / "lib"), report["loader"].split(":"))
            self.assertIn(
                str(self.root / f"target/{profile}/deps"), report["loader"].split(":")
            )
            self.assertFalse((self.base / "cargo.log").exists())
            app.unlink()
            self.runtime.unlink()
            result = self.invoke(
                "--build-host", "local", *flags, "--no-build", "--run", status=1
            )
            self.assertIn(str(app), result.stderr)
            self.assertFalse(self.runtime.exists())
            self.assertFalse((self.base / "cargo.log").exists())

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

    def test_native_test_preserves_cargo_separator_features_and_failure(self):
        self.put(
            "tests/fixture.rs",
            'const DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/test.json"));',
        )
        self.put("data/test.json", '{"fixture": true}')
        for host in ("desktop", "local"):
            with self.subTest(host=host):
                self.env["FAKE_BUILD_STATUS"] = "101"
                result = self.invoke(
                    "--build-host",
                    host,
                    "--release",
                    "--no-default-features",
                    "--features",
                    "client-mists",
                    "--test",
                    "--test",
                    "test_addon",
                    "addon_filter",
                    "--",
                    "--exact",
                    "--nocapture",
                    status=101,
                )
                request = self.request()
                self.assertEqual(
                    request["cargo_args"],
                    [
                        "test",
                        "--no-default-features",
                        "--features",
                        "client-mists",
                        "--test",
                        "test_addon",
                        "addon_filter",
                        "--",
                        "--exact",
                        "--nocapture",
                    ],
                )
                self.assertEqual(request["host"], host)
                self.assertTrue(request["release"])
                self.assertIsNone(request["runtime_args"])
                self.assertFalse(self.runtime.exists())
                self.assertIn("native build " + host, result.stdout)
                self.assertIn(
                    "running 1 test\ntest addon_filter ... FAILED", result.stdout
                )
                self.assertEqual(
                    (self.output / "wow-ui-sim/data/test.json").read_bytes(),
                    (self.root / "data/test.json").read_bytes(),
                )

    def test_native_test_defaults_do_not_select_one_binary(self):
        self.invoke("--build-host", "desktop", "--test")
        self.assertEqual(self.request()["cargo_args"], ["test"])
        self.assertIsNone(self.request()["runtime_args"])

    def test_native_check_selects_binary_and_features_without_running(self):
        self.invoke("--build-host", "desktop", "--check")
        self.assertEqual(self.request()["cargo_args"], ["check", "--bin", "wow-sim"])
        self.env["FAKE_BUILD_STATUS"] = "101"
        self.invoke(
            "--build-host",
            "local",
            "--bin",
            "wow-cli",
            "--no-default-features",
            "--features",
            "client-era",
            "--check",
            status=101,
        )
        self.assertEqual(
            self.request()["cargo_args"],
            [
                "check",
                "--bin",
                "wow-cli",
                "--no-default-features",
                "--features",
                "client-era",
            ],
        )
        self.assertIsNone(self.request()["runtime_args"])
        self.assertFalse(self.runtime.exists())

    def test_build_modes_and_save_only_conflicts_do_not_execute(self):
        for args in (
            ("--run", "--test"),
            ("--run", "--check"),
            ("--check", "--test"),
            ("--save-build-host", "desktop", "--test"),
            ("--save-build-host", "desktop", "--check"),
            ("--check", "--", "addon_filter"),
        ):
            with self.subTest(args=args):
                self.invoke("--build-host", "desktop", *args, status=2)
                self.assertFalse(self.record.exists())
                self.assertFalse(self.runtime.exists())
        self.invoke("--save-build-host", "desktop", "--test", status=2)
        self.invoke("--save-build-host", "desktop", "--check", status=2)

    def test_root_addon_fixtures_snapshot_owned_working_data_only(self):
        self.put("test_addons/TestAddon/TestAddon.lua", "tracked Lua")
        self.put("test_addons/TestAddon/TestAddon.toc", "TestAddon.lua")
        self.put("test_addons/TestAddon/layout.xml", "<Ui/>")
        self.put("test_addons/TestAddon/private.pem", "SECRET")
        self.git("add", "test_addons")
        self.put("test_addons/TestAddon/TestAddon.lua", "modified working Lua")
        for name in ("new.lua", "new.xml", "new.toc", "nested/test.lua"):
            self.put("test_addons/TestAddon/" + name, "working fixture " + name)
        for name in (
            "SavedVariables/state.lua",
            "cache/result.lua",
            ".git/config.lua",
            "private.key",
            ".env",
            "unknown.bin",
        ):
            self.put("test_addons/TestAddon/" + name, "SECRET")
        self.put("test_addons/Private/private.lua", "unowned data")
        external = self.put("private-outside.lua", "SECRET")
        (self.root / "test_addons/TestAddon/link.lua").symlink_to(external)
        self.invoke("--build-host", "desktop")
        for name in (
            "TestAddon.lua",
            "TestAddon.toc",
            "layout.xml",
            "new.lua",
            "new.xml",
            "new.toc",
            "nested/test.lua",
        ):
            relative = Path("test_addons/TestAddon") / name
            self.assertEqual(
                (self.output / "wow-ui-sim" / relative).read_bytes(),
                (self.root / relative).read_bytes(),
            )
        for name in (
            "SavedVariables/state.lua",
            "cache/result.lua",
            ".git/config.lua",
            "private.key",
            "private.pem",
            ".env",
            "unknown.bin",
            "link.lua",
        ):
            self.assertFalse(
                (self.output / "wow-ui-sim/test_addons/TestAddon" / name).exists(), name
            )
        self.assertFalse((self.output / "wow-ui-sim/test_addons/Private").exists())

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
