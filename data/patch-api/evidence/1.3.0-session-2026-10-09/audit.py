"""Frozen historical Retail 1.3.0 literal declaration accounting; SOURCE-only proof."""

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


def validate_source(*, raw=None, response=None, manifest_bytes=None, registry_bytes=None):
    raw = (EVIDENCE / "source.wikitext").read_bytes() if raw is None else raw
    response = (
        (EVIDENCE / "source-response.json").read_bytes()
        if response is None
        else response
    )
    pin = read_json("source-pin.json")
    manifest_bytes = (
        (EVIDENCE / "frozen-manifest.json").read_bytes()
        if manifest_bytes is None
        else manifest_bytes
    )
    assert (
        digest(manifest_bytes)
        == "b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba"
    ), "manifest hash"
    manifest = json.loads(manifest_bytes)
    assert pin == next(p for p in manifest["pages"] if p["version"] == "1.3.0"), (
        "manifest pin"
    )
    assert (
        digest(raw)
        == pin["wikitext_sha256"]
        == "8c9dd641e26d95da146202fbe8bb82999d22d6536e139d761390fce18f7145fd"
    ), "raw hash"
    assert (
        digest(response)
        == pin["response_sha256"]
        == "a821d5a31396c9ad5bed90799f63cee7921a8460b6982477a63856338e6f55c4"
    ), "response hash"
    assert len(raw) == pin["wikitext_bytes"] == 1364, "raw bytes"
    page = json.loads(response)["query"]["pages"]["350208"]
    (revision,) = page["revisions"]
    assert page["pageid"] == pin["pageid"] == 350208, "pageid"
    assert page["title"] == pin["title"] == "Patch 1.3.0/API changes", "title"
    assert revision["revid"] == pin["revid"] == 3376287, "revid"
    assert revision["timestamp"] == pin["timestamp"] == "2021-04-23T02:34:08Z", (
        "revision timestamp"
    )
    assert revision["slots"]["main"]["*"].encode() == raw, "returned bytes"
    registry_bytes = (
        (EVIDENCE / "frozen-registry.json").read_bytes()
        if registry_bytes is None
        else registry_bytes
    )
    assert (
        digest(registry_bytes)
        == manifest["registry_sha256"]
        == "e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c"
    ), "registry hash"
    registry = json.loads(registry_bytes)["pages"]
    assert len(registry) == 101 and registry[-1]["version"] == "1.0.0", (
        "registry boundary"
    )
    registered = next(p for p in registry if p["version"] == "1.3.0")
    assert all(pin[k] == v for k, v in registered.items()), "registry identity"
    return raw, pin, registry


def is_retail_successor(version):
    major, minor, _ = map(int, version.split("."))
    return major >= 2 and (major, minor) not in {(2, 5), (3, 4), (4, 4), (5, 5)}


def append_literal(records, category, line, literal, **fields):
    row = dict(id=f"{category}-1.3.0-{len(records) + 1:03}", line=line, literal=literal, **fields)
    records.append(row)
    return row


def collect_boundaries(text):
    groups = {name: [] for name in [
        "source_rows", "inventory", "signatures", "prose", "headers",
        "references", "templates", "defaults", "count_claims",
    ]}
    section, category = None, None
    for line, literal in enumerate(text.splitlines(), 1):
        kind = "blank" if not literal.strip() else "declaration"
        if literal.startswith("== "):
            section, category = literal.strip("= "), None
            kind = "section-header"
        elif literal.startswith("; "):
            category = literal[2:]
            kind = "category-header"
        elif line == 1:
            kind = "metadata-template"
        append_literal(groups["source_rows"], "raw", line, literal, kind=kind)
        if kind in {"section-header", "category-header"}:
            append_literal(groups["headers"], "header", line, literal, kind=kind,
                           numeric_claim=None)
        for match in re.finditer(r"\[\[[^\]]+\]\]|\[https?://[^\]]+\]", literal):
            value = match[0]
            target = value[2:-2] if value.startswith("[[") else value[1:-1].split(" ", 1)[0]
            append_literal(groups["references"], "link", line, value, target=target,
                           column=match.start() + 1, expanded=False, status="UNPROVEN")
        for match in re.finditer(r"\{\{([^{}]+)\}\}", literal):
            pieces = match[1].split("|")
            append_literal(groups["templates"], "template", line, match[0],
                           column=match.start() + 1, name=pieces[0],
                           parameters=pieces[1:], expanded=False, status="UNPROVEN")
            if pieces[0] == "api":
                declaration = pieces[-1]
                if section == "Changed":
                    append_literal(groups["inventory"], "api", line,
                                   literal.split(" became ")[0], change=section,
                                   category="Widget API", template_kind="w", rename_role="old")
                template_kind = next((p[2:] for p in pieces[1:] if p.startswith("t=")), None)
                append_literal(groups["inventory"], "api", line, declaration,
                               change=section, category=category,
                               template_kind=template_kind,
                               rename_role="new" if section == "Changed" else None)
        if line == 1:
            append_literal(groups["prose"], "prose", line, literal.split("|misc=", 1)[1][:-2],
                           kind="attribution")
        elif " - Fires " in literal:
            append_literal(groups["prose"], "prose", line, literal.split(" - ", 1)[1],
                           kind="event-trigger")
        elif " became " in literal:
            append_literal(groups["prose"], "prose", line, literal, kind="rename")
    for row in groups["inventory"]:
        match = re.fullmatch(r"([^()]+)\(([^()]*)\)", row["literal"])
        append_literal(groups["signatures"], "signature", row["line"], row["literal"],
                       inventory_id=row["id"],
                       symbol_literal=match[1] if match else row["literal"],
                       argument_literal=match[2] if match else None,
                       returns=None, defaults=None, complete_contract=False)
    return groups


