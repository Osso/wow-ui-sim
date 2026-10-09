"""Owned frozen 1.1.0 SOURCE controls, not runtime/native acceptance."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("audit110", EVIDENCE / "audit.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
SYMBOLS = [
    "CheckTalentMasterDist",
    "ConfirmSummon",
    "ConfirmTalentWipe",
    "GetAuctionItemLink",
    "GetCVarDefault",
    "GetResSicknessDuration",
    "GetSummonConfirmAreaName",
    "GetSummonConfirmSummoner",
    "GetSummonConfirmTimeLeft",
    "TutorialsEnabled",
    "UnitRangedAttack",
]


class FrozenSource(unittest.TestCase):
    def test_literal_rows_links_headers_navigation(self):
        ledger = audit.build()
        self.assertEqual(len(ledger["source_rows"]), 15)
        self.assertEqual(ledger["totals"]["physical_lines"], 17)
        self.assertEqual([x["symbol"] for x in ledger["inventory"]], SYMBOLS)
        self.assertEqual([x["line"] for x in ledger["inventory"]], list(range(7, 18)))
        self.assertTrue(all(x["direction"] == "added" for x in ledger["inventory"]))
        self.assertEqual(
            [x["title"] for x in ledger["headers"]], ["Global API", "Added"]
        )
        self.assertEqual(
            ledger["templates"][0]["literal"],
            "{{apichanges|1.1.0|prev=1.0.0|next=1.2.0}}",
        )
        self.assertEqual(len(ledger["references"]), 12)
        self.assertTrue(all(not x["expanded"] for x in ledger["references"]))
        self.assertEqual(ledger["prose"][0]["line"], 3)
        self.assertEqual(ledger["defaults"], [])
        self.assertEqual(ledger["count_claims"], [])

    def test_precise_unspecified_contracts(self):
        ledger = audit.build()
        self.assertEqual(len(ledger["signatures"]), 11)
        self.assertEqual(len(ledger["contracts"]), 13)
        for signature in ledger["signatures"]:
            self.assertIsNone(signature["arguments"])
            self.assertIsNone(signature["returns"])
            self.assertEqual(signature["status"], "UNPROVEN")
        for contract in ledger["contracts"]:
            self.assertEqual(contract["status"], "UNPROVEN")
            for field in [
                "arguments",
                "returns",
                "defaults",
                "events",
                "state_transitions",
                "security",
                "native_equivalence",
            ]:
                self.assertIsNone(contract[field], (contract["id"], field))
        self.assertEqual(
            ledger["measurements"], {"model": 0, "runtime": 0, "native": 0}
        )

    def test_identity_and_frozen_source_mutations(self):
        raw, pin, registry = audit.validate_source()
        self.assertEqual(
            (pin["pageid"], pin["revid"], pin["timestamp"]),
            (271516, 5913060, "2023-12-27T16:05:57Z"),
        )
        self.assertEqual(len(raw), 701)
        self.assertEqual(len(registry), 101)
        self.assertNotIn("1.2.0", [p["version"] for p in registry])
        for changed in [
            raw + b"\n",
            raw.replace(b"UnitRangedAttack", b"InventedAttack"),
        ]:
            with self.assertRaises(AssertionError):
                audit.validate_source(raw=changed)
        response = json.loads((EVIDENCE / "source-response.json").read_bytes())
        response["query"]["pages"]["271516"]["revisions"][0]["revid"] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(response=json.dumps(response).encode())
        for name in ["source-pin.json", "frozen-manifest.json", "frozen-registry.json"]:
            with self.assertRaises(AssertionError):
                audit.validate_source(
                    overrides={name: (EVIDENCE / name).read_bytes() + b" "}
                )

    def test_every_occurrence_count_and_identity_required(self):
        ledger = audit.build()
        omissions = 0
        for name in [
            "source_rows",
            "inventory",
            "signatures",
            "references",
            "contracts",
            "prose",
            "headers",
            "templates",
        ]:
            self.assertTrue(ledger[name], name)
            for index in range(len(ledger[name])):
                changed = copy.deepcopy(ledger)
                changed[name].pop(index)
                with self.assertRaises(AssertionError, msg=f"{name}:{index}"):
                    audit.validate_ledger(changed)
                omissions += 1
        self.assertEqual(omissions, 66)
        for name in ledger["totals"]:
            changed = copy.deepcopy(ledger)
            changed["totals"][name] += 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["inventory"][0]["symbol"] = "ConfirmTalentWipe"
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed)

    def test_invented_defaults_signatures_proof_and_expansion_rejected(self):
        ledger = audit.build()
        for name in ["defaults", "count_claims"]:
            changed = copy.deepcopy(ledger)
            changed[name].append({"symbol": "GetCVarDefault", "value": "1"})
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field in ["arguments", "returns"]:
            changed = copy.deepcopy(ledger)
            changed["signatures"][0][field] = []
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field in ledger["measurements"]:
            changed = copy.deepcopy(ledger)
            changed["measurements"][field] = 1
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["references"][0]["expanded"] = True
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed)

    def test_queued_history_separate_without_invented_120(self):
        ledger = audit.build()
        history = ledger["history"]
        self.assertEqual(history["applied_successors"], [])
        self.assertEqual(
            sorted(p["version"] for p in history["queued_successor_references"]),
            ["1.3.0", "1.4.0", "1.5.0"],
        )
        self.assertEqual(
            history["navigation_only_next"], "1.2.0; no registry entry, no invented pin"
        )
        self.assertIn(
            "2.0.1",
            [p["version"] for p in history["separate_retail_successor_references"]],
        )
        for version in ["1.13.2", "1.60.1", "2.5.6", "3.4.3", "4.4.2", "5.5.4"]:
            self.assertFalse(audit.is_retail_successor(version), version)

    def test_historical_default_bytes_and_errors(self):
        audit.replay_defaults()


if __name__ == "__main__":
    unittest.main(verbosity=2)
