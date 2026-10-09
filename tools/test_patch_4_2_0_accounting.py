"""Bounded development fixtures for the pinned 4.2.0 serialized audit contract."""

import hashlib
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import extract_patch_non_inventory as extract
import gen_patch_wikitext_register as generator

ROOT = Path(__file__).resolve().parent.parent
SOURCES = ROOT / "data/patch-api/sources"
EVIDENCE = ROOT / "data/patch-api/evidence/4.2.0-session-2026-10-09"


def load(path):
    return json.loads(path.read_text())


class Patch420Accounting(unittest.TestCase):
    def test_response_and_pin_match_literal_source(self):
        pin = load(EVIDENCE / "source-pin.json")
        response = load(EVIDENCE / "source-response.json")
        page = response["query"]["pages"][str(pin["pageid"])]
        revision = page["revisions"][0]
        raw = (SOURCES / "4.2.0-api-changes.wikitext").read_bytes()
        self.assertEqual(page["title"], pin["title"])
        self.assertEqual(revision["revid"], pin["revid"])
        self.assertEqual(revision["timestamp"], pin["timestamp"])
        self.assertEqual(revision["slots"]["main"]["*"].encode(), raw)
        self.assertEqual(hashlib.sha256(raw).hexdigest(), pin["wikitext_sha256"])
        self.assertEqual(
            hashlib.sha256((EVIDENCE / "source-response.json").read_bytes()).hexdigest(),
            pin["response_sha256"],
        )

    def test_saved_register_reproduces_every_literal_inventory_reference(self):
        source = SOURCES / "4.2.0-api-changes.wikitext"
        register = load(SOURCES / "4.2.0-wikitext-register.json")
        provenance = load(SOURCES / "4.2.0-api-changes.provenance.json")
        with tempfile.TemporaryDirectory(dir=EVIDENCE) as directory:
            output = Path(directory) / "register.json"
            argv = ["generator", "4.2.0", str(source), str(provenance["revid"]), str(output)]
            with patch.object(sys, "argv", argv + provenance["generator_flags"]):
                generator.main()
            self.assertEqual(output.read_bytes(), (SOURCES / "4.2.0-wikitext-register.json").read_bytes())
        literal = [
            (number, match[1])
            for number, line in enumerate(source.read_text().splitlines(), 1)
            if (match := re.fullmatch(r": \{\{api\|t=a\|([^{}|]+)\}\}", line))
        ]
        self.assertEqual(literal, [(row["wikitext_line"], row["symbol"]) for row in register["entries"]])
        for count in register["header_counts"]:
            observed = sum(row["direction"] == count["direction"] for row in register["entries"])
            self.assertEqual(count["header_count"], observed)
            self.assertEqual(count["parsed_count"], observed)

    def test_extracted_boundary_has_only_navigation_not_prose_or_signatures(self):
        raw = (SOURCES / "4.2.0-api-changes.wikitext").read_text()
        text = extract.extract_text(raw)
        self.assertEqual(text, (SOURCES / "4.2.0-api-changes.txt").read_text())
        self.assertEqual(text.strip(), "Patch 4.2.0 API changes")
        rows = extract.seed_rows(text, "4.2.0")
        self.assertEqual([row["source_id"] for row in rows], ["source-context-001"])
        self.assertTrue(all(row["status"] == "metadata-only" for row in rows))

    def test_accounting_matches_development_observations(self):
        register = load(SOURCES / "4.2.0-wikitext-register.json")
        coverage = load(SOURCES / "4.2.0-page-coverage.json")
        results = load(EVIDENCE / "development-results.json")
        gaps = load(ROOT / "tests/data/patch_4_2_0_sweep_known_gaps.json")
        ids = {row["id"] for row in register["entries"]}
        self.assertEqual(set(results), ids)
        self.assertEqual(set(gaps), {key for key, result in results.items() if not result["ok"]})
        rows = {row["source_id"]: row for row in coverage["source_rows"]}
        self.assertEqual(set(rows), ids | {"source-context-001"})
        for key, result in results.items():
            row = rows[key]
            self.assertEqual(row["status"], "bounded-coverage" if result["ok"] else "audit-pending")
            self.assertEqual(row["capabilities"], ["publication-absence"] if result["ok"] else [])
            self.assertTrue(row["note"])
        self.assertEqual(rows["source-context-001"]["status"], "metadata-only")
        self.assertEqual(
            coverage["source_sha256"],
            hashlib.sha256((SOURCES / "4.2.0-wikitext-register.json").read_bytes()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
