"""Validate retained 11.2.0 source identities, coverage credit and local proof."""
import collections
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / "data/patch-api/sources"
LATER = ("11.2.5", "11.2.7", "12.0.0", "12.0.1", "12.0.5", "12.0.7", "12.1.0")


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    provenance = load(SOURCES / "11.2.0-api-changes.provenance.json")
    assert provenance["pageid"] == 636685
    assert provenance["wikitext"]["revid"] == register["source"]["revid"] == 6726773
    assert digest(ROOT / register["source"]["path"]) == register["source"]["sha256"]
    assert register["source"]["sha256"] == provenance["wikitext"]["sha256"]
    assert coverage["source_sha256"] == digest(ROOT / coverage["source_register"])
    assert len(register["entries"]) == 162 and len(register["header_counts"]) == 8
    assert all(row["header_count"] == row["parsed_count"] for row in register["header_counts"])
    source = coverage["non_inventory_source"]
    assert digest(ROOT / source["path"]) == source["sha256"]
    assert source["wikitext_sha256"] == register["source"]["sha256"]
    assert source["wikitext_revid"] == 6726773
    regeneration = load(SESSION / "p1120-register-regeneration.json")
    assert {row["patch"] for row in regeneration} == set(LATER)
    assert all(row["exit_code"] == 0 and row["byte_identical"] for row in regeneration)


def validate_inventory(register, coverage, results):
    entries = {row["id"]: row for row in register["entries"]}
    assert len(entries) == 162 and set(entries) == set(results)
    gaps = {key for key, row in results.items() if not row["ok"]}
    known = load(ROOT / "tests/data/patch_11_2_0_sweep_known_gaps.json")
    assert len(known) == len(set(known)) == 26 and set(known) == gaps
    latest = {}
    for patch in LATER:
        for row in load(SOURCES / f"{patch}-wikitext-register.json")["entries"]:
            if row["direction"] != "changed":
                latest[row["symbol"]] = row
    ledger = {row["source_id"]: row for row in coverage["source_rows"]}
    counts = collections.Counter()
    for key, source in entries.items():
        observed = results[key]
        expected = observed["expected"]
        assert expected["symbol"] == source["symbol"] and expected["direction"] == source["direction"]
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
            continue
        assert credit["capabilities"] == ["publication-sweep-11-2-0"]
        if reversed_direction:
            counts["superseded_ok"] += 1
            assert credit["status"] == "metadata-only"
        elif removed:
            assert credit["status"] == "bounded-coverage"
            if "deprecated-fallback=" in observed["observed"]["detail"]:
                counts["deprecated_aliases"] += 1
                assert "not strict raw absence" in credit["note"]
            else:
                counts["strict_removals"] += 1
        else:
            counts["published"] += 1
            assert credit["status"] == "partial-development-green"
    assert counts == {"superseded_ok": 4, "deprecated_aliases": 10, "strict_removals": 47, "published": 75}
    return dict(counts)


def validate_review_and_control(results):
    initial = load(SESSION / "p1120-initial-sweep.json")
    review = load(SESSION / "p1120-gap-review.json")["rows"]
    assert len(review) == len({row["source_id"] for row in review}) == 38
    assert {row["source_id"] for row in review} == {key for key, row in initial.items() if not row["ok"]}
    for row in review:
        key = row["source_id"]
        assert row["initial"] == initial[key] and row["final"] == results[key]
        assert (row["decision"] == "fixed") == results[key]["ok"]
        assert row["reason"] and row["investigate"]
    assert sum(row["decision"] == "fixed" for row in review) == 12
    negative = load(SESSION / "p1120-negative-result.json")
    key = "wt-global-api-C_ChallengeMode.GetLeaverPenaltyWarningTimeLeft-43"
    assert set(negative) == set(results)
    assert [entry for entry in results if results[entry] != negative[entry]] == [key]
    assert not negative[key]["ok"] and results[key]["ok"]
    assert sum(not row["ok"] for row in negative.values()) == 27
    control = load(SESSION / "p1120-negative-control.json")
    assert control["actual_exit_code"] == control["expected_exit_code"] == 101
    assert control["changed_ids"] == control["new_ids"] == [key] and not control["resolved_ids"]
    normal_register = load(SOURCES / "11.2.0-wikitext-register.json")
    negative_register = load(SESSION / "p1120-negative-register.json")
    changed = [(a, b) for a, b in zip(normal_register["entries"], negative_register["entries"]) if a != b]
    assert len(normal_register["entries"]) == len(negative_register["entries"]) == 162
    assert len(changed) == 1
    original, modified = changed[0]
    assert original["id"] == key and modified == dict(original, direction="removed")


