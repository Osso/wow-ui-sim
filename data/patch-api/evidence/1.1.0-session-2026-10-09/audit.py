"""Frozen original Retail 1.1.0 literal accounting; SOURCE-only proof."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile

EVIDENCE = Path(__file__).resolve().parent
BASE = "b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f"
FIXED_HASHES = {
    "source.wikitext": "d584c6fd1208de8669c85333f63c3da63bebdd21bd0b3a3e3387a36f3ddf8625",
    "source-response.json": "9ad4deff959b721b3561d017b1e3613be2b6b35e7d5d27d195639b96af1b7dde",
    "frozen-manifest.json": "b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba",
    "frozen-registry.json": "e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())


def validate_source(*, raw=None, response=None, overrides=None):
    values = {name: (EVIDENCE / name).read_bytes() for name in FIXED_HASHES}
    values["source-pin.json"] = (EVIDENCE / "source-pin.json").read_bytes()
    values.update(overrides or {})
    if raw is not None:
        values["source.wikitext"] = raw
    if response is not None:
        values["source-response.json"] = response
    for name, expected in FIXED_HASHES.items():
        assert digest(values[name]) == expected, f"frozen hash: {name}"
    original_pin = (EVIDENCE / "source-pin.json").read_bytes()
    assert values["source-pin.json"] == original_pin, "pin bytes"
    pin = json.loads(values["source-pin.json"])
    manifest = json.loads(values["frozen-manifest.json"])
    assert pin == next(p for p in manifest["pages"] if p["version"] == "1.1.0"), (
        "manifest pin"
    )
    raw = values["source.wikitext"]
    assert len(raw) == pin["wikitext_bytes"] == 701, "body bytes"
    assert digest(raw) == pin["wikitext_sha256"], "body pin"
    assert digest(values["source-response.json"]) == pin["response_sha256"], (
        "response pin"
    )
    page = json.loads(values["source-response.json"])["query"]["pages"]["271516"]
    (revision,) = page["revisions"]
    assert page["pageid"] == pin["pageid"] == 271516, "pageid"
    assert page["title"] == pin["title"] == "Patch 1.1.0/API changes", "title"
    assert revision["revid"] == pin["revid"] == 5913060, "revision"
    assert revision["timestamp"] == pin["timestamp"] == "2023-12-27T16:05:57Z", (
        "revision timestamp"
    )
    assert revision["slots"]["main"]["*"].encode() == raw, "returned body bytes"
    registry = json.loads(values["frozen-registry.json"])["pages"]
    assert digest(values["frozen-registry.json"]) == manifest["registry_sha256"], (
        "manifest registry"
    )
    assert len(registry) == 101 and registry[-1]["version"] == "1.0.0", (
        "registry boundary"
    )
    registered = next(p for p in registry if p["version"] == "1.1.0")
    assert all(pin[key] == value for key, value in registered.items()), (
        "registry identity"
    )
    assert not any(p["version"] == "1.2.0" for p in registry), "absent registry 1.2.0"
    return raw, pin, registry


def is_retail_successor(version):
    major, minor, _ = map(int, version.split("."))
    return major >= 2 and (major, minor) not in {(2, 5), (3, 4), (4, 4), (5, 5)}


def unproven_contract(identity, line, missing, symbol=None):
    return {
        "id": identity,
        "line": line,
        "symbol": symbol,
        "status": "UNPROVEN",
        **{
            name: None
            for name in [
                "arguments",
                "returns",
                "defaults",
                "events",
                "state_transitions",
                "security",
                "native_equivalence",
            ]
        },
        "missing_contract": missing,
    }


def collect_occurrences(text):
    rows, inventory, references, signatures, contracts = [], [], [], [], []
    headers, prose, templates = [], [], []
    section, direction = None, None
    for line, literal in enumerate(text.splitlines(), 1):
        if not literal:
            continue
        status = "metadata-only"
        if literal.startswith("{{apichanges|"):
            templates.append(
                {
                    "id": f"template-{line}",
                    "line": line,
                    "literal": literal,
                    "expanded": False,
                    "patch": "1.1.0",
                    "prev": "1.0.0",
                    "next": "1.2.0",
                    "status": "UNPROVEN",
                }
            )
            contracts.append(
                unproven_contract(
                    f"contract-navigation-{line}",
                    line,
                    "Navigation template expansion and next-page source absent; next=1.2.0 is a literal label, not a registry pin or behavioral successor proof.",
                )
            )
        elif literal.startswith("Source: "):
            target = literal.removeprefix("Source: ")
            prose.append(
                {
                    "id": f"prose-{line}",
                    "line": line,
                    "literal": literal,
                    "status": "UNPROVEN",
                    "qualification": "Source attribution only; linked Global_functions oldid4997032 not expanded.",
                }
            )
            references.append(
                {
                    "id": f"link-{line}",
                    "line": line,
                    "literal": target,
                    "target": target,
                    "kind": "external-source",
                    "revision_label": 4997032,
                    "expanded": False,
                    "status": "UNPROVEN",
                }
            )
            contracts.append(
                unproven_contract(
                    f"contract-citation-{line}",
                    line,
                    "Pinned attribution URL is present; cited Global_functions revision4997032 content and historical signatures/defaults are not retained in this slice.",
                )
            )
            status = "UNPROVEN"
        elif match := re.fullmatch(r"(={2,3})([^=]+)\1", literal):
            title = match[2]
            headers.append(
                {
                    "id": f"header-{line}",
                    "line": line,
                    "literal": literal,
                    "title": title,
                    "level": len(match[1]),
                    "numeric_count": None,
                }
            )
            if len(match[1]) == 2:
                section = title
            else:
                direction = title.lower()
        elif match := re.fullmatch(r": (\[\[(API ([^|]+))\|([^]]+)\]\])", literal):
            assert section == "Global API" and direction == "added", (
                "inventory headings"
            )
            assert match[3] == match[4], "literal API target/label identity"
            symbol = match[3]
            inventory.append(
                {
                    "id": f"inventory-{line}",
                    "line": line,
                    "literal": literal,
                    "section": "global-api",
                    "direction": direction,
                    "symbol": symbol,
                    "target": match[2],
                    "display": match[4],
                    "status": "SOURCE-publication-only",
                }
            )
            references.append(
                {
                    "id": f"link-{line}",
                    "line": line,
                    "literal": match[1],
                    "target": match[2],
                    "kind": "api",
                    "expanded": False,
                    "status": "UNPROVEN",
                }
            )
            signatures.append(
                {
                    "id": f"signature-{line}",
                    "line": line,
                    "symbol": symbol,
                    "literal_signature": None,
                    "arguments": None,
                    "returns": None,
                    "status": "UNPROVEN",
                    "limit": "Bare linked function name; parentheses, arity, types and optional/default arguments absent.",
                }
            )
            contracts.append(
                unproven_contract(
                    f"contract-api-{line}",
                    line,
                    f"{symbol} is explicitly listed as added Global API; link body/revision, callable signature, defaults, errors, events, transitions and security are absent. Current namespace/alias/default behavior cannot fill historical contract.",
                    symbol,
                )
            )
            status = "SOURCE-publication-only"
        else:
            raise AssertionError(f"unaccounted literal line {line}: {literal}")
        rows.append(
            {
                "id": f"raw-{line}",
                "line": line,
                "literal": literal,
                "status": status,
                "capabilities": [],
            }
        )
    return {
        "source_rows": rows,
        "inventory": inventory,
        "references": references,
        "signatures": signatures,
        "contracts": contracts,
        "headers": headers,
        "prose": prose,
        "templates": templates,
        "defaults": [],
        "count_claims": [],
    }


def build():
    raw, pin, registry = validate_source()
    collections = collect_occurrences(raw.decode())
    return {
        "schema": "patch-source-accounting/v1",
        "patch": "1.1.0",
        "source": pin,
        "base_revision": BASE,
        "client_line": "original-historical-retail-task-scope",
        "literal_client_attribution": None,
        **collections,
        "measurements": {"model": 0, "runtime": 0, "native": 0},
        "model_review": {
            "result": "no-grounded-historical-behavioral-subset",
            "candidate_report": "local-candidates.json records own-base current declarations/references only; GetCVarDefault, GetAuctionItemLink and UnitRangedAttack have current implementations. Global summon stubs and C_SummonInfo are not historical global contracts. Name-only publication cannot ground arguments/defaults or native behavior; no runtime change/test credit.",
            "current_candidate": "GetCVarDefault genuinely reads CVarStorage defaults/registered_defaults separately from overrides. Existing register_cvar_numeric_default_preserves_existing_override_and_first_default observes override1.25/default0 and repeated registration6 preserving both; grounded current behavior, not executed in this bounded Python slice. Main owns independent execution.",
            "rejected_candidate": "UnitRangedAttack derives max(level*5,0) and always returns modifier0; no independent ranged-skill/modifier state. Existing Mists numeric/positive test does not establish meaningful ranged-attack transitions; no credit.",
            "limit": "All 11 historical API contracts remain UNPROVEN. Current Retail, Era and Forever observations would be separate evidence, not 1.1.0 parity.",
        },
        "history": {
            "separate_retail_successor_references": [
                p for p in registry if is_retail_successor(p["version"])
            ],
            "queued_successor_references": [
                p for p in registry if p["version"] in {"1.3.0", "1.4.0", "1.5.0"}
            ],
            "queued_status": "1.3.0 active p130-page; 1.4.0/1.5.0 pending, all separately unapplied",
            "navigation_only_next": "1.2.0; no registry entry, no invented pin",
            "applied_successors": [],
            "reference_source_revision": BASE,
            "templates": "Canonical own-base 1.6.0/1.7.0/1.8.0 SOURCE methodology only; no imported proof/contracts/defaults.",
            "foreign_history_limit": "Classic Era1.13.x+, TBC2.5.x, Wrath3.4.x, Cataclysm4.4.x, Mists5.5.x and Forever1.60.1 remain distinct from original historical Retail. Links/navigation never import behavior.",
        },
        "totals": {
            "physical_lines": len(raw.decode().splitlines()),
            "nonblank_rows": len(collections["source_rows"]),
            **{
                name: len(values)
                for name, values in collections.items()
                if name != "source_rows"
            },
            "metadata_rows": sum(
                r["status"] == "metadata-only" for r in collections["source_rows"]
            ),
            "unproven_source_rows": sum(
                r["status"] == "UNPROVEN" for r in collections["source_rows"]
            ),
            "unproven_contracts": sum(
                r["status"] == "UNPROVEN" for r in collections["contracts"]
            ),
        },
    }


def validate_ledger(ledger):
    assert ledger == build(), "serialized literal ledger"
    return ledger["totals"]


def load_historical(name):
    spec = importlib.util.spec_from_file_location(
        name, EVIDENCE / "historical-tools" / f"{name}.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def replay_defaults():
    generator = load_historical("gen_patch_wikitext_register")
    with tempfile.TemporaryDirectory(dir=EVIDENCE) as directory:
        output = Path(directory) / "register.json"
        previous = sys.argv
        try:
            sys.argv = [
                str(EVIDENCE / "historical-tools/gen_patch_wikitext_register.py"),
                "1.1.0",
                str(EVIDENCE / "source.wikitext"),
                "5913060",
                str(output),
            ]
            generator.main()
        finally:
            sys.argv = previous
        assert (
            output.read_bytes() == (EVIDENCE / "default-register.json").read_bytes()
        ), "default generator bytes"
    extractor = load_historical("extract_patch_non_inventory")
    actual = extractor.extract_text((EVIDENCE / "source.wikitext").read_text()).encode()
    assert actual == (EVIDENCE / "default-extract.txt").read_bytes(), (
        "default extractor bytes"
    )
    errors = []
    for tool, function, argument in [
        (generator, "parse_symbol", "not a reference"),
        (extractor, "extract_text", None),
    ]:
        try:
            getattr(tool, function)(argument)
        except Exception as error:
            errors.append(
                {
                    "function": function,
                    "argument": argument,
                    "type": type(error).__name__,
                    "message": str(error),
                }
            )
        else:
            raise AssertionError("historical error unexpectedly accepted")
    assert errors == read_json("default-errors.json"), "default error replay"


def check_seals():
    seals = read_json("seals.json")
    for name, expected in seals.items():
        assert digest((EVIDENCE / name).read_bytes()) == expected, f"seal: {name}"
    return len(seals)


def main():
    if sys.argv[1:] == ["capture"]:
        destination = EVIDENCE / "ledger.json"
        assert not destination.exists(), "refuse ledger overwrite"
        destination.write_text(json.dumps(build(), indent=2) + "\n")
    else:
        seals = check_seals()
        totals = validate_ledger(read_json("ledger.json"))
        replay_defaults()
        print(
            json.dumps(
                {
                    "scope": "SOURCE-only historical replay",
                    "seals": seals,
                    "totals": totals,
                },
                indent=2,
            )
        )


if __name__ == "__main__":
    main()
