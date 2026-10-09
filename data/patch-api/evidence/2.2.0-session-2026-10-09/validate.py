#!/usr/bin/env python3
"""Own frozen historical 2007 retail SOURCE replay; no runtime or Git inputs."""

from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

EVIDENCE = Path(__file__).resolve().parent
BASE = "dd8ca0f19e9f0172851116a7bfae43792a1b78b4"
RAW_HASH = "8cbaf1b10b334913ff4a0f2c4ede58fad8b862c7f01f78d32088e5b8fe06590d"
RESPONSE_HASH = "b8f11baba49fa1fde43e31089b3da183c8c547475c97bdd4769493f51e10a3f4"
REGISTRY_HASH = "e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c"
LINK = re.compile(r"\[\[(API [^|]+)\|(.*?)\]\]")
LIMIT = {
    "status": "UNPROVEN",
    "capabilities": [],
    "note": "Literal source only; linked member contracts unexpanded; no runtime/native proof.",
}
# Reviewed bare occurrences; no inferred pickup APIs or enumerated event members.
BARE = {
    44: [("pitchLimit", "cvar-setting", None)],
    57: [("GetItemInfo", "global-api", 'GetItemInfo("item:N")')],
    62: [
        ("OnShow", "widget-script", "OnShow()"),
        ("OnHide", "widget-script", "OnHide()"),
    ],
    63: [("OnUpdate", "widget-script", None), ("OnEnter", "widget-script", None)],
    67: [("SecureButton_GetModifiedAttrbute", "secure-helper", None)],
    73: [("GetPoint", "widget-method", "GetPoint()")],
    76: [("UNIT_SPELLCAST_*", "event-family", None)],
    82: [
        ("RegisterStateDriver", "global-api", None),
        (
            "RegisterStateDriver",
            "global-api",
            'RegisterStateDriver(MyBadTargetFrame, "visibility", "[exists,harm] show; hide")',
        ),
    ],
    83: [("SecureStateHeader", "secure-template", None)],
}
RETURN_FRAGMENTS = {
    17: "count",
    18: "action",
    20: "binding",
    21: "active",
    66: "left, right, top, bottom",
    67: "value",
}
QUEUE = ["2.3.0", "2.4.0", "2.4.2", "3.0.x"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())


