"""Process proof for profile launch routing; no simulator or build is started."""

import json
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "start-profile.sh"


class StartProfileTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "scripts").mkdir()
        self.script = self.root / "scripts/start-profile.sh"
        self.script.write_bytes(SCRIPT.read_bytes())
        self.script.chmod(0o755)
        helper = self.root / "scripts/build-host.py"
        helper.write_text(
            'import json,os,pathlib,sys\nwith pathlib.Path(os.environ["PROFILE_RECORD"]).open("a") as f:f.write(json.dumps({"args":sys.argv[1:],"cwd":os.getcwd()})+"\\n")\nsys.exit(int(os.environ.get("FAKE_STATUS","0")))\n'
        )
        self.record = self.root / "record.jsonl"
        self.env = dict(os.environ, PROFILE_RECORD=str(self.record))

    def run_profile(self, *args, **env):
        return subprocess.run(
            [str(self.script), *args],
            env=self.env | env,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

    def records(self, count=1):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            rows = (
                [json.loads(line) for line in self.record.read_text().splitlines()]
                if self.record.exists()
                else []
            )
            if len(rows) >= count:
                return rows
            time.sleep(0.05)
        self.fail("profile helper did not run")

    def test_foreground_routes_profile_host_runtime_and_exit(self):
        result = self.run_profile(
            "--build-host",
            "desktop",
            "ptr",
            "--no-saved-vars",
            "lua-errors",
            WOW_SIM_START_FOREGROUND="1",
            FAKE_STATUS="7",
        )
        self.assertEqual(result.returncode, 7, result.stderr)
        row = self.records()[0]
        self.assertEqual(row["cwd"], str(self.root))
        self.assertEqual(
            row["args"],
            [
                "--build-host",
                "desktop",
                "--no-default-features",
                "--features",
                "sound,gui,casc,client-ptr",
                "--run",
                "--",
                "--no-saved-vars",
                "lua-errors",
            ],
        )

    def test_release_no_build_and_separator_preserved(self):
        result = self.run_profile(
            "retail",
            "--",
            "--build-host",
            "runtime-owned",
            WOW_SIM_START_FOREGROUND="1",
            WOW_SIM_START_RELEASE="1",
            WOW_SIM_START_NO_BUILD="1",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        args = self.records()[0]["args"]
        self.assertIn("--release", args)
        self.assertIn("--no-build", args)
        self.assertEqual(
            args[args.index("--") + 1 :], ["--build-host", "runtime-owned"]
        )

    def test_all_background_routes_three_profiles_and_writes_logs(self):
        result = self.run_profile("all", "lua-errors")
        self.assertEqual(result.returncode, 0, result.stderr)
        rows = self.records(3)
        features = {row["args"][row["args"].index("--features") + 1] for row in rows}
        self.assertEqual(
            features,
            {
                "sound,gui,casc,client-retail",
                "sound,gui,casc,client-ptr",
                "sound,gui,casc,client-mists",
            },
        )
        for label in ("live", "ptr", "mists"):
            self.assertTrue(
                (self.root / "target/profile-runs" / (label + ".pid"))
                .read_text()
                .strip()
                .isdigit()
            )
            self.assertTrue(
                (self.root / "target/profile-runs" / (label + ".log")).is_file()
            )

    def test_all_foreground_fails_before_launch(self):
        result = self.run_profile("all", WOW_SIM_START_FOREGROUND="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.record.exists())
