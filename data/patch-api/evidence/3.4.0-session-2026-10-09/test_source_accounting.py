"""Serialized historical accounting tests; not native/runtime API tests."""
import copy
import json
from pathlib import Path
import unittest

from validate import validate

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / "data/patch-api/sources"


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        self.inputs = {
            "response": (EVIDENCE / "source-response.json").read_bytes(),
            "raw": (SOURCE / "3.4.0-api-changes.wikitext").read_bytes(),
            "pin": json.loads((EVIDENCE / "source-pin.json").read_bytes()),
            "ledger": json.loads((SOURCE / "3.4.0-page-coverage.json").read_bytes()),
            "text": (SOURCE / "3.4.0-api-changes.txt").read_bytes(),
            "profile": json.loads((EVIDENCE / "profile-observation.json").read_bytes()),
        }

    def check(self, **changes):
        return validate(**dict(self.inputs, **changes))

    def test_literal_counts_and_command_in_cvar_header(self):
        result = self.check()
        self.assertEqual(result.get("inventory_occurrences"), 331)
        self.assertEqual(result["kinds"], {
            "global-api": 255, "widgets": 10, "events": 46, "cvars": 19, "command": 1,
        })
        self.assertEqual(result["source_rows"], 386)
        self.assertEqual(result["statuses"], {"metadata-only": 51, "UNPROVEN": 335})
        self.assertEqual(result["prose_limits"], 4)
        self.assertEqual(result["signature_limits"], 266)
        self.assertEqual(result["explicit_signatures"], 0)
        self.assertEqual(result["removal_occurrences"], 31)
        self.assertEqual(result["native_observations"], 0)
        self.assertEqual(result["runtime_observations"], 0)
        headers = result["header_counts"]
        self.assertEqual([(h["header_count"], h["parsed_count"]) for h in headers],
                         [(228, 228), (27, 27), (10, 10), (0, 0),
                          (43, 43), (3, 3), (19, 19), (1, 1)])

    def test_every_inventory_prose_signature_and_source_row_required(self):
        for field in ["inventory_rows", "prose_ledger", "signature_ledger", "source_rows"]:
            for index in range(len(self.inputs["ledger"][field])):
                with self.subTest(field=field, index=index):
                    ledger = copy.deepcopy(self.inputs["ledger"])
                    ledger[field].pop(index)
                    with self.assertRaises(AssertionError):
                        self.check(ledger=ledger)

    def test_literal_identity_and_hash_tampering_rejected(self):
        response = json.loads(self.inputs["response"])
        response["query"]["pages"]["16896"]["revisions"][0]["revid"] += 1
        with self.assertRaisesRegex(AssertionError, "revid"):
            self.check(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, "returned content"):
            self.check(raw=self.inputs["raw"] + b"\n")
        pin = copy.deepcopy(self.inputs["pin"])
        pin["wikitext_sha256"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "source hash"):
            self.check(pin=pin)

    def test_invented_behavior_or_signature_credit_rejected(self):
        for field in ["inventory_rows", "prose_ledger", "signature_ledger"]:
            ledger = copy.deepcopy(self.inputs["ledger"])
            ledger[field][0]["status"] = "native-covered"
            with self.subTest(field=field), self.assertRaises(AssertionError):
                self.check(ledger=ledger)
        ledger = copy.deepcopy(self.inputs["ledger"])
        ledger["signature_ledger"][0]["arguments"] = ["unit"]
        with self.assertRaises(AssertionError):
            self.check(ledger=ledger)

    def test_literal_cvar_metadata_and_command_retained(self):
        rows = self.inputs["ledger"]["inventory_rows"]
        self.assertEqual(next(r for r in rows if r["symbol"] == "cameraFov")
                         ["page_metadata"]["default"], "90")
        command = next(r for r in rows if r["symbol"] == "SetRaidDifficulty")
        self.assertEqual((command["kind"], command["section"], command["direction"]),
                         ("command", "cvars", "added"))
        ledger = copy.deepcopy(self.inputs["ledger"])
        next(r for r in ledger["inventory_rows"] if r["symbol"] == "SetRaidDifficulty")["kind"] = "cvar"
        with self.assertRaises(AssertionError):
            self.check(ledger=ledger)
        ledger = copy.deepcopy(self.inputs["ledger"])
        next(r for r in ledger["inventory_rows"] if r["symbol"] == "cameraFov")["page_metadata"]["default"] = "0"
        with self.assertRaises(AssertionError):
            self.check(ledger=ledger)

    def test_partial_signature_not_reconstructed(self):
        row = self.inputs["ledger"]["signature_ledger"][-1]
        self.assertEqual(row["symbol"], "UnitAura")
        self.assertEqual(row["partial_return_contract"], {
            "name": "shouldConsolidate",
            "position": "immediately before variable returns",
            "numeric_slot": None, "type": None, "value_derivation": None,
        })
        for field, value in [("numeric_slot", 16), ("type", "boolean"),
                             ("value_derivation", "false")]:
            ledger = copy.deepcopy(self.inputs["ledger"])
            ledger["signature_ledger"][-1]["partial_return_contract"][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                self.check(ledger=ledger)

    def test_wrong_profile_or_foreign_supersession_rejected(self):
        for field, value in [("client_line", "retail"), ("profile", "mists"),
                             ("later_registers", ["10.0.2"]),
                             ("later_registers", ["4.4.0"]),
                             ("later_registers", ["3.4.3"])]:
            with self.subTest(field=field, value=value):
                ledger = copy.deepcopy(self.inputs["ledger"])
                ledger[field] = value
                with self.assertRaisesRegex(AssertionError, "client history"):
                    self.check(ledger=ledger)

    def test_profile_support_is_not_runtime_or_native_credit(self):
        for field, value in [("profile", "retail"), ("supported_profile", False),
                             ("source_toc", 38001), ("runtime_observations", 1),
                             ("native_observations", 1)]:
            with self.subTest(field=field):
                profile = copy.deepcopy(self.inputs["profile"])
                profile[field] = value
                with self.assertRaisesRegex(AssertionError, "profile evidence"):
                    self.check(profile=profile)


if __name__ == "__main__":
    unittest.main(verbosity=2)
