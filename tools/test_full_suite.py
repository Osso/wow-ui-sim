"""Bounded subprocess coverage of the detached full-suite worker."""

import fcntl
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest


RUNNER = Path(__file__).with_name("full_suite.py")


def write_executable(path, body):
    path.write_text(f"#!{sys.executable}\n" + body)
    path.chmod(0o755)


def wait_for(path, process):
    deadline = time.monotonic() + 5
    while not path.exists():
        if process.poll() is not None:
            raise AssertionError(
                f"Worker exited before {path.name}: {process.communicate()}"
            )
        if time.monotonic() >= deadline:
            raise AssertionError(f"Timed out waiting for {path.name}")
        time.sleep(0.01)


class FullSuiteWorkerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.checkout = self.root / "Projects/wow/full-suite-checkout"
        (self.checkout / ".git").mkdir(parents=True)
        (self.root / "Projects/wow/wow-ui-sim").mkdir()
        self.wrapper = self.bin / "build-lock"
        self.lock_path = self.root / "builder.lock"
        write_executable(
            self.bin / "git",
            "import sys\nif sys.argv[1] == 'rev-parse': print('sandbox-sha')\n",
        )
        write_executable(
            self.wrapper,
            "import fcntl, os, pathlib, subprocess, sys\n"
            "root = pathlib.Path(os.environ['HOME'])\n"
            "with (root / 'builder.lock').open('w') as lock:\n"
            "    (root / 'waiting').touch()\n"
            "    fcntl.flock(lock, fcntl.LOCK_EX)\n"
            "    sys.exit(subprocess.call(sys.argv[1:]))\n",
        )
        write_executable(
            self.bin / "cargo",
            "import json, os, pathlib, sys, time\n"
            "root = pathlib.Path(os.environ['HOME'])\n"
            "name = 'lib' if '--lib' in sys.argv else sys.argv[sys.argv.index('--test') + 1]\n"
            "(root / (name + '.started')).write_text(json.dumps({\n"
            "    'argv': sys.argv[1:], 'jobs': os.environ.get('CARGO_BUILD_JOBS')}))\n"
            "deadline = time.monotonic() + 5\n"
            "while not (root / (name + '.release')).exists():\n"
            "    if time.monotonic() > deadline: sys.exit(98)\n"
            "    time.sleep(0.01)\n"
            "if name == 'prefork_full_ui' and os.environ['FAIL_PREFORK'] == '1':\n"
            "    print('test sandbox::failure ... FAILED')\n"
            "    sys.exit(7)\n",
        )
        self.harness = self.root / "worker.py"
        self.harness.write_text(
            "import pathlib, runpy, sys\n"
            f"worker = runpy.run_path({str(RUNNER)!r})\n"
            # Redirect only the installed dependency path; run every real worker function.
            "worker['run'].__globals__['BUILD_LOCK'] = pathlib.Path(sys.argv[1])\n"
            "worker['run']('sandbox-ref')\n"
        )
        self.env = dict(
            os.environ,
            HOME=str(self.root),
            PATH=f"{self.bin}{os.pathsep}{os.environ['PATH']}",
            FULL_SUITE_JOBS="3",
            CARGO_BUILD_JOBS="29",
            FAIL_PREFORK="0",
        )

    def start_worker(self):
        process = subprocess.Popen(
            [sys.executable, str(self.harness), str(self.wrapper)],
            env=self.env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        self.addCleanup(self.stop_worker, process)
        return process

    @staticmethod
    def stop_worker(process):
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
        process.communicate(timeout=5)

    def assert_lock_held(self, lock):
        with self.assertRaises(BlockingIOError):
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def exercise_worker(self, fail_prefork):
        self.env["FAIL_PREFORK"] = str(int(fail_prefork))
        with self.lock_path.open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            process = self.start_worker()
            # Both old and new workers have a deterministic observable first boundary.
            deadline = time.monotonic() + 5
            while (
                not (self.root / "waiting").exists()
                and not (self.root / "integration.started").exists()
            ):
                self.assertIsNone(process.poll())
                self.assertLess(time.monotonic(), deadline)
                time.sleep(0.01)
            time.sleep(0.05)
            self.assertFalse(
                (self.root / "integration.started").exists(),
                "Cargo bypassed shared lock",
            )
            fcntl.flock(lock, fcntl.LOCK_UN)
            for name in ("integration", "prefork_full_ui", "lib"):
                started = self.root / f"{name}.started"
                wait_for(started, process)
                self.assert_lock_held(lock)
                record = json.loads(started.read_text())
                self.assertIn("--offline", record["argv"])
                self.assertIn("--locked", record["argv"])
                self.assertEqual(record["jobs"], "4")
                if name != "prefork_full_ui":
                    index = record["argv"].index("--test-threads")
                    self.assertEqual(record["argv"][index + 1], "3")
                (self.root / f"{name}.release").touch()
            stdout, stderr = process.communicate(timeout=5)
            self.assertEqual(process.returncode, 0, stderr)
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        results = self.root / "Projects/wow/full-suite-results"
        result = json.loads((results / "sandbox-sha.json").read_text())
        self.assertEqual(result["steps"]["prefork"]["exit"], 7 if fail_prefork else 0)
        self.assertEqual(result["steps"]["lib"]["exit"], 0)
        self.assertEqual(
            result["failures"]["prefork"], ["sandbox::failure"] if fail_prefork else []
        )
        self.assertEqual(json.loads(stdout)["steps"], result["steps"])
        log = (results / "sandbox-sha.log").read_text()
        self.assertIn("exit 7" if fail_prefork else "exit 0", log)
        if fail_prefork:
            self.assertIn("test sandbox::failure ... FAILED", log)

    def test_shared_lock_covers_successful_children(self):
        self.exercise_worker(False)

    def test_shared_lock_releases_after_failure_and_retains_result(self):
        self.exercise_worker(True)

    def test_missing_shared_wrapper_fails_without_starting_cargo(self):
        self.wrapper.unlink()
        for name in ("integration", "prefork_full_ui", "lib"):
            (self.root / f"{name}.release").touch()
        process = self.start_worker()
        _, stderr = process.communicate(timeout=5)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("Required shared build wrapper missing", stderr)
        self.assertIn(str(self.wrapper), stderr)
        self.assertFalse((self.root / "integration.started").exists())


if __name__ == "__main__":
    unittest.main()
