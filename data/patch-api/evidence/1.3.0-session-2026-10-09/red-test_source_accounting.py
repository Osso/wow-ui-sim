"""Own frozen 1.3.0 SOURCE controls; no runtime/native acceptance."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("audit130", EVIDENCE / "audit.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class FrozenPage(unittest.TestCase):
    def ledger(self):
        ledger = audit.build()
        self.assertEqual(ledger.get("patch"), "1.3.0", "own source accounting absent")
        return ledger

    def test_all_literal_declarations_and_old_rename_identity(self):
        ledger = self.ledger()
        self.assertEqual(len(ledger["inventory"]), 31)
        self.assertEqual(sum(r["change"] == "New" for r in ledger["inventory"]), 22)
        self.assertEqual(sum(r["change"] == "Removed" for r in ledger["inventory"]), 7)
        self.assertEqual(
            [r["literal"] for r in ledger["inventory"] if r["change"] == "Changed"],
            ["TabardModel:CanSave()", "TabardModel:CanSaveTabardNow()"],
        )
        self.assertEqual(ledger["inventory"][0]["literal"], "AddQuestWatch(index)")
        self.assertEqual(ledger["inventory"][6]["literal"], 'IsUnitOnQuest(index,"unit")')
        self.assertEqual(ledger["inventory"][11]["literal"], "SetActionBarToggles(show1,show2,show3,show4)")
        self.assertEqual(ledger["measurements"], {"model": 0, "runtime": 0, "native": 0})

    def test_every_row_header_link_template_prose_and_default_limit(self):
        ledger = self.ledger()
        raw = (EVIDENCE / "source.wikitext").read_text()
        self.assertEqual([r["literal"] for r in ledger["source_rows"]], raw.splitlines())
        self.assertEqual(len(ledger["source_rows"]), 44)
        self.assertEqual(len(ledger["headers"]), 7)
        self.assertEqual(len(ledger["references"]), 4)
        self.assertEqual(len(ledger["templates"]), 31)
        self.assertEqual(len(ledger["prose"]), 3)
        self.assertEqual(ledger["defaults"], [])
        self.assertEqual(ledger["count_claims"], [])
        self.assertEqual(len(ledger["signatures"]), 31)
        self.assertEqual(sum(r["argument_literal"] is not None for r in ledger["signatures"]), 23)
        for row in ledger["signatures"]:
            self.assertIsNone(row["returns"])
            self.assertIsNone(row["defaults"])
            self.assertFalse(row["complete_contract"])
        self.assertEqual(len(ledger["contracts"]), 76)
        self.assertTrue(all(r["status"] == "UNPROVEN" for r in ledger["contracts"]))
        self.assertTrue(all(not r["expanded"] for r in ledger["references"] + ledger["templates"]))

    def test_each_omission_and_count_mutation_rejected(self):
        ledger = self.ledger()
        for category in ["source_rows", "inventory", "signatures", "headers", "references", "templates", "prose", "contracts"]:
            for index in range(len(ledger[category])):
                changed = copy.deepcopy(ledger)
                changed[category].pop(index)
                with self.assertRaises(AssertionError, msg=f"{category}:{index}"):
                    audit.validate_ledger(changed)
        for name in ledger["totals"]:
            changed = copy.deepcopy(ledger)
            changed["totals"][name] += 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)

    def test_invented_alias_default_expansion_or_native_credit_rejected(self):
        ledger = self.ledger()
        for category, field, value in [
            ("inventory", "literal", "C_QuestLog.AddQuestWatch(index)"),
            ("signatures", "returns", "boolean"),
            ("signatures", "defaults", False),
            ("references", "expanded", True),
            ("templates", "expanded", True),
            ("contracts", "status", "PROVEN"),
        ]:
            changed = copy.deepcopy(ledger)
            changed[category][0][field] = value
            with self.assertRaises(AssertionError, msg=f"{category}:{field}"):
                audit.validate_ledger(changed)
        for category in ["defaults", "count_claims"]:
            changed = copy.deepcopy(ledger)
            changed[category].append({"invented": True})
            with self.assertRaises(AssertionError, msg=category):
                audit.validate_ledger(changed)
        for name in ["model", "runtime", "native"]:
            changed = copy.deepcopy(ledger)
            changed["measurements"][name] = 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_ledger(changed)

    def test_exact_frozen_body_response_identity_mutations_rejected(self):
        validated = audit.validate_source()
        self.assertIsNotNone(validated, "own source identity absent")
        raw, pin, registry = validated
        self.assertEqual((len(raw), pin["pageid"], pin["revid"]), (1364, 350208, 3376287))
        self.assertEqual(len(registry), 101)
        self.assertEqual(registry[-1]["version"], "1.0.0")
        for changed in [raw + b"\n", raw.replace(b"show4", b"show5")]:
            with self.assertRaises(AssertionError):
                audit.validate_source(raw=changed)
        response = json.loads((EVIDENCE / "source-response.json").read_bytes())
        for field, value in [("pageid", 350209), ("title", "Patch 1.4.0/API changes"), ("revid", 3376288), ("timestamp", "2021-04-23T02:34:09Z")]:
            changed = copy.deepcopy(response)
            page = changed["query"]["pages"]["350208"]
            target = page if field in {"pageid", "title"} else page["revisions"][0]
            target[field] = value
            with self.assertRaises(AssertionError, msg=field):
                audit.validate_source(response=json.dumps(changed).encode())

    def test_manifest_registry_identity_mutations_rejected(self):
        self.assertIsNotNone(audit.validate_source(), "own manifest identity absent")
        for name, parameter in [("frozen-manifest.json", "manifest_bytes"), ("frozen-registry.json", "registry_bytes")]:
            original = (EVIDENCE / name).read_bytes()
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_source(**{parameter: original + b"\n"})
            changed = json.loads(original)
            selected = next(r for r in changed["pages"] if r["version"] == "1.3.0")
            selected["revid"] += 1
            with self.assertRaises(AssertionError, msg=name):
                audit.validate_source(**{parameter: json.dumps(changed).encode()})

    def test_queued_successors_separate_and_foreign_histories_not_applied(self):
        ledger = self.ledger()
        self.assertEqual(ledger["history"]["applied_successors"], [])
        self.assertEqual([r["version"] for r in ledger["history"]["queued_successor_references"]], ["1.5.0", "1.4.0"])
        for version in ["1.13.2", "1.15.9", "1.60.1", "2.5.6", "3.4.3", "4.4.2", "5.5.4"]:
            self.assertFalse(audit.is_retail_successor(version), version)
        self.assertTrue(audit.is_retail_successor("2.0.1"))

    def test_own_base_default_bytes_and_error_replay(self):
        audit.replay_defaults()


if __name__ == "__main__":
    unittest.main(verbosity=2)