def validate_extract(coverage):
    spec = importlib.util.spec_from_file_location("extract", ROOT / "tools/extract_patch_non_inventory.py")
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = extractor.extract_text((SOURCES / "11.2.0-api-changes.wikitext").read_text())
    assert text == (SOURCES / "11.2.0-api-changes.txt").read_text()
    expected = extractor.seed_rows(text, "11.2.0")
    actual = [row for row in coverage["source_rows"] if not row["source_id"].startswith("wt-")]
    assert actual == expected and len(actual) == 82
    assignment = load(SESSION / "p1120-extract-assignments.json")
    pending = [key for keys in assignment["batches"].values() for key in keys]
    metadata = assignment["metadata"]
    assert len(pending) == len(set(pending)) == 65
    assert len(metadata) == len(set(metadata)) == 17
    assert not set(pending) & set(metadata)
    assert set(pending + metadata) == {row["source_id"] for row in actual}
    scout = (SESSION / "p1120-extract-scout.md").read_text()
    assert all(scout.count(f"`{key}`") == 1 for key in pending + metadata)
    return {key: len(values) for key, values in assignment["batches"].items()}


def validate_proof(coverage):
    proof = load(SESSION / "p1120-proof.json")
    current = [row for row in proof["results"] if row.get("invalidated_by") is None]
    sweeps = [row for row in current if "rows" in row]
    assert len(sweeps) == 8 and all(row["exit_code"] == 0 for row in sweeps)
    assert coverage["capabilities"][0]["compiled_revision"] == proof["runtime_revision"]
    for row in sweeps:
        results = load(ROOT / row["result_path"])
        assert len(results) == row["rows"]
        assert sum(value["ok"] for value in results.values()) == row["ok"]
        assert sum(not value["ok"] for value in results.values()) == row["gaps"]
        assert row["revision"] == proof["runtime_revision"]
        assert row["command"][-2:] == ["--nocapture", "--test-threads=1"]
        assert "--build-host" in row["command"] and "local" in row["command"]
    for name in ("fmt-final", "retail-check-final", "mists-check", "fixes-final", "extract-green", "register-tests", "extract-reproduce"):
        matched = [row for row in current if row["name"] == name]
        assert len(matched) == 1 and matched[0]["exit_code"] == 0
        if name in ("retail-check-final", "mists-check"):
            log = (ROOT / matched[0]["log_path"]).read_text()
            warnings = [line for line in log.splitlines() if line.startswith("warning:")]
            assert all("iced-wgpu-patched/Cargo.toml" in line or "`iced_wgpu` (manifest)" in line for line in warnings)
            assert not any(line.startswith("error") for line in log.splitlines())
    preservation = load(SESSION / "p1120-baseline-preservation.json")
    assert len(preservation) == 7 and all(row["identical_observations"] for row in preservation)
    for row in preservation:
        assert load(SESSION / row["patch"]) == load(ROOT / row["baseline_path"])
    preserved_inputs = load(SESSION / "p1120-preserved-inputs.json")
    assert all(row["unchanged"] and digest(ROOT / row["path"]) == row["sha256"] for row in preserved_inputs["files"])
    startup = load(SESSION / "p1120-startup-result.json")
    assert startup["exit_code"] == 0 and startup["json"] == []
    assert (ROOT / startup["stdout_path"]).read_text().splitlines()[-1] == "[]"
    return [{key: row[key] for key in ("name", "rows", "ok", "gaps", "exit_code")} for row in sweeps]


def main():
    register = load(SOURCES / "11.2.0-wikitext-register.json")
    coverage = load(SOURCES / "11.2.0-page-coverage.json")
    results = load(SESSION / "p1120-sweep-result.json")
    validate_sources(register, coverage)
    inventory = validate_inventory(register, coverage, results)
    validate_review_and_control(results)
    batches = validate_extract(coverage)
    sweeps = validate_proof(coverage)
    rows = coverage["source_rows"]
    ids = [row["source_id"] for row in rows]
    assert len(ids) == len(set(ids)) == 244
    counts = dict(collections.Counter(row["status"] for row in rows))
    assert counts == {"partial-development-green": 75, "audit-pending": 91, "metadata-only": 21, "bounded-coverage": 57}
    capability = coverage["capabilities"][0]
    for path in capability["tests"] + [capability["spec"], capability["ledger"]]:
        assert (ROOT / path).is_file()
    output = {"result": "PASS", "inventory_rows": 162, "extract_rows": 82,
              "unique_source_ids": len(ids), "statuses": counts, "inventory": inventory,
              "scout_batches": batches, "isolated_sweeps": sweeps}
    (SESSION / "p1120-page-validation.json").write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
