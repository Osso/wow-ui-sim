"""Frozen original Retail 1.4.0 literal accounting; SOURCE proof only."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile

EVIDENCE = Path(__file__).resolve().parent
BASE = "b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f"
MANIFEST_HASH = "b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba"
REGISTRY_HASH = "e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c"
RAW_HASH = "94fc69b008e0531c8630274dcf0ea03864fa2a3d5a8faab00fb93b40b45e9f52"
RESPONSE_HASH = "f9125fedab7a00d8a65b1a1b3ca69fe855d178982d31dc7ca47640be51d7f5fa"
API = re.compile(r"\{\{api\|([^{}]+)\}\}")
TEMPLATE = re.compile(r"\{\{([^{}]+)\}\}")
LINK = re.compile(r"\[\[([^\]]+)\]\]|\[(https?://\S+) ([^\]]+)\]")

# Limits describe only missing parts; exact sourced prose stays beside each limit.
LIMITS = {
    "AcceptBattlefieldPort": "One accept boolean is literal: true accepts porting; false rejects/leaves queue. Queue identity, invitation prerequisites, completion timing, port side effects, return values and event payload/order absent. Current two-argument registration cannot establish this one-argument historical contract.",
    "GetBattlefieldEstimatedWaitTime": "Estimated entry wait described; units, return count/types, queue identity, no-queue result, update timing and error conditions absent.",
    "GetBattlefieldInfo": "Index named; index domain and detailed information return fields/types/order, absence and errors absent.",
    "GetBattlefieldInstanceExpiration": "Shutdown timer described; units, return count/types, no-instance result and timer evolution absent.",
    "GetBattlefieldInstanceInfo": "Index named; instance index domain, information fields/types/order, absent instance and errors absent.",
    "GetBattlefieldPortExpiration": "Milliseconds before port expiry explicitly stated; return shape, invitation/expired/no-port cases, clock evolution and error behavior absent.",
    "GetBattlefieldStatus": "Status information described; status values, fields/types/order and queue/port lifecycle absent.",
    "ShowBattlefieldList": "Request for available instances described; request prerequisites, asynchronous response, event/payload ordering and returned values absent.",
    "GetPVPLastWeekStats": "Last-week contribution statistics described; return fields/types/order, week boundary/timezone, reset behavior and unavailable results absent.",
    "GetPVPLifetimeStats": "Lifetime contribution statistics described; return fields/types/order, accumulation and unavailable results absent.",
    "GetPVPRankInfo": "Literal rank and optional unit; second argument not used by UI is an observation, not a default. Rank domain, unit resolution, omission default and information fields/types/order absent.",
    "GetPVPSessionStats": "Session contribution statistics described; session boundary, return fields/types/order, reset/update and absent data absent.",
    "GetPVPYesterdayStats": "Yesterday contribution statistics described; day boundary/timezone, return fields/types/order and absent data absent.",
    "UnitPVPRank": "Literal quoted unit token and rank information; unit domain, identity resolution, return fields/types/order and absent-unit behavior absent.",
    "ClearInspectPlayer": "Clear currently inspected player, called when inspect frame hides; state ownership, outstanding-request cancellation, completion/event ordering and return values absent.",
    "GetInspectHonorData": "Inspected-unit honor information described; fields/types/order, inspected identity, request completion and unavailable result absent.",
    "HasInspectHonorData": "Whether inspected-unit honor data has loaded described; return type/count, stale/cancelled requests, identity changes and availability lifecycle absent.",
    "RequestInspectHonorData": "Honor-data request for inspected unit described; prerequisites, server completion, INSPECT_HONOR_UPDATE relationship/payload/order, errors and return values absent.",
    "GetWeaponEnchantInfo": "Enchantments and charges mentioned; weapon slot identities, tuple shape/types/order, durations/units, absent enchantment and update lifecycle absent.",
    "GetCurrentMultisampleFormat": "Current antialias format described; format identity, return shape/types/order and initial/default selection absent.",
    "GetMultisampleFormats": "Available antialias formats described; format catalogue, return shape/types/order and device dependence absent.",
    "SetEuropeanNumbers": "Ellipsis is literal; source explicitly says unknown purpose. Argument schema, defaults, numeric formatting rules, scope, side effects and return values absent.",
    "SetMultisampleFormat": "Format index named; valid indices, device catalogue, invalid input, persistence, rendering effects and return values absent.",
    "TogglePVP": "Replacement of EnablePVP and toggling PvP status stated; initial state, eligibility, transitions/timers, synchronization/events and return values absent. No alias or callable removal rule inferred.",
}


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
    manifest_bytes = (EVIDENCE / "frozen-manifest.json").read_bytes()
    assert digest(manifest_bytes) == MANIFEST_HASH, "manifest hash"
    manifest = json.loads(manifest_bytes)
    pin_bytes = (EVIDENCE / "source-pin.json").read_bytes()
    pin = json.loads(pin_bytes)
    assert pin_bytes == (json.dumps(pin, indent=2) + "\n").encode(), "pin serialization"
    assert pin == next(p for p in manifest["pages"] if p["version"] == "1.4.0"), (
        "manifest pin"
    )
    assert digest(raw) == pin["wikitext_sha256"] == RAW_HASH, "raw hash"
    assert len(raw) == pin["wikitext_bytes"] == 2670, "raw bytes"
    assert digest(response) == pin["response_sha256"] == RESPONSE_HASH, "response hash"
    page = json.loads(response)["query"]["pages"]["316397"]
    (revision,) = page["revisions"]
    assert page["pageid"] == pin["pageid"] == 316397, "pageid"
    assert page["title"] == pin["title"] == "Patch 1.4.0/API changes", "title"
    assert revision["revid"] == pin["revid"] == 3052898, "revid"
    assert revision["timestamp"] == pin["timestamp"] == "2021-04-23T23:31:01Z", (
        "timestamp"
    )
    assert revision["slots"]["main"]["*"].encode() == raw, "returned bytes"
    registry_bytes = (EVIDENCE / "frozen-registry.json").read_bytes()
    assert digest(registry_bytes) == manifest["registry_sha256"] == REGISTRY_HASH, (
        "registry hash"
    )
    registry = json.loads(registry_bytes)["pages"]
    assert len(registry) == 101 and registry[-1]["version"] == "1.0.0", (
        "registry boundary"
    )
    registered = next(p for p in registry if p["version"] == "1.4.0")
    assert all(pin[k] == v for k, v in registered.items()), "registry identity"
    return raw, pin, registry


def is_retail_successor(version):
    major, minor, _ = map(int, version.split("."))
    return major >= 2 and (major, minor) not in {(2, 5), (3, 4), (4, 4), (5, 5)}


def record(category, line, literal, **fields):
    return dict(id=f"{category}-1.4.0-{line:03d}", line=line, literal=literal, **fields)


def signature(line, literal, role):
    match = re.fullmatch(r"(\w+)\((.*)\)", literal)
    assert match, "literal callable signature"
    return record(
        "signature",
        line,
        literal,
        symbol=match[1],
        arguments_literal=match[2],
        role=role,
        optional_argument_default=None,
        returns=None,
        status="UNPROVEN",
    )


def contract(line, symbol, claim, missing, *, suffix=""):
    return record(
        "contract" + suffix,
        line,
        claim,
        symbol=symbol,
        source_claim=claim,
        missing_contract=missing,
        defaults=None,
        security=None,
        native_equivalence=None,
        status="UNPROVEN",
    )


def collect_declaration(number, line, section, group, result):
    match = API.search(line)
    assert match, "declaration template"
    parts = match[1].split("|")
    literal = parts[-1]
    event = parts[0] == "t=e"
    symbol = literal if event else literal.split("(", 1)[0]
    claim = line[match.end() :].strip().removeprefix("-").strip()
    result["inventory"].append(
        record(
            "inventory",
            number,
            match[0],
            symbol=symbol,
            literal_signature=literal,
            kind="event" if event else "global-api",
            direction=section,
            group=group,
            status="UNPROVEN",
        )
    )
    if event:
        result["contracts"].append(
            contract(
                number,
                symbol,
                line,
                "Event name and RegisterEvent example only. Trigger conditions, argument count/types/payload, unit context, ordering and historical delivery behavior absent.",
            )
        )
        return
    result["signatures"].append(signature(number, literal, "declaration"))
    result["prose"].append(
        record("prose", number, claim, symbol=symbol, status="UNPROVEN")
    )
    result["contracts"].append(contract(number, symbol, claim, LIMITS[symbol]))
    replaced = re.search(r"<code>(EnablePVP\(\))</code>", claim)
    if replaced:
        result["signatures"].append(
            signature(number, replaced[1], "replaced-prose-reference")
        )
        result["contracts"].append(
            contract(
                number,
                "EnablePVP",
                claim,
                "Replacement claim is literal, not a frozen runtime removal test or alias requirement. Prior callable behavior, retirement timing, compatibility and native equivalence absent.",
                suffix="-replaced",
            )
        )


def collect_references(number, line, result):
    for i, match in enumerate(LINK.finditer(line), 1):
        target = match[1] or match[2]
        result["references"].append(
            record(
                f"link-{i}",
                number,
                match[0],
                target=target,
                label=match[3],
                kind="internal" if match[1] else "external",
                expanded=False,
                status="UNPROVEN",
            )
        )
        result["contracts"].append(
            contract(
                number,
                None,
                match[0],
                "Linked content/revision is not frozen in this slice. Attribution or archive provenance does not import callable contracts. Main owns primary-target research; redirect targets, if encountered there, remain unexpanded here.",
                suffix=f"-link-{i}",
            )
        )
    for i, match in enumerate(TEMPLATE.finditer(line), 1):
        parts = match[1].split("|")
        result["templates"].append(
            record(
                f"template-{i}",
                number,
                match[0],
                name=parts[0],
                parameters_literal=parts[1:],
                expanded=False,
                status="metadata-only" if parts[0] == "apichanges" else "UNPROVEN",
            )
        )


def build():
    raw, pin, registry = validate_source()
    text = raw.decode()
    result = {
        name: []
        for name in [
            "source_rows",
            "inventory",
            "signatures",
            "defaults",
            "prose",
            "headers",
            "count_claims",
            "templates",
            "references",
            "navigation",
            "contracts",
        ]
    }
    section, group = None, None
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        header = line.startswith("==") or line.startswith(";")
        status = "metadata-only" if header or number == 1 else "UNPROVEN"
        result["source_rows"].append(record("raw", number, line, status=status))
        if line.startswith("=="):
            section = line.strip("= ")
            result["headers"].append(
                record(
                    "header",
                    number,
                    line,
                    level=2,
                    label=section,
                    claimed_count=None,
                    status="metadata-only",
                )
            )
        elif line.startswith(";"):
            group = line[1:].strip()
            result["headers"].append(
                record(
                    "header",
                    number,
                    line,
                    level="definition-term",
                    label=group,
                    claimed_count=None,
                    status="metadata-only",
                )
            )
        elif line.startswith("*"):
            collect_declaration(number, line, section, group, result)
        collect_references(number, line, result)
    first = text.splitlines()[0]
    result["prose"].insert(
        0,
        record(
            "prose",
            1,
            first.split("|misc=", 1)[1][:-2],
            symbol=None,
            status="provenance-only",
        ),
    )
    result["contracts"].append(
        contract(
            1,
            None,
            first,
            "Navigation and adaptation attribution only; 1.3.0/1.5.0 content and source forum post are not expanded. Template output/transclusion revision and historical provenance cannot supply behavioral defaults.",
            suffix="-navigation",
        )
    )
    for field in ["prev", "next"]:
        version = re.search(rf"\|{field}=([^|}}]+)", first)[1]
        result["navigation"].append(
            record(
                "navigation-" + field,
                1,
                field + "=" + version,
                target_version=version,
                expanded=False,
                status="metadata-only",
            )
        )
    totals = {name: len(values) for name, values in result.items()}
    totals.update(
        physical_lines=len(text.splitlines()),
        nonblank_rows=len(result["source_rows"]),
        metadata_rows=sum(
            r["status"] == "metadata-only" for r in result["source_rows"]
        ),
        unproven_source_rows=sum(
            r["status"] == "UNPROVEN" for r in result["source_rows"]
        ),
    )
    return dict(
        result,
        schema="patch-source-accounting/v1",
        patch="1.4.0",
        source=pin,
        base_revision=BASE,
        client_line="original-historical-retail-task-scope",
        literal_client_attribution=None,
        totals=totals,
        measurements={"model": 0, "runtime": 0, "native": 0},
        model_review={
            "candidate": "AcceptBattlefieldPort(accept) true/false transition",
            "observation": "Current src/lua_api/globals/battlefield_verbs.rs reads accept at stack argument 2, while source names a single accept argument. Static mismatch, not executed reproduction.",
            "disposition": "Reported before alias/default invention. Runtime/Cargo/client changes and execution excluded by explicit bounded assignment. Main owns model/native proof; no SOURCE success closes parent behavior goal.",
        },
        history={
            "applied_successors": [],
            "queued_successor_references": [
                p for p in registry if p["version"] == "1.5.0"
            ],
            "queued_150": "/home/osso/.worktrees/wow-ui-sim-p150-page active, separate and unapplied",
            "separate_retail_successor_references": [
                p for p in registry if is_retail_successor(p["version"])
            ],
            "foreign_history_limit": "Classic Era 1.13.x+, TBC 2.5.x, Wrath 3.4.x, Cataclysm 4.4.x, Mists 5.5.x and Forever remain separate, never historical Retail supersession.",
            "templates": "Own-base canonical 1.6.0/1.7.0 evidence/wiki methodology only, not inherited execution proof or expanded target contracts.",
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
    with tempfile.TemporaryDirectory(dir=EVIDENCE) as folder:
        output = Path(folder) / "register.json"
        previous = sys.argv
        try:
            sys.argv = [
                "gen_patch_wikitext_register.py",
                "1.4.0",
                str(EVIDENCE / "source.wikitext"),
                "3052898",
                str(output),
            ]
            generator.main()
        finally:
            sys.argv = previous
        assert (
            output.read_bytes() == (EVIDENCE / "default-register.json").read_bytes()
        ), "default generator bytes"
    extractor = load_historical("extract_patch_non_inventory")
    assert (
        extractor.extract_text((EVIDENCE / "source.wikitext").read_text()).encode()
        == (EVIDENCE / "default-extract.txt").read_bytes()
    ), "default extract_text bytes"
    errors = []
    for tool, function, argument in [
        (generator, "parse_symbol", "not a reference"),
        (extractor, "extract_text", None),
    ]:
        try:
            getattr(tool, function)(argument)
        except Exception as error:
            errors.append(
                dict(
                    function=function,
                    argument=argument,
                    type=type(error).__name__,
                    message=str(error),
                )
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
        output = EVIDENCE / "ledger.json"
        assert not output.exists(), "refuse ledger overwrite"
        output.write_text(json.dumps(build(), indent=2) + "\n")
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
