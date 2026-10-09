"""Frozen historical Retail 1.9.0 redirect accounting; SOURCE-only proof."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile

EVIDENCE = Path(__file__).resolve().parent
BASE = "3d1a4140734c9fbc82cd3ec6dfaf0536767a9edd"


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
    assert (
        digest((EVIDENCE / "frozen-manifest.json").read_bytes())
        == "b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba"
    ), "manifest hash"
    assert (
        digest((EVIDENCE / "source-pin.json").read_bytes())
        == "76f18437cf8dd55a5379e4c9265eaf5f85250574bfa6be0eb24bb62c2011dc2a"
    ), "pin hash"
    pin = read_json("source-pin.json")
    manifest = read_json("frozen-manifest.json")
    assert pin == next(p for p in manifest["pages"] if p["version"] == "1.9.0"), (
        "manifest pin"
    )
    assert (
        digest(raw)
        == pin["wikitext_sha256"]
        == "96679828d7b158a7117a5123bcd91a92be223cfa2c285d6ba84beba2606a48b2"
    ), "raw hash"
    assert (
        digest(response)
        == pin["response_sha256"]
        == "506f008a1792fc53fbb2b01f83aaa0677b1ab1f3b26ab92470a5764a6c1435b7"
    ), "response hash"
    assert len(raw) == pin["wikitext_bytes"] == 45, "raw bytes"
    page = json.loads(response)["query"]["pages"]["278324"]
    (revision,) = page["revisions"]
    assert page["pageid"] == pin["pageid"] == 278324, "pageid"
    assert page["title"] == pin["title"] == "Patch 1.9.0/API changes", "title"
    assert revision["revid"] == pin["revid"] == 2690597, "revid"
    assert revision["timestamp"] == pin["timestamp"] == "2020-04-05T21:06:37Z", (
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
    registered = next(p for p in registry if p["version"] == "1.9.0")
    assert all(pin[k] == v for k, v in registered.items()), "registry identity"
    return raw, pin, registry


def is_retail_successor(version):
    major, minor, _ = map(int, version.split("."))
    return major >= 2 and (major, minor) not in {(2, 5), (3, 4), (4, 4), (5, 5)}


def build():
    raw, pin, registry = validate_source()
    text = raw.decode()
    match = re.fullmatch(r"#REDIRECT (\[\[([^\]]+)\]\])", text)
    assert match, "frozen redirect boundary"
    rows = [
        {
            "id": "raw-1.9.0-001",
            "line": 1,
            "literal": text,
            "status": "metadata-only",
            "capabilities": [],
        }
    ]
    references = [
        {
            "id": "link-1.9.0-001",
            "line": 1,
            "literal": match[1],
            "target": match[2],
            "status": "UNPROVEN",
            "expanded": False,
        }
    ]
    contracts = [
        {
            "id": "contract-1.9.0-redirect",
            "reference_id": references[0]["id"],
            "status": "UNPROVEN",
            "arguments": None,
            "returns": None,
            "defaults": None,
            "events": None,
            "state_transitions": None,
            "security": None,
            "native_equivalence": None,
            "missing_contract": "Pinned redirect target revision/content and patch-specific historical API delta are absent. No local callable/signature/default/prose declaration supplies a model contract.",
        }
    ]
    empty = {
        name: []
        for name in [
            "inventory",
            "signatures",
            "defaults",
            "prose",
            "headers",
            "templates",
        ]
    }
    return dict(
        empty,
        schema="patch-source-accounting/v1",
        patch="1.9.0",
        source=pin,
        base_revision=BASE,
        client_line="original-historical-retail-task-scope",
        literal_client_attribution=None,
        source_rows=rows,
        references=references,
        contracts=contracts,
        measurements={"model": 0, "runtime": 0, "native": 0},
        model_review={
            "result": "no-grounded-behavioral-subset",
            "reason": "Only a redirect is published. Choosing current APIs or Classic defaults would invent a historical contract; no runtime changes or tests justified.",
            "current_default_retail_limit": "Current model tests cannot establish native 1.9.0 equivalence.",
        },
        history={
            "separate_retail_successor_references": [
                p for p in registry if is_retail_successor(p["version"])
            ],
            "applied_successors": [],
            "queued_1100": "1.10.0 active separately; queued unapplied, no imported contracts",
            "queued_successor_references": [
                p
                for p in registry
                if p["version"] in {"1.10.0", "1.10.1", "1.10.2", "1.11.0", "1.12.0"}
            ],
            "reference_source_revision": "c6fec74a1",
            "foreign_history_limit": "Classic Era 1.13.x+, TBC Classic 2.5.x, Wrath Classic 3.4.x, Cataclysm Classic 4.4.x, Mists Classic 5.5.x and Forever are separate. Links/navigation never import behavior.",
            "retail_limit": "Original Retail 2.0.1 and later historical Retail pages are separate successors; none can supply missing redirect content or create local inventory.",
        },
        totals={
            "physical_lines": len(text.splitlines()),
            "nonblank_rows": len(rows),
            "metadata_rows": sum(r["status"] == "metadata-only" for r in rows),
            "unproven_source_rows": sum(r["status"] == "UNPROVEN" for r in rows),
            "references": len(references),
            "unproven_contracts": len(contracts),
            **{name: len(values) for name, values in empty.items()},
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
                "1.9.0",
                str(EVIDENCE / "source.wikitext"),
                "2690597",
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