def load_tool(name):
    spec = importlib.util.spec_from_file_location(
        name, EVIDENCE / f"historical-{name}.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_identity():
    raw = (EVIDENCE / "historical-source.wikitext").read_bytes()
    response = (EVIDENCE / "source-response.json").read_bytes()
    assert digest(raw) == RAW_HASH and len(raw) == 7518, "source hash/bytes"
    assert digest(response) == RESPONSE_HASH, "response hash"
    page = json.loads(response)["query"]["pages"]["184149"]
    (revision,) = page["revisions"]
    assert (
        page["pageid"],
        page["title"],
        revision["revid"],
        revision["timestamp"],
    ) == (184149, "Patch 2.2.0/API changes", 6428980, "2025-08-03T13:04:54Z"), (
        "exact revision identity"
    )
    assert revision["slots"]["main"]["*"].encode() == raw, "returned body"
    registry_bytes = (EVIDENCE / "registry.json").read_bytes()
    registry = json.loads(registry_bytes)
    manifest = read_json("cache-manifest.json")
    assert digest(registry_bytes) == REGISTRY_HASH == manifest["registry_sha256"], (
        "registry hash"
    )
    assert (
        len(registry["pages"]) == 101 and registry["pages"][-1]["version"] == "1.0.0"
    ), "registry endpoint"
    source = next(p for p in registry["pages"] if p["version"] == "2.2.0")
    pin = read_json("source-pin.json")
    assert all(source[k] == pin[k] for k in source), "registry source identity"
    assert pin == next(p for p in manifest["pages"] if p["version"] == "2.2.0"), (
        "manifest source pin"
    )
    assert (
        pin["response_sha256"] == RESPONSE_HASH and pin["wikitext_sha256"] == RAW_HASH
    ), "pin hashes"
    assert pin["wikitext_bytes"] == len(raw) and len(manifest["pages"]) == 65, (
        "manifest accounting"
    )
    return raw.decode(), pin


def direction(number):
    if number in (7, 17, 18, 19, 20, 21, 65, 66):
        return "added"
    if number == 41:
        return "replaced"
    if number in (44, 62, 63, 67, 76, 82, 83):
        return "changed"
    return "context"


def inventory_row(number, ordinal, symbol, kind, line, fragment, linked_page=None):
    row = dict(
        LIMIT,
        id=f"source-{number:03}-{ordinal:02}",
        wikitext_line=number,
        symbol=symbol,
        kind=kind,
        direction=direction(number),
        source_text=line,
        linked_page=linked_page,
        fragment=fragment,
    )
    if kind == "cvar-setting":
        row["default"] = None
    if kind == "event-family":
        row["expanded_members"] = None
        row["partial_payload"] = {"arg2": "spell name", "arg3": "spell rank"}
    return row


def derive_inventory(raw):
    rows = []
    for number, line in enumerate(raw.splitlines(), 1):
        ordinal = 0
        for match in LINK.finditer(line):
            ordinal += 1
            fragment = match[2]
            symbol = fragment.split("(")[0]
            kind = "widget-method" if symbol.startswith("Frame:") else "global-api"
            rows.append(
                inventory_row(number, ordinal, symbol, kind, line, fragment, match[1])
            )
        for symbol, kind, fragment in BARE.get(number, []):
            ordinal += 1
            rows.append(inventory_row(number, ordinal, symbol, kind, line, fragment))
    return rows


def derive_signatures(inventory):
    rows = []
    for entry in inventory:
        if entry["kind"] not in (
            "global-api",
            "widget-method",
            "widget-script",
            "secure-helper",
        ):
            continue
        number = entry["wikitext_line"]
        rows.append(
            dict(
                LIMIT,
                inventory_id=entry["id"],
                symbol=entry["symbol"],
                wikitext_line=number,
                fragment=entry["fragment"],
                return_fragment=(
                    RETURN_FRAGMENTS.get(number) if entry["linked_page"] else None
                ),
                arguments=None,
                returns=None,
                partial_input=(
                    "self; unspecified affected OnEnter functions"
                    if entry["symbol"] == "OnEnter"
                    else None
                ),
                complete_contract=False,
            )
        )
    return rows


def derive_rows(raw, inventory):
    headings, prose, source, examples = [], [], [], []
    section = None
    byline = {}
    for row in inventory:
        byline.setdefault(row["wikitext_line"], []).append(row["id"])
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        heading = re.fullmatch(r"==\s*([^=]+?)\s*==", line)
        if heading:
            section = heading[1]
            headings.append(
                {
                    "wikitext_line": number,
                    "title": section,
                    "source_text": line,
                    "declared_count": None,
                }
            )
        metadata = bool(heading) or number in (1, 16, 86)
        row = {
            "source_id": f"raw-{number:03}",
            "wikitext_line": number,
            "section": section,
            "source_text": line,
            "inventory_ids": byline.get(number, []),
            "status": "metadata-only" if metadata else "UNPROVEN",
            "capabilities": [],
        }
        source.append(row)
        if not metadata:
            prose.append(
                dict(
                    row,
                    note="Full literal claim/example; no runtime/native/performance proof.",
                )
            )
        if number == 13 or 29 <= number <= 39:
            match = re.search(
                r'ModifiedClick action="([A-Z]+)" default="([A-Z0-9-]+)"', line
            )
            assert match, "modified click XML example"
            examples.append(
                dict(
                    LIMIT,
                    source_id=row["source_id"],
                    action=match[1],
                    default=match[2],
                    source_text=line,
                )
            )
    return headings, prose, source, examples


def build_ledger():
    raw, pin = validate_identity()
    extractor = load_tool("extract_patch_non_inventory")
    text = extractor.extract_text(raw)
    inventory = derive_inventory(raw)
    headers, prose, rows, examples = derive_rows(raw, inventory)
    return {
        "schema": "patch-source-accounting/v1",
        "patch": "2.2.0",
        "source": pin,
        "code_revision": BASE,
        "client_line": "historical-retail",
        "source_era": "2007 retail",
        "scope": "SOURCE-only",
        "later_registers": [],
        "successor_queue": QUEUE,
        "runtime_observations": 0,
        "native_observations": 0,
        "model_credit": 0,
        "inventory_rows": inventory,
        "signature_ledger": derive_signatures(inventory),
        "headers": headers,
        "header_counts": [],
        "prose_ledger": prose,
        "source_rows": rows,
        "modified_click_examples": examples,
        "extractor_flags": [],
        "extract_sha256": digest(text.encode()),
        "extract_rows": [
            {
                "source_id": f"extract-{n:03}",
                "line": n,
                "text": line,
                "status": "UNPROVEN",
                "capabilities": [],
            }
            for n, line in enumerate(text.splitlines(), 1)
            if line.strip()
        ],
        "limits": [
            "No linked-page expansion or concrete UNIT_SPELLCAST family enumeration.",
            "Split SaveBindings call fragments are not reconstructed member signatures.",
            "Empty parentheses in source are not zero-arity/type/return proofs.",
            "Source replacement is not a current-retail removal or a supersession proof.",
            "No explicit console/slash command inventory; macro syntax is prose, not commands.",
            "Default extractor preserves optional IsModifiedClick link markup verbatim.",
            "2007 historical retail is neither Classic 2.5/Wrath 3.4/Era nor modern native retail.",
        ],
    }


def validate(ledger):
    expected = build_ledger()
    assert ledger == expected, (
        "complete literal source/signature/prose/header/client accounting"
    )
    text = load_tool("extract_patch_non_inventory").extract_text(
        (EVIDENCE / "historical-source.wikitext").read_text()
    )
    assert (EVIDENCE / "historical-extract.txt").read_bytes() == text.encode(), (
        "extract reproduction"
    )
    return {
        "inventory_occurrences": len(ledger["inventory_rows"]),
        "kinds": dict(Counter(r["kind"] for r in ledger["inventory_rows"])),
        "directions": dict(Counter(r["direction"] for r in ledger["inventory_rows"])),
        "signature_limits": len(ledger["signature_ledger"]),
        "literal_signature_fragments": sum(
            r["fragment"] is not None for r in ledger["signature_ledger"]
        ),
        "unspecified_signature_fragments": sum(
            r["fragment"] is None for r in ledger["signature_ledger"]
        ),
        "raw_nonblank_rows": len(ledger["source_rows"]),
        "raw_statuses": dict(Counter(r["status"] for r in ledger["source_rows"])),
        "extract_nonblank_rows": len(ledger["extract_rows"]),
        "prose_limits": len(ledger["prose_ledger"]),
        "headers": len(ledger["headers"]),
        "header_counts": ledger["header_counts"],
        "modified_click_examples": len(ledger["modified_click_examples"]),
        "registry_pages": 101,
        "registry_endpoint": "1.0.0",
        "runtime_observations": 0,
        "native_observations": 0,
        "model_credit": 0,
    }


def check_seals():
    seals = read_json("seals.json")
    for name, expected in seals.items():
        assert digest((EVIDENCE / name).read_bytes()) == expected, (
            f"sealed input: {name}"
        )
    return len(seals)


def main():
    sealed = check_seals()
    result = validate(read_json("historical-page-coverage.json"))
    print(json.dumps(dict(result, sealed_inputs=sealed), sort_keys=True))


if __name__ == "__main__":
    main()
