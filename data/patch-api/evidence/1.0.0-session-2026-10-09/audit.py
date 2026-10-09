"""Frozen original Retail 1.0.0 literal accounting; SOURCE-only proof."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile

EVIDENCE = Path(__file__).resolve().parent
BASE = "b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())


def validate_source(*, raw=None, response=None):
    raw = (EVIDENCE / "source.wikitext").read_bytes() if raw is None else raw
    response = (
        (EVIDENCE / "source-response.json").read_bytes()
        if response is None
        else response
    )
    pin = read_json("source-pin.json")
    manifest_bytes = (EVIDENCE / "frozen-manifest.json").read_bytes()
    assert (
        digest(manifest_bytes)
        == "b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba"
    ), "manifest hash"
    manifest = json.loads(manifest_bytes)
    assert pin == next(p for p in manifest["pages"] if p["version"] == "1.0.0"), (
        "manifest pin"
    )
    assert (
        digest(raw)
        == pin["wikitext_sha256"]
        == "f87a35e3e2e6033ea2d0d3ed082787dac7b724d8f4dc209fd83e8bffa5f73280"
    ), "raw hash"
    assert (
        digest(response)
        == pin["response_sha256"]
        == "994ead4a255006669a0ffa18dc706e971a6c2b4be6222910b7aa606aedbb2af7"
    ), "response hash"
    assert len(raw) == pin["wikitext_bytes"] == 37407, "raw bytes"
    page = json.loads(response)["query"]["pages"]["67688"]
    (revision,) = page["revisions"]
    assert page["pageid"] == pin["pageid"] == 67688, "pageid"
    assert page["title"] == pin["title"] == "Patch 1.0.0/API changes", "title"
    assert revision["revid"] == pin["revid"] == 5580410, "revid"
    assert revision["timestamp"] == pin["timestamp"] == "2023-10-16T00:50:33Z", (
        "revision timestamp"
    )
    assert revision["slots"]["main"]["*"].encode() == raw, "returned bytes"
    registry_bytes = (EVIDENCE / "frozen-registry.json").read_bytes()
    assert (
        digest(registry_bytes)
        == manifest["registry_sha256"]
        == "e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c"
    ), "registry hash"
    registry = json.loads(registry_bytes)["pages"]
    assert len(registry) == 101 and registry[-1]["version"] == "1.0.0", (
        "registry boundary"
    )
    assert all(pin[k] == v for k, v in registry[-1].items()), "registry identity"
    return raw, pin, registry


def is_retail_successor(version):
    major, minor, _ = map(int, version.split("."))
    return major >= 2 and (major, minor) not in {(2, 5), (3, 4), (4, 4), (5, 5)}


def contract(identity, source_id, missing):
    return {
        "id": identity,
        "source_id": source_id,
        "status": "UNPROVEN",
        "arguments": None,
        "returns": None,
        "defaults": None,
        "events": None,
        "state_transitions": None,
        "security": None,
        "native_equivalence": None,
        "missing_contract": missing,
    }


def references_on_line(number, literal):
    references = []
    patterns = [
        ("internal", r"\[\[([^]|]+)(?:\|([^]]+))?\]\]"),
        ("external", r"\[(https?://[^\s\]]+)\s+([^]]+)\]"),
        ("comparison-citation", r"(?<=compared )(https?://\S+)"),
    ]
    for kind, pattern in patterns:
        for match in re.finditer(pattern, literal):
            references.append(
                {
                    "id": f"link-1.0.0-{number:03}-{len(references) + 1}",
                    "line": number,
                    "literal": match[0],
                    "target": match[1],
                    "display": match[2] if kind != "comparison-citation" else None,
                    "kind": kind,
                    "status": "UNPROVEN",
                    "expanded": False,
                }
            )
    return references


def build():
    raw, pin, registry = validate_source()
    text = raw.decode()
    rows, inventory, signatures, prose, headers, templates, references, contracts = (
        [] for _ in range(8)
    )
    blank_lines = []
    for number, literal in enumerate(text.splitlines(), 1):
        if not literal.strip():
            blank_lines.append(number)
            continue
        row_id = f"raw-1.0.0-{number:03}"
        refs = references_on_line(number, literal)
        references.extend(refs)
        api = re.fullmatch(r": \[\[API ([^]|]+)\|([^]]+)\]\]", literal)
        if api:
            assert api[1] == api[2], "literal target/display identity"
            kind, status = "inventory", "UNPROVEN"
            inventory_id = f"inventory-1.0.0-{number:03}"
            inventory.append(
                {
                    "id": inventory_id,
                    "source_id": row_id,
                    "line": number,
                    "literal": literal,
                    "symbol": api[2],
                    "reference_id": refs[0]["id"],
                    "section": "global-api",
                    "direction": "listed-unspecified",
                    "status": "UNPROVEN",
                }
            )
            signatures.append(
                {
                    "id": f"signature-1.0.0-{number:03}",
                    "inventory_id": inventory_id,
                    "line": number,
                    "symbol": api[2],
                    "literal_signature": None,
                    "arguments": None,
                    "returns": None,
                    "status": "UNPROVEN",
                    "limit": "Bare linked name only; no local signature or linked revision content.",
                }
            )
            contracts.append(
                contract(
                    f"contract-1.0.0-{number:03}-api",
                    inventory_id,
                    "Name-only occurrence. Callable signature, arguments, returns, defaults, events, state transitions, security and historical native behavior unspecified; linked API revision/content absent.",
                )
            )
        elif re.fullmatch(r"==[^=]+==", literal):
            kind, status = "header", "metadata-only"
            headers.append(
                {
                    "id": f"header-1.0.0-{number:03}",
                    "source_id": row_id,
                    "line": number,
                    "literal": literal,
                    "declared_count": None,
                }
            )
        elif literal == "{{apichanges|1.0.0|next=1.1.0}}":
            kind, status = "navigation-template", "metadata-only"
            template_id = f"template-1.0.0-{number:03}"
            templates.append(
                {
                    "id": template_id,
                    "source_id": row_id,
                    "line": number,
                    "literal": literal,
                    "expanded": False,
                    "status": "UNPROVEN",
                    "next": "1.1.0",
                }
            )
            contracts.append(
                contract(
                    f"contract-1.0.0-{number:03}-navigation",
                    template_id,
                    "Navigation template and next=1.1.0 are unexpanded; no successor contract applied or endpoint closure established.",
                )
            )
        elif number in (4, 7):
            kind, status = (
                "archive-claim" if number == 4 else "comparison-comment",
                "UNPROVEN",
            )
            prose_id = f"prose-1.0.0-{number:03}"
            prose.append(
                {
                    "id": prose_id,
                    "source_id": row_id,
                    "line": number,
                    "literal": literal,
                    "kind": kind,
                    "status": status,
                }
            )
            contracts.append(
                contract(
                    f"contract-1.0.0-{number:03}-prose",
                    prose_id,
                    "Published archive/comparison attribution only. Archive 1.1.2.4115 and oldid4864/string dump bytes unexpanded and unavailable in this slice; no 1.0.0 behavioral or native validation.",
                )
            )
            for reference in refs:
                contracts.append(
                    contract(
                        f"contract-{reference['id']}",
                        reference["id"],
                        "External citation body and source identity unexpanded; does not import a FrameXML or global API model contract.",
                    )
                )
        else:
            raise AssertionError(f"unaccounted literal line: {number}")
        rows.append(
            {
                "id": row_id,
                "line": number,
                "literal": literal,
                "kind": kind,
                "status": status,
                "capabilities": [],
            }
        )
    collections = dict(
        inventory=inventory,
        signatures=signatures,
        defaults=[],
        prose=prose,
        headers=headers,
        templates=templates,
        references=references,
        contracts=contracts,
    )
    return dict(
        collections,
        schema="patch-source-accounting/v1",
        patch="1.0.0",
        source=pin,
        base_revision=BASE,
        client_line="original-historical-retail-task-scope",
        literal_client_attribution="comparison comment: wow.exe v1.0.0; no Classic/Era/Forever equivalence",
        source_rows=rows,
        blank_lines=blank_lines,
        measurements={"model": 0, "runtime": 0, "native": 0},
        model_review={
            "result": "no-grounded-behavioral-subset",
            "reason": "854 bare linked names provide no local behavioral signatures/defaults. FrameXML archive and strings-comparison claim provide no inspected bodies. Current API models, guessed aliases or Classic defaults cannot supply historical 1.0.0 semantics.",
            "runtime_changes": False,
            "limit": "SOURCE accounting is meaningful literal coverage, not modeled behavior or native parity; main owns separate target research and full acceptance.",
        },
        history={
            "separate_retail_successor_references": [
                p for p in registry if is_retail_successor(p["version"])
            ],
            "applied_successors": [],
            "queue": "Task-start 1.1.0 active p110-page; 1.3.0/1.4.0/1.5.0 pending, all separately unapplied. This frozen queue is not live sibling status.",
            "queued_successor_references": [
                p
                for p in registry
                if p["version"] in {"1.1.0", "1.3.0", "1.4.0", "1.5.0"}
            ],
            "reference_source_revision": BASE,
            "templates": "Canonical own-base 1.6.0/1.7.0/1.8.0 methodology only; execution receipts not transplanted.",
            "foreign_history_limit": "Era 1.13.x+, TBC Classic 2.5.x, Wrath Classic 3.4.x, Cataclysm Classic 4.4.x, Mists Classic 5.5.x and Forever are separate histories; no linked behavior imported.",
            "registry_endpoint_closes_parent": False,
        },
        totals={
            "physical_lines": len(text.splitlines()),
            "blank_lines": len(blank_lines),
            "nonblank_rows": len(rows),
            "metadata_rows": sum(r["status"] == "metadata-only" for r in rows),
            "unproven_source_rows": sum(r["status"] == "UNPROVEN" for r in rows),
            "numeric_count_headers": sum(
                h["declared_count"] is not None for h in headers
            ),
            "local_literal_signatures": 0,
            "unproven_contracts": len(contracts),
            **{name: len(values) for name, values in collections.items()},
        },
    )


def validate_ledger(ledger, *, expected=None):
    assert ledger == (build() if expected is None else expected), (
        "serialized literal ledger"
    )
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
                "gen_patch_wikitext_register.py",
                "1.0.0",
                str(EVIDENCE / "source.wikitext"),
                "5580410",
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
        "default extract_text bytes"
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
