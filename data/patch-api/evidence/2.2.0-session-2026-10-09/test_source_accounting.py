"""Bounded serialized SOURCE contracts, not WoW runtime tests."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        path = EVIDENCE / "validate.py"
        self.assertTrue(path.exists(), "2.2.0 historical validator is missing")
        spec = importlib.util.spec_from_file_location("p220_validate", path)
        self.validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.validator)
        self.ledger = json.loads(
            (EVIDENCE / "historical-page-coverage.json").read_bytes()
        )

    def test_counts_and_literal_headers(self):
        result = self.validator.validate(self.ledger)
        self.assertEqual(result["inventory_occurrences"], 36)
        self.assertEqual(result["signature_limits"], 33)
        self.assertEqual(result["raw_nonblank_rows"], 67)
        self.assertEqual(result["headers"], 11)
        self.assertEqual(result["header_counts"], [])
        self.assertEqual(result["modified_click_examples"], 12)
        self.assertEqual(result["registry_pages"], 101)
        self.assertEqual(result["registry_endpoint"], "1.0.0")
        self.assertEqual(result["runtime_observations"], 0)
        self.assertEqual(result["native_observations"], 0)

    def test_all_serialized_accounting_required(self):
        for field in [
            "inventory_rows",
            "signature_ledger",
            "prose_ledger",
            "source_rows",
            "extract_rows",
            "headers",
            "modified_click_examples",
        ]:
            for index in range(len(self.ledger[field])):
                ledger = copy.deepcopy(self.ledger)
                ledger[field].pop(index)
                with (
                    self.subTest(field=field, index=index),
                    self.assertRaises(AssertionError),
                ):
                    self.validator.validate(ledger)

    def test_literal_optional_and_split_signatures_not_reconstructed(self):
        signatures = self.ledger["signature_ledger"]
        row = next(r for r in signatures if r["symbol"] == "IsModifiedClick")
        self.assertEqual(row["fragment"], 'IsModifiedClick(["action"])')
        self.assertEqual(row["return_fragment"], "active")
        self.assertEqual(
            next(r for r in signatures if r["symbol"] == "SaveBindings")["fragment"],
            "SaveBindings(",
        )
        self.assertEqual(
            next(r for r in signatures if r["symbol"] == "GetCurrentBindingSet")[
                "fragment"
            ],
            "GetCurrentBindingSet())",
        )
        self.assertIn(
            "SecureButton_GetModifiedAttrbute", [r["symbol"] for r in signatures]
        )
        ledger = copy.deepcopy(self.ledger)
        ledger["signature_ledger"][0]["arguments"] = []
        with self.assertRaises(AssertionError):
            self.validator.validate(ledger)

    def test_no_invented_native_or_foreign_supersession(self):
        for field, value in [
            ("client_line", "classic"),
            ("runtime_observations", 1),
            ("native_observations", 1),
            ("model_credit", 1),
            ("later_registers", ["2.5.0"]),
            ("later_registers", ["3.4.0"]),
            ("later_registers", ["2.3.0"]),
        ]:
            ledger = copy.deepcopy(self.ledger)
            ledger[field] = value
            with (
                self.subTest(field=field, value=value),
                self.assertRaises(AssertionError),
            ):
                self.validator.validate(ledger)
        for field in ["inventory_rows", "signature_ledger", "prose_ledger"]:
            ledger = copy.deepcopy(self.ledger)
            ledger[field][0]["status"] = "native-covered"
            with self.subTest(field=field), self.assertRaises(AssertionError):
                self.validator.validate(ledger)

    def test_event_family_cvar_and_replaced_functions_limits(self):
        rows = self.ledger["inventory_rows"]
        self.assertEqual(sum(r["direction"] == "replaced" for r in rows), 6)
        family = next(r for r in rows if r["kind"] == "event-family")
        self.assertEqual(family["symbol"], "UNIT_SPELLCAST_*")
        self.assertEqual(family["expanded_members"], None)
        setting = next(r for r in rows if r["symbol"] == "pitchLimit")
        self.assertEqual(setting["kind"], "cvar-setting")
        self.assertEqual(setting["default"], None)
        self.assertEqual(sum(r["kind"] == "command" for r in rows), 0)
        self.assertEqual(
            [r["symbol"] for r in rows if r["kind"] == "secure-template"],
            ["SecureStateHeader"],
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
