"""Own frozen 1.4.0 SOURCE controls; no runtime or native measurements."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
import audit

EVIDENCE = Path(__file__).resolve().parent
EXPECTED = [
    "AcceptBattlefieldPort(accept)",
    "GetBattlefieldEstimatedWaitTime()",
    "GetBattlefieldInfo(index)",
    "GetBattlefieldInstanceExpiration()",
    "GetBattlefieldInstanceInfo(index)",
    "GetBattlefieldPortExpiration()",
    "GetBattlefieldStatus()",
    "ShowBattlefieldList()",
    "GetPVPLastWeekStats()",
    "GetPVPLifetimeStats()",
    "GetPVPRankInfo(rank [, unit])",
    "GetPVPSessionStats()",
    "GetPVPYesterdayStats()",
    'UnitPVPRank("unit")',
    "PLAYER_PVP_KILLS_CHANGED",
    "PLAYER_PVP_RANK_CHANGED",
    "ClearInspectPlayer()",
    "GetInspectHonorData()",
    "HasInspectHonorData()",
    "RequestInspectHonorData()",
    "GetWeaponEnchantInfo()",
    "INSPECT_HONOR_UPDATE",
    "GetCurrentMultisampleFormat()",
    "GetMultisampleFormats()",
    "SetEuropeanNumbers(...)",
    "SetMultisampleFormat(index)",
    "TogglePVP()",
]


class SourceAccounting(unittest.TestCase):
    def test_exact_identity_and_literal_rows(self):
        raw, pin, registry = audit.validate_source()
        self.assertIn("pageid", pin, "frozen identity absent")
        self.assertEqual((pin["pageid"], pin["revid"]), (316397, 3052898))
        self.assertEqual(pin["timestamp"], "2021-04-23T23:31:01Z")
        self.assertEqual(len(registry), 101)
        self.assertEqual(registry[-1]["version"], "1.0.0")
        ledger = audit.build()
        self.assertEqual(
            [(r["line"], r["literal"]) for r in ledger["source_rows"]],
            [
                (i, line)
                for i, line in enumerate(raw.decode().splitlines(), 1)
                if line.strip()
            ],
        )
        self.assertEqual(
            [r["literal_signature"] for r in ledger["inventory"]], EXPECTED
        )
        self.assertEqual(ledger["totals"]["physical_lines"], 42)
        self.assertEqual(ledger["totals"]["nonblank_rows"], 34)
        self.assertEqual(ledger["totals"]["inventory"], 27)
        self.assertEqual(ledger["totals"]["signatures"], 25)
        self.assertEqual(ledger["totals"]["headers"], 6)
        self.assertEqual(ledger["totals"]["templates"], 28)
        self.assertEqual(ledger["totals"]["references"], 3)
        self.assertEqual(ledger["totals"]["navigation"], 2)
        self.assertEqual(ledger["totals"]["prose"], 25)
        self.assertEqual(ledger["totals"]["contracts"], 32)
        self.assertEqual(ledger["defaults"], [])
        self.assertEqual(ledger["count_claims"], [])

    def test_every_boundary_omission_and_count_mutation_rejected(self):
        ledger = audit.build()
        controls = 0
        for name in [
            "source_rows",
            "inventory",
            "signatures",
            "prose",
            "headers",
            "templates",
            "references",
            "navigation",
            "contracts",
        ]:
            for i in range(len(ledger[name])):
                changed = copy.deepcopy(ledger)
                changed[name].pop(i)
                with self.assertRaisesRegex(
                    AssertionError, "serialized literal ledger"
                ):
                    audit.validate_ledger(changed)
                controls += 1
        for name in ledger["totals"]:
            changed = copy.deepcopy(ledger)
            changed["totals"][name] += 1
            with self.assertRaisesRegex(AssertionError, "serialized literal ledger"):
                audit.validate_ledger(changed)
            controls += 1
        self.assertGreater(controls, 170)
        print(json.dumps({"omission_and_count_controls": controls}))

    def test_invention_expansion_and_credit_rejected(self):
        ledger = audit.build()
        for name in ["defaults", "count_claims"]:
            changed = copy.deepcopy(ledger)
            changed[name].append({"invented": True})
            with self.assertRaisesRegex(AssertionError, "serialized literal ledger"):
                audit.validate_ledger(changed)
        for name in ["model", "runtime", "native"]:
            changed = copy.deepcopy(ledger)
            changed["measurements"][name] = 1
            with self.assertRaisesRegex(AssertionError, "serialized literal ledger"):
                audit.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["references"][0]["expanded"] = True
        with self.assertRaisesRegex(AssertionError, "serialized literal ledger"):
            audit.validate_ledger(changed)
        changed = copy.deepcopy(ledger)
        changed["contracts"][0]["status"] = "bounded-coverage"
        with self.assertRaisesRegex(AssertionError, "serialized literal ledger"):
            audit.validate_ledger(changed)

    def test_identity_source_response_and_manifest_registry_mutations(self):
        raw = (EVIDENCE / "source.wikitext").read_bytes()
        response = (EVIDENCE / "source-response.json").read_bytes()
        for changed in [raw + b"\n", raw.replace(b"accept)", b"index, accept)", 1)]:
            with self.assertRaises(AssertionError):
                audit.validate_source(raw=changed)
        parsed = json.loads(response)
        page = parsed["query"]["pages"]["316397"]
        for key, value in [("pageid", 316398), ("title", "Patch 1.5.0/API changes")]:
            changed = copy.deepcopy(parsed)
            changed["query"]["pages"]["316397"][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_source(response=json.dumps(changed).encode())
        for key, value in [("revid", 3052899), ("timestamp", "2021-04-24T23:31:01Z")]:
            changed = copy.deepcopy(parsed)
            changed["query"]["pages"]["316397"]["revisions"][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_source(response=json.dumps(changed).encode())
        self.assertEqual(page["pageid"], 316397)
        for name in [
            "source.wikitext",
            "source-response.json",
            "source-pin.json",
            "frozen-manifest.json",
            "frozen-registry.json",
        ]:
            with tempfile.TemporaryDirectory(dir=EVIDENCE) as folder:
                copied = Path(folder)
                for input_name in [
                    "source.wikitext",
                    "source-response.json",
                    "source-pin.json",
                    "frozen-manifest.json",
                    "frozen-registry.json",
                ]:
                    (copied / input_name).write_bytes(
                        (EVIDENCE / input_name).read_bytes()
                    )
                path = copied / name
                path.write_bytes(path.read_bytes() + b" ")
                original = audit.EVIDENCE
                audit.EVIDENCE = copied
                try:
                    with self.assertRaises(AssertionError):
                        audit.validate_source()
                finally:
                    audit.EVIDENCE = original

    def test_precise_contract_limits_no_alias_or_default(self):
        ledger = audit.build()
        self.assertTrue(ledger["inventory"], "literal declarations absent")
        first = ledger["inventory"][0]
        self.assertEqual(first["literal_signature"], "AcceptBattlefieldPort(accept)")
        self.assertEqual(
            ledger["contracts"][0]["source_claim"],
            "Called with true to accept porting into a battlefield, or false to reject/leave queue.",
        )
        self.assertTrue(all(c["status"] == "UNPROVEN" for c in ledger["contracts"]))
        self.assertTrue(
            all(c["native_equivalence"] is None for c in ledger["contracts"])
        )
        rank = next(s for s in ledger["signatures"] if s["symbol"] == "GetPVPRankInfo")
        self.assertEqual(rank["arguments_literal"], "rank [, unit]")
        self.assertIsNone(rank["optional_argument_default"])
        replaced = next(s for s in ledger["signatures"] if s["symbol"] == "EnablePVP")
        self.assertEqual(replaced["role"], "replaced-prose-reference")
        unknown = next(
            c for c in ledger["contracts"] if c.get("symbol") == "SetEuropeanNumbers"
        )
        self.assertIn("unknown purpose", unknown["source_claim"])
        self.assertEqual(
            ledger["measurements"], {"model": 0, "runtime": 0, "native": 0}
        )

    def test_separate_unapplied_histories(self):
        history = audit.build()["history"]
        self.assertEqual(history["applied_successors"], [])
        self.assertTrue(history["queued_successor_references"], "queue identity absent")
        self.assertEqual(history["queued_successor_references"][0]["version"], "1.5.0")
        self.assertTrue(
            all(
                audit.is_retail_successor(p["version"])
                for p in history["separate_retail_successor_references"]
            )
        )
        self.assertIn("Forever", history["foreign_history_limit"])

    def test_historical_default_bytes_and_errors(self):
        self.assertTrue((EVIDENCE / "default-register.json").is_file())
        audit.replay_defaults()


if __name__ == "__main__":
    unittest.main(verbosity=2)
