"""Own targeted 1.0.0 SOURCE controls; no runtime/native acceptance."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("audit100", EVIDENCE / "audit.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class FrozenSource(unittest.TestCase):
    def test_exact_identity_and_manifest_linked_registry_endpoint(self):
        raw, pin, registry = audit.validate_source()
        self.assertEqual(len(raw), 37407)
        self.assertEqual((pin["pageid"], pin["revid"]), (67688, 5580410))
        self.assertEqual(pin["timestamp"], "2023-10-16T00:50:33Z")
        self.assertEqual(len(registry), 101)
        self.assertEqual(registry[-1]["version"], "1.0.0")
        for changed in [raw + b"\n", raw.replace(b"GetNumLaguages", b"GetNumLanguages")]:
            with self.assertRaises(AssertionError):
                audit.validate_source(raw=changed)
        response = json.loads((EVIDENCE / "source-response.json").read_bytes())
        response["query"]["pages"]["67688"]["revisions"][0]["revid"] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(response=json.dumps(response).encode())

    def test_all_literal_rows_and_names_remain_occurrences(self):
        ledger = audit.build()
        raw = (EVIDENCE / "source.wikitext").read_text().splitlines()
        self.assertEqual(
            [(r["line"], r["literal"]) for r in ledger["source_rows"]],
            [(n, line) for n, line in enumerate(raw, 1) if line.strip()],
        )
        self.assertEqual(len(ledger["inventory"]), 854)
        self.assertEqual(ledger["inventory"][0]["symbol"], "AbandonQuest")
        self.assertEqual(ledger["inventory"][-1]["symbol"], "UseSoulstone")
        for row, literal in zip(ledger["inventory"], raw[7:]):
            self.assertEqual(row["literal"], literal)
            self.assertEqual(row["status"], "UNPROVEN")
            self.assertEqual(row["direction"], "listed-unspecified")
        symbols = {r["symbol"] for r in ledger["inventory"]}
        for name in ["GetNumLaguages", "SetBagPortaitTexture", "SetInventoryPortaitTexture", "UnstablePet"]:
            self.assertIn(name, symbols)
        self.assertNotIn("GetNumLanguages", symbols)
        self.assertEqual(ledger["totals"]["physical_lines"], 861)
        self.assertEqual(ledger["totals"]["nonblank_rows"], 859)

    def test_unspecified_signatures_defaults_and_contracts(self):
        ledger = audit.build()
        self.assertEqual(len(ledger["signatures"]), 854)
        self.assertEqual(ledger["defaults"], [])
        self.assertEqual(len(ledger["contracts"]), 859)
        for contract in ledger["contracts"]:
            self.assertEqual(contract["status"], "UNPROVEN")
            self.assertTrue(contract["missing_contract"])
            for field in ["arguments", "returns", "defaults", "events", "state_transitions", "security", "native_equivalence"]:
                self.assertIsNone(contract[field], (contract["id"], field))
        self.assertEqual(ledger["measurements"], {"model": 0, "runtime": 0, "native": 0})
        self.assertEqual(ledger["model_review"]["result"], "no-grounded-behavioral-subset")

    def test_prose_headers_links_templates_are_unexpanded(self):
        ledger = audit.build()
        self.assertEqual([(h["line"], h["literal"], h["declared_count"]) for h in ledger["headers"]], [(3, "==FrameXML==", None), (6, "==Global API==", None)])
        self.assertEqual([r["line"] for r in ledger["prose"]], [4, 7])
        self.assertIn("1.1.2.4115", ledger["prose"][0]["literal"])
        self.assertIn("wow.exe v1.0.0 strings dump", ledger["prose"][1]["literal"])
        self.assertEqual(len(ledger["references"]), 856)
        self.assertEqual([r["target"] for r in ledger["references"][:2]], ["https://www.townlong-yak.com/framexml/1.1.2", "https://warcraft.wiki.gg/wiki/Global_functions?oldid=4864"])
        self.assertTrue(all(r["expanded"] is False for r in ledger["references"]))
        self.assertEqual(ledger["templates"][0]["literal"], "{{apichanges|1.0.0|next=1.1.0}}")
        self.assertEqual(ledger["templates"][0]["expanded"], False)

    def test_every_occurrence_and_count_is_required(self):
        ledger = audit.build()
        controls = 0
        for collection in ["source_rows", "inventory", "signatures", "prose", "headers", "references", "templates", "contracts"]:
            self.assertTrue(ledger[collection], collection)
            for index in range(len(ledger[collection])):
                # Shallow outer copy plus replaced list: original ledger remains unchanged.
                changed = dict(ledger)
                changed[collection] = ledger[collection][:index] + ledger[collection][index + 1:]
                with self.assertRaises(AssertionError, msg=f"{collection}:{index}"):
                    audit.validate_ledger(changed, expected=ledger)
                controls += 1
        for key in ledger["totals"]:
            changed = copy.deepcopy(ledger)
            changed["totals"][key] += 1
            with self.assertRaises(AssertionError, msg=key):
                audit.validate_ledger(changed, expected=ledger)
        print(f"omission controls: {controls}")

    def test_invented_aliases_defaults_credit_and_mutations_rejected(self):
        ledger = audit.build()
        mutations = []
        for collection in ["inventory", "signatures", "defaults", "prose", "headers", "references", "templates", "contracts"]:
            changed = copy.deepcopy(ledger)
            changed[collection].append({"symbol": "InventedNativeAPI"})
            mutations.append(changed)
        for key in ledger["measurements"]:
            changed = copy.deepcopy(ledger)
            changed["measurements"][key] = 1
            mutations.append(changed)
        for field, value in [("symbol", "CorrectedAlias"), ("direction", "added"), ("literal", "fabricated")]:
            changed = copy.deepcopy(ledger)
            changed["inventory"][0][field] = value
            mutations.append(changed)
        for field in ["arguments", "returns", "defaults", "events", "state_transitions", "security", "native_equivalence"]:
            changed = copy.deepcopy(ledger)
            changed["contracts"][0][field] = "invented"
            mutations.append(changed)
        changed = copy.deepcopy(ledger)
        changed["references"][0]["expanded"] = True
        mutations.append(changed)
        changed = copy.deepcopy(ledger)
        changed["history"]["applied_successors"] = ["1.1.0"]
        mutations.append(changed)
        for changed in mutations:
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed, expected=ledger)

    def test_queued_and_foreign_histories_never_apply(self):
        ledger = audit.build()
        history = ledger["history"]
        self.assertEqual(history["applied_successors"], [])
        self.assertEqual([r["version"] for r in history["queued_successor_references"]], ["1.5.0", "1.4.0", "1.3.0", "1.1.0"])
        self.assertIn("p110-page", history["queue"])
        self.assertIn("2.0.1", [r["version"] for r in history["separate_retail_successor_references"]])
        for version in ["1.13.2", "1.15.9", "1.60.1", "2.5.6", "3.4.3", "4.4.2", "5.5.4"]:
            self.assertFalse(audit.is_retail_successor(version), version)
        self.assertFalse(history["registry_endpoint_closes_parent"])

    def test_historical_default_bytes_and_errors_replay(self):
        audit.replay_defaults()


if __name__ == "__main__":
    unittest.main(verbosity=2)