def contract_gap(group, row):
    if group == "inventory":
        if row["template_kind"] == "e":
            return "Event name and prose recovery trigger only. Payload, dispatch ordering, recovery threshold, delivery guarantees, security and native behavior unspecified."
        if row["change"] == "Removed":
            return "Removal-name claim only. Historical arguments, returns, defaults, removal timing, errors, consumers, replacement and native behavior unspecified; no current retirement authorized."
        if row["change"] == "Changed":
            return "Old/new rename spelling only. Argument/return parity, retained alias, defaults, transition timing, validation, security and native behavior unspecified."
        return "Callable spelling and literal parenthesized arguments only. Types, optionality, enforced arity, return values, defaults, validation/errors, events, state lifecycle, security and native behavior unspecified. Empty parentheses do not prove a complete zero-argument contract."
    if group == "prose":
        return {
            "attribution": "Adaptation attribution only. Linked forum/archive authorship, full content, revision and applicability unexpanded/unproven.",
            "event-trigger": "Published recovery trigger only. No allocation limit, recovery protocol, event payload, ordering, listener semantics or native trace supplied.",
            "rename": "Published CanSave to CanSaveTabardNow rename only. No alias retention, behavior parity, default or native proof supplied.",
        }[row["kind"]]
    if group == "headers":
        return "Literal section/category organization only. No numeric count claim or inherited runtime/default/behavior contract."
    if group == "references":
        return "Literal link target only. Target revision/content and any contract remain unexpanded in this bounded frozen slice."
    return "Literal template name/parameters only. Expansion unavailable; API/navigation templates do not supply linked signatures/defaults or successor coverage."


def build():
    raw, pin, registry = validate_source()
    groups = collect_boundaries(raw.decode())
    contracts = []
    for group in ["inventory", "prose", "headers", "references", "templates"]:
        for row in groups[group]:
            contracts.append(dict(
                id=f"contract-{row['id']}", boundary_id=row["id"], line=row["line"],
                status="UNPROVEN", missing_contract=contract_gap(group, row),
                arguments=None, returns=None, defaults=None, events=None,
                state_transitions=None, security=None, native_equivalence=None,
            ))
    totals = {name: len(rows) for name, rows in groups.items()}
    totals.update(
        physical_lines=len(raw.decode().splitlines()),
        nonblank_rows=sum(bool(r["literal"].strip()) for r in groups["source_rows"]),
        explicit_argument_lists=sum(r["argument_literal"] is not None for r in groups["signatures"]),
        unproven_contracts=len(contracts),
    )
    return dict(
        groups, schema="patch-source-accounting/v1", patch="1.3.0", source=pin,
        base_revision=BASE, client_line="original-historical-retail-task-scope",
        literal_client_attribution=None, contracts=contracts, totals=totals,
        measurements={"model": 0, "runtime": 0, "native": 0},
        model_review={
            "grounded_candidates": [
                "Quest-watch membership/index queries: AddQuestWatch, RemoveQuestWatch, GetNumQuestWatches, GetQuestIndexForWatch, IsQuestWatched",
                "Four action-bar toggle inputs: SetActionBarToggles(show1,show2,show3,show4), GetActionBarToggles()",
                "Frame movable/resizable query/set flag pairs with named setter inputs",
                "TabardModel:CanSave() became TabardModel:CanSaveTabardNow()",
            ],
            "result": "candidate-identities-only; no model/runtime/native test performed",
            "reason": "Returns, defaults, validation and lifecycle absent. MEMORY_RECOVERED supplies a trigger description, not a recovery protocol. Main owns grounded-contract research and meaningful behavior acceptance; no invented modern aliases/defaults or runtime changes in this SOURCE slice.",
        },
        history={
            "applied_successors": [],
            "queued_successor_references": [p for p in registry if p["version"] in {"1.4.0", "1.5.0"}],
            "queue": "1.4.0 active p140-page, 1.5.0 p150-page separate/queued; main integrates newer pages first. No branch status or successor content proof claimed here.",
            "separate_retail_successor_references": [p for p in registry if is_retail_successor(p["version"])],
            "foreign_history_limit": "Original Retail is not Classic Era, TBC/Wrath/Cataclysm/Mists Classic or Forever. Foreign history/navigation cannot supply defaults, removal/alias proof or supersession.",
        },
    )


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
                "gen_patch_wikitext_register.py",
                "1.3.0",
                str(EVIDENCE / "source.wikitext"),
                "3376287",
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
