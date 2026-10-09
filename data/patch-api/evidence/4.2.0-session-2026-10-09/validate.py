"""Replay sealed 4.2.0 historical accounting, not current/native runtime acceptance.

Original handoff ebf7d2c91 is provenance only: no Git object or live repo reads.
Hash pins detect changed retained bytes; they are not authenticity attestations.
"""

from collections import Counter
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

MANIFEST_SHA256 = "fe7c280903e4d9a70dd9af5119f3030ae7b8a5b0fb7dbedf76fc629c2e7401a6"
SOURCE = "data/patch-api/sources/4.2.0-"
GAPS = "tests/data/patch_4_2_0_sweep_known_gaps.json"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sealed_bytes(path, expected):
    data = path.read_bytes()
    require(sha(data) == expected, f"seal mismatch: {path.name}")
    return data


def load_snapshot(evidence):
    manifest = json.loads(
        sealed_bytes(evidence / "historical-inputs.json", MANIFEST_SHA256)
    )
    archive = sealed_bytes(
        evidence / "historical-inputs.tar.gz", manifest["archive_sha256"]
    )
    members = {}
    with tarfile.open(fileobj=io.BytesIO(gzip.decompress(archive))) as stream:
        for entry in stream:
            require(
                entry.isfile() and entry.name not in members, "invalid snapshot member"
            )
            require(
                entry.name in manifest["members"], f"unexpected member: {entry.name}"
            )
            data = stream.extractfile(entry).read()
            require(
                sha(data) == manifest["members"][entry.name],
                f"member seal: {entry.name}",
            )
            members[entry.name] = data
    require(set(members) == set(manifest["members"]), "missing snapshot members")
    retained = {
        name: sealed_bytes(evidence / name, digest)
        for name, digest in manifest["evidence"].items()
    }
    return members, retained


def check_identity(members, retained):
    pin = json.loads(retained["source-pin.json"])
    provenance = json.loads(members[SOURCE + "api-changes.provenance.json"])
    identity = ("title", "version", "pageid", "revid", "timestamp")
    expected = (
        "Patch 4.2.0/API changes",
        "4.2.0",
        315583,
        3045158,
        "2021-08-22T03:01:41Z",
    )
    require(tuple(pin[key] for key in identity) == expected, "source page identity")
    require(
        all(provenance[key] == value for key, value in pin.items()),
        "provenance identity",
    )
    response = retained["source-response.json"]
    require(sha(response) == pin["response_sha256"], "response hash")
    page = json.loads(response)["query"]["pages"][str(pin["pageid"])]
    revision = page["revisions"][0]
    require(
        page["pageid"] == pin["pageid"] and page["title"] == pin["title"],
        "response page identity",
    )
    require(
        revision["revid"] == pin["revid"] and revision["timestamp"] == pin["timestamp"],
        "response revision identity",
    )
    raw = members[SOURCE + "api-changes.wikitext"]
    require(revision["slots"]["main"]["*"].encode() == raw, "response/source bytes")
    require(
        sha(raw) == pin["wikitext_sha256"] and len(raw) == pin["wikitext_bytes"],
        "source hash/length",
    )
    return provenance


def replay_source(members, provenance):
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for name, data in members.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        generated = root / "register.json"
        commands = [
            [
                sys.executable,
                "-I",
                str(root / "tools/gen_patch_wikitext_register.py"),
                "4.2.0",
                str(root / (SOURCE + "api-changes.wikitext")),
                str(provenance["revid"]),
                str(generated),
                *provenance["generator_flags"],
            ],
            [
                sys.executable,
                "-I",
                str(root / "tools/extract_patch_non_inventory.py"),
                "--patch",
                "4.2.0",
                "--text-only",
                *provenance["extractor_flags"],
            ],
        ]
        outputs = []
        for command in commands:
            result = subprocess.run(command, capture_output=True, text=True, timeout=30)
            require(result.returncode == 0, f"source replay failed: {result.stderr}")
            outputs.append(result.stdout)
        require(
            generated.read_bytes() == members[SOURCE + "wikitext-register.json"],
            "register replay differs",
        )
        require(
            (root / (SOURCE + "api-changes.txt")).read_bytes()
            == members[SOURCE + "api-changes.txt"],
            "extract replay differs",
        )
        return json.loads(outputs[1])


def check_receipts(members, retained):
    receipt = json.loads(retained["development-proof.json"])
    for name, digest in receipt["artifact_seals"].items():
        require(sha(retained[name]) == digest, f"receipt artifact seal: {name}")
    for name, digest in receipt["accounting_input_hashes"].items():
        require(sha(members[name]) == digest, f"receipt accounting seal: {name}")
    for step in receipt["steps"]:
        for field in ("log", "results"):
            if field in step:
                require(
                    step[field] in receipt["artifact_seals"],
                    f"unsealed receipt: {step[field]}",
                )
    return len(receipt["artifact_seals"])


