#!/usr/bin/env python3
"""Replay own frozen Wrath source accounting, without Git or runtime probes."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / "data/patch-api/sources"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(
        name, EVIDENCE / f"historical-{name}.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_seals():
    seals = json.loads((EVIDENCE / "seals.json").read_bytes())
    for path, expected in seals.items():
        assert digest((ROOT / path).read_bytes()) == expected, f"seal: {path}"
    return len(seals)


def derive_inventory(raw):
    generator = load_tool("gen_patch_wikitext_register")
    entries, headers = [], []
    lines = raw.splitlines()
    for section, bucket in generator.split_sections(raw).items():
        parsed, counts = generator.parse_section(section, bucket)
        entries.extend(parsed)
        headers.extend(counts)
    for entry in entries:
        entry["source_text"] = lines[entry["wikitext_line"] - 1]
        if entry["section"] == "cvars" and "{{apitooltip|" in entry["source_text"]:
            entry["page_metadata"] = generator.parse_symbol(entry["source_text"])[1]
    assert all(h["header_count"] == h["parsed_count"] for h in headers), "headers"
    return entries, headers


def check_limits(rows, label):
    for row in rows:
        assert row["status"] == "UNPROVEN", f"{label} proof limit"
        assert row["capabilities"] == [], f"{label} invented credit"
        assert row["note"], f"{label} missing limit"


def validate_inventory(raw, ledger):
    entries, headers = derive_inventory(raw)
    rows = ledger["inventory_rows"]
    assert len(rows) == len(entries), "inventory row accounting"
    for row, entry in zip(rows, entries):
        observed = {k: v for k, v in row.items()
                    if k not in ("note", "status", "capabilities")}
        assert observed == entry, "literal inventory accounting"
    check_limits(rows, "inventory")
    assert ledger["header_counts"] == headers, "header accounting"
    return entries, headers


def validate_prose(raw, ledger):
    # Both exact substantive statements, including the widget-equivalence prose
    # omitted by the stock non-inventory extractor. Never expand either link.
    lines = raw.splitlines()
    expected = [(4, "prose-004"), (235, "prose-235")]
    rows = ledger["prose_ledger"]
    assert [(r["wikitext_line"], r["source_id"]) for r in rows] == expected, "prose accounting"
    for row in rows:
        assert row["source_text"] == lines[row["wikitext_line"] - 1], "literal prose"
    check_limits(rows, "prose")
    return rows


def validate_signatures(entries, ledger):
    expected = [{"inventory_id": r["id"], "symbol": r["symbol"],
                 "wikitext_line": r["wikitext_line"], "arguments": None,
                 "returns": None} for r in entries if r["section"] == "global-api"]
    rows = ledger["signature_ledger"]
    observed = [{k: v for k, v in r.items()
                 if k not in ("note", "status", "capabilities")} for r in rows]
    assert observed == expected, "signature accounting"
    check_limits(rows, "signature")
    return rows


def validate_source_rows(raw, entries, prose, ledger):
    byline = {r["wikitext_line"]: r for r in entries + prose}
    expected = []
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        item = byline.get(number)
        expected.append({
            "source_id": (item.get("id", item.get("source_id")) if item
                          else f"source-context-{number:03}"),
            "wikitext_line": number, "source_text": line,
            "category": ("inventory" if item and "symbol" in item else
                         "prose" if item else "metadata"),
            "status": "UNPROVEN" if item else "metadata-only",
            "capabilities": [],
        })
    assert ledger["source_rows"] == expected, "complete literal source accounting"
    return expected


def validate_identity(response, raw, pin, ledger):
    page = json.loads(response)["query"]["pages"]["379792"]
    revision, = page["revisions"]
    for key, value in [("pageid", 379792), ("title", "Patch 3.4.1/API changes")]:
        assert page[key] == pin[key] == ledger["source"][key] == value, key
    for key, value in [("revid", 3656581), ("timestamp", "2023-05-04T13:15:35Z")]:
        assert revision[key] == pin[key] == ledger["source"][key] == value, key
    assert revision["slots"]["main"]["*"].encode() == raw, "returned content"
    assert digest(raw) == pin["wikitext_sha256"] == (
        "b336b152bea05e3609de06ac3971a98e7d2584129e4e3b74a5c333c6fc060ed1"
    ), "source hash"
    assert digest(response) == pin["response_sha256"] == (
        "8fd09b4b807c4191b62843d4e2dbfc18a5313e15796a6e284e24016694e99afe"
    ), "response hash"
    assert len(raw) == pin["wikitext_bytes"], "source bytes"
    assert ledger["source"] == pin, "source pin"


def validate(*, response, raw, pin, ledger, text, profile):
    validate_identity(response, raw, pin, ledger)
    assert (ledger["schema"], ledger["patch"], ledger["client_line"],
            ledger["profile"], ledger["scope"], ledger["later_registers"]) == (
        "patch-source-accounting/v1", "3.4.1", "wrath-classic", "wrath",
        "source-and-profile-accounting", []
    ), "client history"
    assert [q["patch"] for q in ledger["successor_queue"]] == ["3.4.2", "3.4.3"], "Wrath queue"
    extractor = load_tool("extract_patch_non_inventory")
    expected_text = extractor.extract_text(raw.decode(), canonical_patch_navigation=True)
    assert text == expected_text.encode(), "plaintext reproduction"
    assert ledger["non_inventory_source"]["sha256"] == digest(text), "plaintext hash"
    assert ledger["non_inventory_source"]["extractor_flags"] == [
        "--text-only", "--canonical-patch-navigation"
    ], "extractor flags"
    entries, headers = validate_inventory(raw.decode(), ledger)
    prose = validate_prose(raw.decode(), ledger)
    signatures = validate_signatures(entries, ledger)
    rows = validate_source_rows(raw.decode(), entries, prose, ledger)
    expected_profile = {
        "profile": "wrath", "feature": "client-wrath", "supported_profile": True,
        "source_toc": 30401, "configured_interface": 38001, "cache_subdir": "wrath",
        "code_revision": "e7eb38c362be230b00af4096ac8092b9dd147200",
        "runtime_observations": 0, "native_observations": 0, "cache_inspected": False,
    }
    assert {k: profile[k] for k in expected_profile} == expected_profile, "profile evidence"
    return {
        "source_rows": len(rows),
        "statuses": dict(Counter(r["status"] for r in rows)),
        "inventory_occurrences": len(entries),
        "kinds": dict(Counter(r.get("kind", r["section"]) for r in entries)),
        "directions": dict(Counter(r["direction"] for r in entries)),
        "header_counts": headers,
        "removal_occurrences": sum(r["direction"] == "removed" for r in entries),
        "prose_limits": len(prose), "signature_limits": len(signatures),
        "explicit_signatures": sum(r["arguments"] is not None for r in signatures),
        "cvar_defaults": sum("page_default" in r for r in entries),
        "runtime_observations": profile["runtime_observations"],
        "native_observations": profile["native_observations"],
    }


def main():
    sealed = check_seals()
    result = validate(
        response=(EVIDENCE / "source-response.json").read_bytes(),
        raw=(SOURCE / "3.4.1-api-changes.wikitext").read_bytes(),
        pin=json.loads((EVIDENCE / "source-pin.json").read_bytes()),
        ledger=json.loads((SOURCE / "3.4.1-page-coverage.json").read_bytes()),
        text=(SOURCE / "3.4.1-api-changes.txt").read_bytes(),
        profile=json.loads((EVIDENCE / "profile-observation.json").read_bytes()),
    )
    print(json.dumps(dict(result, sealed_inputs=sealed), sort_keys=True))


if __name__ == "__main__":
    main()
