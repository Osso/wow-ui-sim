"""Behavioral historical-validator fixtures; tamper only disposable fresh roots."""

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
HERE = ROOT / "data/patch-api/evidence/4.0.1-session-2026-10-09"
SOURCE = Path("data/patch-api/sources/4.0.1-api-changes.wikitext")


class HistoricalValidatorTests(unittest.TestCase):
    def run_validator(self, evidence):
        root = evidence.parents[3]
        self.assertFalse((root / "target").exists())
        result = subprocess.run(
            [
                sys.executable,
                "-B",
                str(evidence / "validate.py"),
                "--evidence",
                str(evidence),
            ],
            cwd=root,
            capture_output=True,
            text=True,
            env={"PATH": "/p401-no-git"},
        )
        self.assertFalse((root / "target").exists())
        return result

    def copied_evidence(self, directory):
        context = json.loads((HERE / "historical-context.json").read_text())
        names = (
            set(context["evidence_sha256"])
            | set(context["archives"])
            | {"historical-context.json", "validate.py"}
        )
        root = Path(directory) / "fresh-root"
        copy = root / HERE.relative_to(ROOT)
        copy.mkdir(parents=True)
        for name in names:
            shutil.copyfile(HERE / name, copy / name)
        source = root / SOURCE
        source.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / SOURCE, source)
        return copy

    def test_sealed_recorded_scope_validates_in_fresh_root_without_target_or_git(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_validator(self.copied_evidence(directory))
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout.split("PASS: ", 1)[1])
        self.assertEqual(summary["inventory"], 419)
        self.assertEqual(summary["publication_gaps"], 117)
        self.assertEqual(summary["statuses"]["bounded-coverage"], 302)

    def test_source_response_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / "source-response.json"
            path.write_bytes(path.read_bytes() + b" ")
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("evidence seal: source-response.json", result.stderr)

    def test_own_green_log_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / "retail-prefork-green.log"
            path.write_bytes(path.read_bytes() + b"\ntampered\n")
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("evidence seal: retail-prefork-green.log", result.stderr)

    def test_historical_archive_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / "historical-inputs.json.gz"
            path.write_bytes(path.read_bytes() + b"bad")
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("archive seal: historical-inputs.json.gz", result.stderr)

    def test_unrelated_later_inventory_does_not_change_recorded_scope(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = self.copied_evidence(directory)
            (evidence / "99.0.0-wikitext-register.json").write_text('{"entries":[]}')
            result = self.run_validator(evidence)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
