"""Validate retained 11.2.5 source identities, audit credit and local proof."""
import collections
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / "data/patch-api/sources"


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location(
        "extract", ROOT / "tools/extract_patch_non_inventory.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_sources(register, coverage):
    provenance = load(SOURCES / "11.2.5-api-changes.provenance.json")
    assert provenance["pageid"] == 641912
    assert provenance["wikitext"]["revid"] == 6726772
    assert register["source"]["revid"] == 6726772
    assert digest(ROOT / register["source"]["path"]) == register["source"]["sha256"]
    assert register["source"]["sha256"] == provenance["wikitext"]["sha256"]
    assert coverage["source_sha256"] == digest(ROOT / coverage["source_register"])
    assert len(register["entries"]) == 163
    assert len(register["header_counts"]) == 8
    assert all(row["header_count"] == row["parsed_count"] for row in register["header_counts"])
    source = coverage["non_inventory_source"]
    assert digest(ROOT / source["path"]) == source["sha256"]
    assert source["wikitext_sha256"] == register["source"]["sha256"]
    assert source["wikitext_revid"] == 6726772


def validate_inventory(register, coverage, results):
    entries = {row["id"]: row for row in register["entries"]}
    assert len(entries) == 163 and set(entries) == set(results)
    gaps = {key for key, row in results.items() if not row["ok"]}
    known = load(ROOT / "tests/data/patch_11_2_5_sweep_known_gaps.json")
    assert len(known) == len(set(known)) == 45 and set(known) == gaps
    latest = {}
    for patch in ("11.2.7", "12.0.0", "12.0.1", "12.0.5", "12.0.7", "12.1.0"):
        for row in load(SOURCES / f"{patch}-wikitext-register.json")["entries"]:
            if row["direction"] != "changed":
                latest[row["symbol"]] = row
    ledger = {row["source_id"]: row for row in coverage["source_rows"]}
    superseded = []
    strict_absences, aliases = [], []
    for key, source in entries.items():
        observed = results[key]
        expected = observed["expected"]
        assert expected["symbol"] == source["symbol"]
        assert expected["direction"] == source["direction"]
        newer = latest.get(source["symbol"])
        removed = source["direction"] == "removed"
        reversed_direction = newer is not None and (newer["direction"] == "removed") != removed
        effective_removed = not removed if reversed_direction else removed
        assert expected["publication"] == ("absent" if effective_removed else "published")
        assert expected["superseded_by"] == (newer["id"] if reversed_direction else None)
        assert not observed["observed"]["default_mismatch"]
        credit = ledger[key]
        if not observed["ok"]:
            assert credit["status"] == "audit-pending" and not credit["capabilities"]
        else:
            assert credit["capabilities"] == ["publication-sweep-11-2-5"]
            if reversed_direction:
                superseded.append(key)
                assert credit["status"] == "metadata-only"
            elif removed:
                assert credit["status"] == "bounded-coverage"
                if "deprecated-fallback=" in observed["observed"]["detail"]:
                    aliases.append(key)
                    assert "not strict raw absence" in credit["note"]
                else:
                    strict_absences.append(key)
            else:
                assert credit["status"] == "partial-development-green"
    assert len(superseded) == 4 and len(strict_absences) == 7 and len(aliases) == 15
    return {"gaps": len(gaps), "superseded_ok": len(superseded),
            "strict_removal_absences": len(strict_absences), "cached_deprecated_aliases": len(aliases)}


def validate_review_and_control(results):
    initial = load(SESSION / "p1125-initial-sweep.json")
    review = load(SESSION / "p1125-gap-review.json")
    reviewed = {row["source_id"]: row for row in review["rows"]}
    assert len(reviewed) == len(review["rows"]) == 47
    assert set(reviewed) == {key for key, row in initial.items() if not row["ok"]}
    for key, row in reviewed.items():
        assert row["initial"] == initial[key] and row["final"] == results[key]
        assert (row["decision"] == "fixed") == results[key]["ok"]
        assert row["reason"]
    fixed = {row["symbol"] for row in reviewed.values() if row["decision"] == "fixed"}
    assert fixed == {"GetAllowRecentAlliesSeeLocation", "SetAllowRecentAlliesSeeLocation"}
    negative = load(SESSION / "p1125-negative-result.json")
    key = "wt-global-api-C_AddOns.GetAddOnName-25"
    assert set(negative) == set(results)
    assert [entry for entry in results if results[entry] != negative[entry]] == [key]
    assert not negative[key]["ok"] and results[key]["ok"]
    assert sum(not row["ok"] for row in negative.values()) == 46
    control = load(SESSION / "p1125-negative-control.json")
    assert control["actual_exit_code"] == control["expected_exit_code"] == 101
    assert control["new_ids"] == [key] and not control["resolved_ids"]


def validate_extract(coverage):
    extractor = load_extractor()
    text = extractor.extract_text((SOURCES / "11.2.5-api-changes.wikitext").read_text())
    assert text == (SOURCES / "11.2.5-api-changes.txt").read_text()
    expected = extractor.seed_rows(text, "11.2.5")
    actual = [row for row in coverage["source_rows"] if not row["source_id"].startswith("wt-")]
    assert actual == expected and len(actual) == 73
    assignment = load(SESSION / "p1125-extract-assignments.json")
    pending = [key for keys in assignment["batches"].values() for key in keys]
    metadata = assignment["metadata"]
    assert len(pending) == len(set(pending)) == 58
    assert len(metadata) == len(set(metadata)) == 15
    assert not set(pending) & set(metadata)
    assert set(pending + metadata) == {row["source_id"] for row in actual}
    scout = (SESSION / "p1125-extract-scout.md").read_text()
    assert all(scout.count(f"`{key}`") == 1 for key in pending + metadata)
    return {key: len(values) for key, values in assignment["batches"].items()}


def validate_proof():
    proof = load(SESSION / "p1125-proof.json")
    sweeps = [row for row in proof["results"] if "rows" in row]
    assert len(sweeps) == 7 and all(row["exit_code"] == 0 for row in sweeps)
    for row in sweeps:
        results = load(ROOT / row["result_path"])
        assert len(results) == row["rows"]
        assert sum(value["ok"] for value in results.values()) == row["ok"]
        assert sum(not value["ok"] for value in results.values()) == row["gaps"]
        assert row["revision"] == proof["runtime_revision"]
    for name in ("fmt", "retail-check", "mists-check"):
        path = f"data/patch-api/evidence/11.2.5-session-2026-10-06/p1125-{name}.log"
        rows = [row for row in proof["results"] if row.get("log_path") == path]
        assert len(rows) == 1 and rows[0]["exit_code"] == 0
    mists = (SESSION / "p1125-mists-check.log").read_text()
    warnings = [line for line in mists.splitlines() if line.startswith("warning:")]
    assert len(warnings) == 7
    assert all("iced-wgpu-patched/Cargo.toml" in line or "`iced_wgpu` (manifest)" in line for line in warnings)
    assert not any(line.startswith("error") for line in mists.splitlines())
    startup = load(SESSION / "p1125-startup-result.json")
    assert startup["exit_code"] == 0 and startup["json"] == []
    assert startup["unique_errors"] == startup["occurrences"] == 0
    stdout = (ROOT / startup["stdout_path"]).read_text()
    assert stdout.splitlines()[-1] == "[]"
    return [{key: row[key] for key in ("rows", "ok", "gaps", "exit_code")} for row in sweeps]


def main():
    register = load(SOURCES / "11.2.5-wikitext-register.json")
    coverage = load(SOURCES / "11.2.5-page-coverage.json")
    results = load(SESSION / "p1125-sweep-result.json")
    validate_sources(register, coverage)
    inventory = validate_inventory(register, coverage, results)
    validate_review_and_control(results)
    batches = validate_extract(coverage)
    sweeps = validate_proof()
    rows = coverage["source_rows"]
    ids = [row["source_id"] for row in rows]
    assert len(ids) == len(set(ids)) == 236
    counts = dict(collections.Counter(row["status"] for row in rows))
    assert counts == {"partial-development-green": 92, "audit-pending": 103,
                      "metadata-only": 19, "bounded-coverage": 22}
    capability = coverage["capabilities"][0]
    for path in capability["tests"] + [capability["spec"], capability["ledger"]]:
        assert (ROOT / path).is_file()
    output = {"result": "PASS", "inventory_rows": 163, "extract_rows": 73,
              "unique_source_ids": len(ids), "statuses": counts, "inventory": inventory,
              "scout_batches": batches, "isolated_sweeps": sweeps}
    (SESSION / "p1125-page-validation.json").write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
