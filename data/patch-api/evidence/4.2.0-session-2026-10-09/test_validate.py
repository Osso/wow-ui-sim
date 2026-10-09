"""Serialized historical-only controls; never loads the live repo or Git."""

import gzip
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

HERE = Path(__file__).resolve().parent


class HistoricalValidatorTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(
            (HERE / "validate.py").is_file(), "historical validator missing"
        )
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.evidence = Path(self.temp.name) / "evidence"
        self.evidence.mkdir()
        manifest = json.loads((HERE / "historical-inputs.json").read_text())
        for name in [
            *manifest["evidence"],
            "historical-inputs.json",
            "historical-inputs.tar.gz",
            "validate.py",
        ]:
            shutil.copyfile(HERE / name, self.evidence / name)

    def run_validator(self):
        return subprocess.run(
            [sys.executable, "-I", str(self.evidence / "validate.py")],
            cwd=self.temp.name,
            capture_output=True,
            text=True,
            timeout=30,
        )

    def reject_file(self, name):
        path = self.evidence / name
        if name.endswith(".json"):
            value = json.loads(path.read_text())
            if isinstance(value, dict):
                value["tampered"] = True
            else:
                value.append("tampered")
            path.write_text(json.dumps(value) + "\n")
        else:
            path.write_bytes(path.read_bytes() + b"\ntampered receipt\n")
        result = self.run_validator()
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(name, result.stderr)

    def reject_member(self, name):
        path = self.evidence / "historical-inputs.tar.gz"
        output = io.BytesIO()
        with tarfile.open(
            fileobj=io.BytesIO(gzip.decompress(path.read_bytes()))
        ) as old:
            with tarfile.open(fileobj=output, mode="w") as new:
                for info in old:
                    data = old.extractfile(info).read()
                    if info.name == name:
                        data += b"\n "
                    info.size = len(data)
                    new.addfile(info, io.BytesIO(data))
        path.write_bytes(gzip.compress(output.getvalue(), mtime=0))
        result = self.run_validator()
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("historical-inputs.tar.gz", result.stderr)

    def test_relocated_replay_without_git_or_current_inputs(self):
        result = self.run_validator()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["historical_gaps"], 31)
        self.assertEqual(report["negative_gaps"], 32)
        self.assertEqual(report["inventory_rows"], 65)
        self.assertEqual(report["native_runtime_replay"], False)
        self.assertEqual(report["source_replay"], "byte-identical")

    def test_response_tampering(self):
        self.reject_file("source-response.json")

    def test_page_identity_tampering(self):
        self.reject_file("source-pin.json")

    def test_receipt_log_tampering(self):
        self.reject_file("behavior-green.log")

    def test_result_tampering(self):
        self.reject_file("development-results.json")

    def test_negative_result_tampering(self):
        self.reject_file("negative-results.json")

    def test_receipt_manifest_tampering(self):
        self.reject_file("development-proof.json")

    def test_snapshot_manifest_tampering(self):
        self.reject_file("historical-inputs.json")

    def test_source_tampering(self):
        self.reject_member("data/patch-api/sources/4.2.0-api-changes.wikitext")

    def test_ledger_tampering(self):
        self.reject_member("data/patch-api/sources/4.2.0-page-coverage.json")

    def test_gap_fixture_tampering(self):
        self.reject_member("tests/data/patch_4_2_0_sweep_known_gaps.json")

    def test_generator_tampering(self):
        self.reject_member("tools/gen_patch_wikitext_register.py")

    def test_later_register_tampering(self):
        self.reject_member("data/patch-api/sources/5.0.1-wikitext-register.json")


if __name__ == "__main__":
    unittest.main()
