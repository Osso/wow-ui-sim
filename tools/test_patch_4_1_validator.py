"""Own historical evidence replay and restored source/log tamper controls."""

import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = Path("data/patch-api/evidence/4.1.0-session-2026-10-09")


class OwnValidatorTests(unittest.TestCase):
    def test_clean_replay_rejects_source_and_own_log_tampering(self):
        script = ROOT / EVIDENCE / "validate.py"
        self.assertTrue(script.exists(), "own historical validator is not implemented")
        spec = importlib.util.spec_from_file_location("p410_validator", script)
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        manifest = json.loads((ROOT / EVIDENCE / "historical-inputs.json").read_text())
        with tempfile.TemporaryDirectory(dir=ROOT / "tools") as directory:
            root = Path(directory)
            here = root / EVIDENCE
            paths = list(manifest["sealed_files"]) + [
                str(EVIDENCE / "historical-inputs.json"),
                str(EVIDENCE / "historical-blobs.json.gz"),
                str(EVIDENCE / "historical-page-coverage.json"),
                str(EVIDENCE / "historical-known-gaps.json"),
            ]
            for relative in paths:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, destination)
            clean = validator.validate(root, here)
            self.assertEqual(clean["publication_rows"], 81)
            self.assertEqual(clean["publication_ok"], 50)
            self.assertEqual(clean["publication_gaps"], 31)
            self.assertEqual(clean["negative_gaps"], 32)
            self.assertEqual(
                clean["publication_rows"],
                len(
                    json.loads(
                        (
                            root / "data/patch-api/sources/4.1.0-wikitext-register.json"
                        ).read_text()
                    )["entries"]
                ),
            )
            ledger_path = root / "data/patch-api/sources/4.1.0-page-coverage.json"
            fixture_path = root / "tests/data/patch_4_1_0_sweep_known_gaps.json"
            ledger = json.loads((here / "historical-page-coverage.json").read_text())
            fixture = json.loads((here / "historical-known-gaps.json").read_text())
            self.assertIn("wt-global-api-IsIPv6Available-73", fixture)
            closure = "wt-global-api-IsIPv6Available-73"
            for row in ledger["source_rows"]:
                if row["source_id"] == closure:
                    row.update(
                        status="bounded-coverage",
                        capabilities=["publication-absence"],
                        note="Removed in 4.3.0; current successor closure only.",
                    )
            fixture = [source_id for source_id in fixture if source_id != closure]
            self.assertEqual(len(fixture), 30)
            ledger_path.write_text(json.dumps(ledger, indent=2) + "\n")
            fixture_path.write_text(json.dumps(fixture, indent=2) + "\n")
            replay = json.loads(json.dumps(validator.validate(root, here)))
            self.assertEqual(replay, json.loads(json.dumps(clean)))
            for relative in [
                "data/patch-api/sources/4.1.0-api-changes.wikitext",
                str(EVIDENCE / "own-sweep-green.log"),
            ]:
                target = root / relative
                original = target.read_bytes()
                try:
                    target.write_bytes(original + b"\nTAMPER\n")
                    with self.assertRaisesRegex(
                        AssertionError, "sealed input: " + relative
                    ):
                        validator.validate(root, here)
                finally:
                    target.write_bytes(original)
                self.assertEqual(validator.validate(root, here), clean)
            for logical, name in [
                (
                    "data/patch-api/sources/4.1.0-page-coverage.json",
                    "historical-page-coverage.json",
                ),
                (
                    "tests/data/patch_4_1_0_sweep_known_gaps.json",
                    "historical-known-gaps.json",
                ),
            ]:
                target = here / name
                original = target.read_bytes()
                with self.subTest(historical_input=name):
                    try:
                        target.write_bytes(original + b"\nTAMPER\n")
                        with self.assertRaisesRegex(
                            AssertionError, "sealed input: " + logical
                        ):
                            validator.validate(root, here)
                        target.unlink()
                        with self.assertRaises(FileNotFoundError):
                            validator.validate(root, here)
                    finally:
                        target.write_bytes(original)
                    self.assertEqual(validator.validate(root, here), clean)


if __name__ == "__main__":
    unittest.main()
