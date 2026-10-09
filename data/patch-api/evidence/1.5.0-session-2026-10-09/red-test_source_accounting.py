"""Targeted frozen redirect accounting controls; no runtime/native acceptance."""

import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("audit150", EVIDENCE / "audit.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class FrozenRedirect(unittest.TestCase):
    def test_literal_redirect_and_empty_publication_inventory(self):
        ledger = audit.build()
        self.assertEqual(
            ledger["source_rows"],
            [
                {
                    "id": "raw-1.5.0-001",
                    "line": 1,
                    "literal": "#REDIRECT [[API change summaries/Historical]]",
                    "status": "metadata-only",
                    "capabilities": [],
                }
            ],
        )
        self.assertEqual(
            ledger["references"][0]["literal"], "[[API change summaries/Historical]]"
        )
        self.assertEqual(
            ledger["references"][0]["target"], "API change summaries/Historical"
        )
        self.assertEqual(ledger["references"][0]["status"], "UNPROVEN")
        for name in [
            "inventory",
            "signatures",
            "defaults",
            "prose",
            "headers",
            "templates",
        ]:
            self.assertEqual(ledger[name], [], name)
        self.assertEqual(
            ledger["measurements"], {"model": 0, "runtime": 0, "native": 0}
        )
        self.assertEqual(ledger["totals"]["physical_lines"], 1)
        self.assertEqual(ledger["totals"]["references"], 1)

    def test_exact_source_pin_response_registry(self):
        raw, pin, registry = audit.validate_source()
        self.assertEqual(raw, b"#REDIRECT [[API change summaries/Historical]]")
        self.assertEqual((pin["pageid"], pin["revid"]), (372076, 3587158))
        self.assertEqual(len(registry), 101)
        self.assertEqual(registry[-1]["version"], "1.0.0")
        for changed in [raw + b"\n", raw.replace(b"Historical", b"Classic")]:
            with self.assertRaises(AssertionError):
                audit.validate_source(raw=changed)
        response = json.loads((EVIDENCE / "source-response.json").read_bytes())
        response["query"]["pages"]["372076"]["revisions"][0]["revid"] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(response=json.dumps(response).encode())

    def test_every_boundary_and_derived_count_is_required(self):
        ledger = audit.build()
        for name in ["source_rows", "references", "contracts"]:
            self.assertTrue(ledger.get(name), f"missing boundary: {name}")
            changed = copy.deepcopy(ledger)
            changed[name].pop()
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)
        for name in ledger["totals"]:
            changed = copy.deepcopy(ledger)
            changed["totals"][name] += 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)
        for name in [
            "inventory",
            "signatures",
            "defaults",
            "prose",
            "headers",
            "templates",
        ]:
            changed = copy.deepcopy(ledger)
            changed[name] = [{"symbol": "InventedNativeAPI"}]
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)
        for name in ledger["measurements"]:
            changed = copy.deepcopy(ledger)
            changed["measurements"][name] = 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["references"][0]["expanded"] = True
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed)

    def test_foreign_histories_never_become_successors(self):
        ledger = audit.build()
        refs = ledger["history"]["separate_retail_successor_references"]
        self.assertIn("2.0.1", [row["version"] for row in refs])
        self.assertTrue(all(audit.is_retail_successor(row["version"]) for row in refs))
        for version in [
            "1.13.2",
            "1.15.9",
            "1.60.1",
            "2.5.6",
            "3.4.3",
            "4.4.2",
            "5.5.4",
        ]:
            self.assertFalse(audit.is_retail_successor(version), version)
        self.assertEqual(ledger["history"]["applied_successors"], [])
        self.assertEqual(
            ledger["history"]["queued_160"],
            "Original frozen 1.6.0/1.7.0 queued identities remain separate and unapplied; newer integrated 1.6.0 at base is methodology only",
        )

    def test_manifest_registry_pin_mutations_reject_in_copied_inputs(self):
        with tempfile.TemporaryDirectory(dir=EVIDENCE) as directory:
            copied = Path(directory)
            for name in ["source.wikitext", "source-response.json", "source-pin.json", "frozen-manifest.json", "frozen-registry.json"]:
                shutil.copyfile(EVIDENCE / name, copied / name)
            previous = audit.EVIDENCE
            try:
                audit.EVIDENCE = copied
                audit.validate_source()
                for name in ["source-pin.json", "frozen-manifest.json", "frozen-registry.json"]:
                    path = copied / name
                    original = path.read_bytes()
                    path.write_bytes(original + b" ")
                    if name == "source-pin.json":
                        changed = json.loads(original)
                        changed["revid"] += 1
                        path.write_text(json.dumps(changed))
                    try:
                        with self.assertRaises(AssertionError, msg=name):
                            audit.validate_source()
                    finally:
                        path.write_bytes(original)
                    audit.validate_source()
                    self.assertEqual(path.read_bytes(), original)
            finally:
                audit.EVIDENCE = previous

    def test_historical_default_tools_reproduce_exact_bytes(self):
        audit.replay_defaults()


if __name__ == "__main__":
    unittest.main(verbosity=2)