def check_later_history(members, retained, register, results):
    pins = json.loads(retained["later-register-pins.json"])
    latest = {}
    for pin in pins["pins"]:
        data = members[pin["path"]]
        require(sha(data) == pin["sha256"], f"later register seal: {pin['path']}")
        later = json.loads(data)
        require(later.get("client_line", "retail") == "retail", "non-retail successor")
        require(
            later["patch"] == pin["patch"] and later["source"] == pin["source"],
            "successor source identity",
        )
        for row in later["entries"]:
            if row["direction"] in ("added", "removed"):
                latest[row["symbol"]] = row
    for row in register["entries"]:
        newer = latest.get(row["symbol"], row)
        expected = results[row["id"]]["expected"]
        publication = "absent" if newer["direction"] == "removed" else "published"
        superseded = newer["id"] if newer["direction"] != row["direction"] else None
        require(
            expected["publication"] == publication
            and expected["superseded_by"] == superseded,
            f"historical successor expectation: {row['id']}",
        )
    return pins


def check_accounting(members, retained, extraction):
    register = json.loads(members[SOURCE + "wikitext-register.json"])
    ledger = json.loads(members[SOURCE + "page-coverage.json"])
    known = json.loads(members[GAPS])
    results = json.loads(retained["development-results.json"])
    negative = json.loads(retained["negative-results.json"])
    ids = {row["id"] for row in register["entries"]}
    require(
        len(ids) == len(register["entries"]) and set(results) == ids,
        "inventory/result identities",
    )
    gaps = {key for key, value in results.items() if not value["ok"]}
    require(
        len(known) == len(set(known)) and set(known) == gaps,
        "historical exact gap fixture",
    )
    negative_gaps = {key for key, value in negative.items() if not value["ok"]}
    require(
        len(negative) == len(results)
        and negative_gaps == gaps | {"negative-p420-missing-global"},
        "negative extra gap",
    )
    for count in register["header_counts"]:
        observed = sum(
            row["direction"] == count["direction"] for row in register["entries"]
        )
        require(
            observed == count["header_count"] == count["parsed_count"],
            "literal header counts",
        )
    rows = {row["source_id"]: row for row in ledger["source_rows"]}
    require(
        len(rows) == len(ledger["source_rows"])
        and set(rows) == ids | {"source-context-001"},
        "ledger identities",
    )
    require(
        rows["source-context-001"]["status"] == "metadata-only", "navigation status"
    )
    for key, result in results.items():
        row = rows[key]
        require(
            row["status"] == ("bounded-coverage" if result["ok"] else "audit-pending"),
            f"ledger status: {key}",
        )
        require(
            row["capabilities"] == (["publication-absence"] if result["ok"] else [])
            and row["note"],
            f"ledger credit: {key}",
        )
    require(
        ledger["source_sha256"] == sha(members[SOURCE + "wikitext-register.json"]),
        "ledger register seal",
    )
    require(
        ledger["non_inventory_source"]["sha256"]
        == sha(members[SOURCE + "api-changes.txt"]),
        "ledger extract seal",
    )
    require(
        extraction["rows"] == 1 and extraction["statuses"] == {"metadata-only": 1},
        "extract metadata boundary",
    )
    summary = json.loads(retained["development-summary.json"])
    require(
        summary["inventory_rows"] == len(ids)
        and summary["directions"]
        == dict(Counter(row["direction"] for row in register["entries"])),
        "summary inventory",
    )
    require(
        summary["status_totals"]
        == dict(Counter(row["status"] for row in rows.values())),
        "summary statuses",
    )
    require(summary["observation_totals"]["gap"] == len(gaps), "summary gaps")
    pins = check_later_history(members, retained, register, results)
    require(
        summary["available_later_registers"] == len(pins["pins"])
        and summary["pending_later_registers"] == pins["pending"],
        "summary successors",
    )
    return {
        "inventory_rows": len(ids),
        "historical_gaps": len(gaps),
        "negative_gaps": len(negative_gaps),
        "ledger_rows": len(rows),
        "later_registers": len(pins["pins"]),
        "pending_later_registers": pins["pending"],
    }


def validate(evidence):
    members, retained = load_snapshot(evidence)
    provenance = check_identity(members, retained)
    seals = check_receipts(members, retained)
    extraction = replay_source(members, provenance)
    report = check_accounting(members, retained, extraction)
    report.update(
        {
            "status": "PASS",
            "source_replay": "byte-identical",
            "receipt_artifact_seals": seals,
            "snapshot_members": len(members),
            "native_runtime_replay": False,
            "scope": "Historical source replay and retained receipt integrity only; no current integration or native/runtime acceptance.",
        }
    )
    return report


if __name__ == "__main__":
    try:
        print(json.dumps(validate(Path(__file__).resolve().parent), indent=2))
    except (
        ValueError,
        KeyError,
        OSError,
        tarfile.TarError,
        subprocess.TimeoutExpired,
    ) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
