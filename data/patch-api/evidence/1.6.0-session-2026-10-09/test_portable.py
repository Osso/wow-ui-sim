"""Portable historical replay controls: standard library, copied evidence only."""

import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

EVIDENCE = Path(__file__).resolve().parent
RECEIPTS = []


class PortableReplay(unittest.TestCase):
    def setUp(self):
        self.assertTrue(
            (EVIDENCE / "seals.json").is_file(), "original immutable seal map absent"
        )
        self.original_map = (EVIDENCE / "seals.json").read_bytes()
        self.seals = json.loads(self.original_map)
        self.temp = tempfile.TemporaryDirectory(dir=EVIDENCE)
        self.addCleanup(self.temp.cleanup)
        self.copied = Path(self.temp.name)
        for name in self.seals:
            self.assertFalse(Path(name).is_absolute() or ".." in Path(name).parts)
            self.assertFalse(
                set(Path(name).parts) & {".git", "target", "__pycache__", "Interface"}
            )
            destination = self.copied / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(EVIDENCE / name, destination)
        (self.copied / "seals.json").write_bytes(self.original_map)
        spec = importlib.util.spec_from_file_location(
            "copied160", self.copied / "audit.py"
        )
        self.audit = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.audit)

    def validate(self):
        count = self.audit.check_seals()
        totals = self.audit.validate_ledger(self.audit.read_json("ledger.json"))
        self.audit.replay_defaults()
        return {"seals": count, "totals": totals}

    def test_copied_historical_replay(self):
        result = self.validate()
        self.assertEqual(result["seals"], len(self.seals))
        self.assertEqual(result["totals"]["references"], 1)
        self.assertEqual(result["totals"]["inventory"], 0)
        RECEIPTS.append(
            {
                "control": "copied-historical-replay",
                "result": result,
                "original_seals_sha256": self.audit.digest(self.original_map),
            }
        )

    def reject_restore(self, name, changed):
        path = self.copied / name
        original = path.read_bytes()
        path.write_bytes(changed)
        try:
            with self.assertRaisesRegex(AssertionError, f"seal: {name}"):
                self.validate()
            RECEIPTS.append(
                {
                    "control": "serialized-tamper-rejected",
                    "file": name,
                    "original_sha256": self.audit.digest(original),
                    "tampered_sha256": self.audit.digest(changed),
                }
            )
        finally:
            path.write_bytes(original)
        self.assertEqual(path.read_bytes(), original)
        self.assertEqual(self.audit.digest(path.read_bytes()), self.seals[name])
        self.assertEqual((self.copied / "seals.json").read_bytes(), self.original_map)
        self.validate()
        RECEIPTS.append(
            {
                "control": "exact-restoration",
                "file": name,
                "restored_sha256": self.audit.digest(path.read_bytes()),
                "original_seals_sha256": self.audit.digest(self.original_map),
            }
        )

    def test_serialized_ledger_omission_rejected_restored(self):
        changed = json.loads((self.copied / "ledger.json").read_bytes())
        changed["references"].pop()
        self.reject_restore(
            "ledger.json", (json.dumps(changed, indent=2) + "\n").encode()
        )

    def test_serialized_log_fabrication_rejected_restored(self):
        original = (self.copied / "green.log").read_bytes()
        self.reject_restore(
            "green.log", original + b"fabricated native 1.6.0 parity PASS\n"
        )


if __name__ == "__main__":
    outcome = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(PortableReplay)
    )
    import sys

    if len(sys.argv) == 2:
        destination = Path(sys.argv[1])
        assert not destination.exists(), "refuse receipt overwrite"
        destination.write_text(
            json.dumps(
                {
                    "scope": "portable SOURCE development controls",
                    "success": outcome.wasSuccessful(),
                    "tests": outcome.testsRun,
                    "receipts": RECEIPTS,
                },
                indent=2,
            )
            + "\n"
        )
    sys.exit(not outcome.wasSuccessful())
