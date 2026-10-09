"""Post-seal category correction; original historical replay stays immutable."""

import copy
import json
from pathlib import Path
import unittest
import audit
import current_accounting

EVIDENCE = Path(__file__).resolve().parent


class CategoryAccounting(unittest.TestCase):
    def test_new_categories_and_changed_no_category(self):
        ledger = current_accounting.build()
        expected = (
            ["Battlefields"] * 8
            + ["PvP Stats"] * 8
            + ["Character inspection and honor data"] * 6
            + ["Miscellaneous"] * 4
            + [None]
        )
        self.assertEqual([r["group"] for r in ledger["inventory"]], expected)
        self.assertEqual(ledger["inventory"][-1]["direction"], "Changed")
        self.assertEqual(ledger["inventory"][-1]["symbol"], "TogglePVP")
        original = audit.build()
        changes = []
        for before, after in zip(
            original["inventory"], ledger["inventory"], strict=True
        ):
            if before != after:
                changes.append((before["symbol"], before["group"], after["group"]))
        self.assertEqual(changes, [("TogglePVP", "Miscellaneous", None)])
        self.assertEqual(original["totals"], ledger["totals"])
        self.assertEqual(original["contracts"], ledger["contracts"])
        self.assertEqual(original["measurements"], ledger["measurements"])

    def test_wrong_category_or_omitted_row_rejected(self):
        ledger = current_accounting.build()
        for i in range(len(ledger["inventory"])):
            changed = copy.deepcopy(ledger)
            changed["inventory"][i]["group"] = "Invented category"
            with self.assertRaisesRegex(AssertionError, "current literal ledger"):
                current_accounting.validate_ledger(changed)
            changed = copy.deepcopy(ledger)
            changed["inventory"].pop(i)
            with self.assertRaisesRegex(AssertionError, "current literal ledger"):
                current_accounting.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["inventory"][-1]["group"] = "Miscellaneous"
        with self.assertRaisesRegex(AssertionError, "current literal ledger"):
            current_accounting.validate_ledger(changed)
        print(json.dumps({"current_group_and_omission_controls": 55}))

    def test_current_serialized_ledger_originals_unchanged(self):
        self.assertTrue(
            (EVIDENCE / "current-ledger.json").is_file(),
            "corrected serialized ledger absent",
        )
        original_map = (EVIDENCE / "seals.json").read_bytes()
        count = audit.check_seals()
        self.assertEqual(count, 44)
        current_accounting.validate_ledger(
            json.loads((EVIDENCE / "current-ledger.json").read_bytes())
        )
        audit.validate_ledger(audit.read_json("ledger.json"))
        audit.replay_defaults()
        self.assertEqual((EVIDENCE / "seals.json").read_bytes(), original_map)


if __name__ == "__main__":
    unittest.main(verbosity=2)
