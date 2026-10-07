#!/usr/bin/env python3
"""Validate saved 10.1.0 accounting/proof artifacts without rerunning Cargo."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / "data/patch-api/sources"


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_source_rows():
    register_path = SOURCES / "10.1.0-wikitext-register.json"
    register = read_json(register_path)
    ledger = read_json(SOURCES / "10.1.0-page-coverage.json")
    raw_path = SOURCES / "10.1.0-api-changes.wikitext"
    text_path = SOURCES / "10.1.0-api-changes.txt"
    provenance = read_json(SOURCES / "10.1.0-api-changes.provenance.json")
    assert register["source"]["revid"] == 2236681
    assert provenance["wikitext"]["sha256"] == sha256(raw_path)
    assert register["source"]["sha256"] == sha256(raw_path)
    assert ledger["source_sha256"] == sha256(register_path)
    assert ledger["non_inventory_source"]["sha256"] == sha256(text_path)
    spec = importlib.util.spec_from_file_location(
        "extract", ROOT / "tools/extract_patch_non_inventory.py")
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = text_path.read_text()
    assert extractor.extract_text(raw_path.read_text(), preserve_examples=True) == text
    extract_rows = extractor.seed_rows(text, "10.1.0")
    expected_ids = [row["id"] for row in register["entries"]]
    expected_ids += [row["source_id"] for row in extract_rows]
    actual_ids = [row["source_id"] for row in ledger["source_rows"]]
    assert actual_ids == expected_ids
    assert len(actual_ids) == len(set(actual_ids)) == 416
    scout = read_json(EVIDENCE / "p1010-extract-scout.json")
    assert [row["source_id"] for row in scout] == expected_ids[129:]
    for row in scout:
        assert row["literal"] == text.splitlines()[row["extract_line"] - 1]
        assert row["reason"]
    return Counter(row["status"] for row in ledger["source_rows"])


def validate_gap_accounting():
    discovery = read_json(EVIDENCE / "p1010-discovery.json")
    final = read_json(EVIDENCE / "p1010-final.json")
    negative = read_json(EVIDENCE / "p1010-negative.json")
    failed = lambda rows: {key for key, value in rows.items() if not value["ok"]}
    old, current, control = map(failed, (discovery, final, negative))
    assert (len(old), len(current), len(control)) == (46, 36, 37)
    assert current <= old
    assert control - current == {"wt-widgets-AnimationGroup:GetElapsed-348"}
    assert not current - control
    known = read_json(ROOT / "tests/data/patch_10_1_0_sweep_known_gaps.json")
    assert set(known) == current
    review = read_json(EVIDENCE / "p1010-gap-review.json")
    assert len(review) == len({row["source_id"] for row in review}) == 46
    assert {row["source_id"] for row in review} == old
    for row in review:
        assert row["reason"]
        assert row["observed"] == final[row["source_id"]]
        expected = "retained-gap" if row["source_id"] in current else "closed-unused-retirement"
        assert row["outcome"] == expected


def validate_sweeps(proof):
    paths = sorted(SOURCES.glob("*-wikitext-register.json"),
                   key=lambda path: tuple(map(int, path.name.split("-")[0].split("."))))
    registers = [read_json(path) for path in paths]
    sweep_proofs = [row for row in proof["entries"] if row.get("output")]
    assert len(sweep_proofs) == 22
    table = []
    for index, (path, register) in enumerate(zip(paths, registers)):
        patch = register["patch"]
        name = f"patch_{patch.replace('.', '_')}_publication_sweep"
        entry = next(row for row in sweep_proofs if row["command"][4] == name)
        rows = read_json(ROOT / entry["output"])
        assert set(rows) == {row["id"] for row in register["entries"]}
        known_path = ROOT / f"tests/data/patch_{patch.replace('.', '_')}_sweep_known_gaps.json"
        assert {key for key, value in rows.items() if not value["ok"]} == set(read_json(known_path))
        latest = {}
        for later in registers[index + 1:]:
            for row in later["entries"]:
                if row["direction"] != "changed":
                    latest[row["symbol"]] = row
        for row in register["entries"]:
            newer = latest.get(row["symbol"])
            removed = row["direction"] == "removed"
            superseded = None
            if newer and (newer["direction"] == "removed") != removed:
                removed = not removed
                superseded = newer["id"]
            expected = rows[row["id"]]["expected"]
            assert expected["publication"] == ("absent" if removed else "published")
            assert expected["superseded_by"] == superseded
        table.append({"patch": patch, "rows": len(rows),
                      "ok": sum(row["ok"] for row in rows.values()), "exit": entry["exit"]})
    return table


def validate_preservation_and_proof(proof):
    for entry in proof["entries"]:
        assert entry["cwd"] == str(ROOT)
        assert entry["exit"] == entry["expected_exit"]
        assert sha256(ROOT / entry["log"]) == entry["sha256"]
    preservation = read_json(EVIDENCE / "p1010-preservation.json")
    assert len(preservation["files"]) == 134
    for row in preservation["files"]:
        assert sha256(ROOT / row["path"]) == row["sha256"]
    reproduction = read_json(EVIDENCE / "p1010-register-reproduction.json")
    assert len(reproduction) == 22
    assert all(row["byte_identical"] and row["exit"] == 0 for row in reproduction)
    before = read_json(EVIDENCE / "p1010-extract-before.json")
    after = read_json(EVIDENCE / "p1010-extract-after.json")
    assert len(before) == len(after) == 44
    for old, new in zip(before, after):
        assert (old["patch"], old["preserve_examples"]) == (new["patch"], new["preserve_examples"])
        assert old["exit"] == new["exit"]
    warnings = (EVIDENCE / "logs/mists-check.log").read_text().splitlines()
    warnings = [line for line in warnings if line.startswith("warning:")]
    assert len(warnings) == 7
    assert all("iced-wgpu-patched/Cargo.toml" in line or "`iced_wgpu` (manifest)" in line for line in warnings)
    assert (EVIDENCE / "logs/startup.log").read_text().splitlines()[0] == "[]"


def main():
    proof = read_json(EVIDENCE / "p1010-proof.json")
    statuses = validate_source_rows()
    validate_gap_accounting()
    table = validate_sweeps(proof)
    validate_preservation_and_proof(proof)
    print(json.dumps({"result": "PASS", "source_ids": 416,
                      "ledger_statuses": dict(statuses), "sweeps": table,
                      "closed_publication_gaps": 10, "remaining_publication_gaps": 36,
                      "preserved_inputs": 134, "reproduced_registers": 22,
                      "proof_entries": len(proof["entries"])}, indent=2))


if __name__ == "__main__":
    main()
