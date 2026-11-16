"""End-to-end drift checks for the pinned generator and its committed artifact."""
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tools import gen_limited_listfile as generator


def identity(data):
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def selection(values):
    values = sorted(map(str, values))
    return {"count": len(values), "sha256": hashlib.sha256(("\n".join(values) + "\n").encode()).hexdigest()}


def fixture_pin(source, overrides, output, override_id):
    paths = {path.replace("\\", "/").lower() for path in generator.FONT_PATHS + generator.PROBE_PATHS}
    return {
        "schema": 1,
        "source": {"url": "https://example.invalid/pinned.csv", "release": "fixture", "fetched_at": "2026-11-16T00:00:00Z", **identity(source)},
        "extra_sources": [{"path": "data/listfile-overrides.csv", **identity(overrides)}],
        "requests": {
            "path_candidates": selection(paths),
            "file_data_ids": selection(set(generator.SOUNDKIT_FDIDS) | {override_id}),
            "blizzard_ui_files": selection([]),
        },
        "output": {"path": "data/wow-ui-sim-listfile.csv", "rows": 1, **identity(output)},
    }


class LimitedListfilePinnedInputTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "tools").mkdir()
        shutil.copyfile(generator.__file__, self.root / "tools/gen_limited_listfile.py")
        for name in ["crates/sim-data/data", "crates/sim-data-profile/data", "data", "src"]:
            (self.root / name).mkdir(parents=True, exist_ok=True)
        (self.root / "crates/sim-data/data/manifest_interface_data.rs").write_text("")
        (self.root / "crates/sim-data-profile/data/atlas.rs").write_text("")
        self.source = self.root / "community-listfile.csv"
        self.source.write_bytes(b"1;Interface/Icons/Unused.blp\n")
        self.overrides = self.root / "data/listfile-overrides.csv"
        self.overrides.write_bytes(b"2;Interface/Icons/Override.blp\n")
        self.output = self.root / "data/wow-ui-sim-listfile.csv"
        self.output.write_bytes(b"2;Interface/Icons/Override.blp\n")
        self.pin = self.root / "data/wow-ui-sim-listfile.provenance.json"
        self.pin.write_text(json.dumps(fixture_pin(self.source.read_bytes(), self.overrides.read_bytes(), self.output.read_bytes(), 2)))

    def run_generator(self, check=True):
        command = [sys.executable, str(self.root / "tools/gen_limited_listfile.py"), "--source", str(self.source)]
        if check:
            command.append("--check")
        return subprocess.run(command, capture_output=True, text=True, cwd=self.root)

    def test_check_accepts_pinned_inputs_without_rewriting_output(self):
        before = self.output.stat().st_mtime_ns
        result = self.run_generator()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(before, self.output.stat().st_mtime_ns)

    def test_source_drift_is_rejected_before_output_is_written(self):
        before = self.output.read_bytes()
        self.source.write_bytes(self.source.read_bytes() + b"3;Interface/Icons/New.blp\n")
        result = self.run_generator(check=False)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("source", result.stderr.lower())
        self.assertEqual(before, self.output.read_bytes())

    def test_extra_row_source_drift_is_rejected_before_writing(self):
        before = self.output.read_bytes()
        self.overrides.write_bytes(self.overrides.read_bytes().replace(b"Override", b"Changed"))
        result = self.run_generator(check=False)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("override", result.stderr.lower())
        self.assertEqual(before, self.output.read_bytes())

    def test_request_selection_drift_is_rejected_before_writing(self):
        before = self.output.read_bytes()
        (self.root / "src/new.rs").write_text('const ICON: &str = "Interface/Icons/New";')
        result = self.run_generator(check=False)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("request", result.stderr.lower())
        self.assertEqual(before, self.output.read_bytes())

    def test_output_drift_fails_check_and_is_not_overwritten(self):
        self.output.write_bytes(b"2;Interface/Icons/Wrong.blp\n")
        result = self.run_generator()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("output", result.stderr.lower())
        self.assertEqual(b"2;Interface/Icons/Wrong.blp\n", self.output.read_bytes())


class CommittedLimitedListfileTests(unittest.TestCase):
    def test_committed_output_matches_pinned_digest(self):
        pin = json.loads((generator.ROOT / "data/wow-ui-sim-listfile.provenance.json").read_text())
        data = generator.DEFAULT_OUTPUT.read_bytes()
        self.assertEqual(pin["output"]["sha256"], hashlib.sha256(data).hexdigest())
        self.assertEqual(pin["output"]["rows"], len(data.splitlines()))

    def test_full_generation_matches_the_committed_artifact(self):
        source = Path(os.environ.get("WOW_UI_SIM_LISTFILE_SOURCE", str(generator.DEFAULT_SOURCE)))
        self.assertTrue(source.is_file(), "Install the pinned source described by data/wow-ui-sim-listfile.provenance.json first")
        result = subprocess.run([sys.executable, generator.__file__, "--source", str(source), "--check"], capture_output=True, text=True)
        self.assertEqual(0, result.returncode, result.stderr)


if __name__ == "__main__":
    unittest.main()
