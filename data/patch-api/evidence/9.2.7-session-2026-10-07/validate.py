"""Validate retained 9.2.7 accounting artifacts without rerunning runtime tests."""

from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / "data/patch-api/sources"


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location(
        "extractor", ROOT / "tools/extract_patch_non_inventory.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    extractor = load_extractor()
    register_path = SOURCES / "9.2.7-wikitext-register.json"
    register = read_json(register_path)
    provenance = read_json(SOURCES / "9.2.7-api-changes.provenance.json")
    raw = SOURCES / "9.2.7-api-changes.wikitext"
    text = SOURCES / "9.2.7-api-changes.txt"
    ledger = read_json(SOURCES / "9.2.7-page-coverage.json")
    scout = read_json(EVIDENCE / "p927-extract-scout.json")
    context = read_json(EVIDENCE / "p927-inventory-context.json")
    observations = read_json(EVIDENCE / "p927-sweep-results.json")
    negative = read_json(EVIDENCE / "p927-negative-results.json")
    assert provenance["revid"] == register["source"]["revid"] == 5227425
    assert provenance["wikitext_sha256"] == register["source"]["sha256"] == sha256(raw)
    assert ledger["source_sha256"] == sha256(register_path)
    assert ledger["non_inventory_source"]["sha256"] == sha256(text)
    for preserve in (False, True):
        assert extractor.extract_text(raw.read_text(), preserve_examples=preserve) == text.read_text()
    entries = register["entries"]
    assert Counter(row["direction"] for row in entries) == {"added": 2, "changed": 1}
    assert all(row["header_count"] == row["parsed_count"] for row in register["header_counts"])
    assert set(observations) == {row["id"] for row in entries}
    assert all(row["ok"] for row in observations.values())
    assert read_json(ROOT / "tests/data/patch_9_2_7_sweep_known_gaps.json") == []
    assert set(negative) == set(observations)
    assert {key for key, value in negative.items() if not value["ok"]} == {
        "wt-events-AUCTION_HOUSE_PURCHASE_COMPLETED-19"}
    assert observations["wt-events-AUCTION_HOUSE_PURCHASE_DELIVERY_DELAY_UPDATE-20"]["expected"]["superseded_by"] == "wt-events-AUCTION_HOUSE_PURCHASE_DELIVERY_DELAY_UPDATE-134"
    extract_ids = {row["source_id"] for row in extractor.seed_rows(text.read_text(), "9.2.7")}
    expected_ids = {row["id"] for row in entries} | extract_ids | {row["source_id"] for row in context}
    actual_ids = [row["source_id"] for row in ledger["source_rows"]]
    assert len(actual_ids) == len(set(actual_ids)) == 27
    assert set(actual_ids) == expected_ids
    assert len(scout) == len(extract_ids) == 23
    for row in scout:
        assert row["literal"] == text.read_text().splitlines()[row["plaintext_line"] - 1]
        assert row["wikitext_line"] <= len(raw.read_text().splitlines())
    counts = Counter(row["status"] for row in ledger["source_rows"])
    assert counts == {"metadata-only": 10, "audit-pending": 14,
                      "bounded-coverage": 1, "partial-development-green": 2}
    capabilities = {row["id"] for row in ledger["capabilities"]}
    for row in ledger["source_rows"]:
        assert set(row["capabilities"]) <= capabilities
        if row["status"] in ("metadata-only", "audit-pending"):
            assert not row["capabilities"]
    preserved = read_json(EVIDENCE / "p927-prior-input-hashes.json")
    for filename, expected in preserved.items():
        assert sha256(SOURCES / filename) == expected, filename
    before = read_json(EVIDENCE / "p927-extract-before.json")
    after = read_json(EVIDENCE / "p927-extract-after.json")
    assert len(before) == len(after) == 52
    assert [(row["patch"], row["preserve_examples"], row["exit"]) for row in before] == [
        (row["patch"], row["preserve_examples"], row["exit"]) for row in after]
    proof = read_json(EVIDENCE / "p927-proof.json")
    assert all(not row["invalidated"] for row in proof)
    assert all(row["exit"] == 0 or "expected_exit" in row for row in proof)
    assert (EVIDENCE / "p927-lua-errors.stdout").read_text().strip() == "[]"
    mists_warnings = [line for line in (EVIDENCE / "p927-mists-check.log").read_text().splitlines()
                     if line.startswith("warning:")]
    assert len(mists_warnings) == 7
    assert all("iced-wgpu-patched/Cargo.toml" in line or "`iced_wgpu` (manifest)" in line
               for line in mists_warnings)
    table = read_json(EVIDENCE / "p927-sweep-table.json")
    assert len(table) == 27
    log = (EVIDENCE / "p927-publication-sweeps.log").read_text()
    assert "28 passed; 0 failed; 28 total" in log
    for row in table:
        name = "patch_" + row["patch"].replace(".", "_") + "_publication_sweep"
        assert f"test {name}::{name} ... ok" in log
        assert row["rows"] == row["ok"] + row["gaps"]
    print(json.dumps({"ledger_rows": len(actual_ids), "statuses": counts,
                      "preserved_inputs": len(preserved), "preserved_mode_outcomes": len(before),
                      "passing_publication_sweeps": len(table)}, indent=2))


if __name__ == "__main__":
    main()
