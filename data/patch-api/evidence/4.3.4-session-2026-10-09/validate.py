#!/usr/bin/env python3
"""Validate bounded 4.3.4 historical bytes, never today's runtime or ledger.

Only the sealed sparse archive supplies historical inputs. Revision/tree labels
are provenance, not dependencies on reachable Git objects or executable proof.
"""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys
import types

SOURCE = "data/patch-api/sources/4.3.4-"
BOUNDARY = "historical artifacts only; not fresh model/native proof"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(page, name):
    return json.loads((page / name).read_text(encoding="utf-8"))


def validate_seals(page, manifest, label):
    for name, pin in manifest["files"].items():
        require(Path(name).name == name, f"{label}: non-local path {name}")
        data = (page / name).read_bytes()
        require(pin == {"sha256": sha(data), "bytes": len(data)}, f"{label}: {name}")


def archived_inputs(page):
    pins = load(page, "historical-input-pins.json")
    raw = (page / pins["archive"]).read_bytes()
    require(sha(raw) == pins["archive_sha256"], "archive seal")
    archive = json.loads(gzip.decompress(raw))
    require(set(archive) == set(pins["paths"]), "archive path set")
    for path, text in archive.items():
        data = text.encode("utf-8")
        blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        require(pins["paths"][path] == {"sha256": sha(data), "blob": blob}, f"archived identity: {path}")
    scope = load(page, "proof-scope.json")
    require(scope["unchanged_historical_input_files"] == len(archive), "historical input count")
    require(scope["recorded_revision"] == pins["revision_label"], "historical revision label")
    require(scope["invalidated_inputs"] == [], "historical invalidated inputs")
    return archive, pins


def archived_tool(archive, path):
    # Execute the pinned local parser, not a present-day shared tool or provider.
    module = types.ModuleType("p434_" + Path(path).stem)
    module.__file__ = path
    sys.modules[module.__name__] = module
    exec(compile(archive[path], path, "exec"), module.__dict__)
    return module


def validate_source(page, archive):
    pin = load(page, "source-pin.json")
    provenance = json.loads(archive[SOURCE + "api-changes.provenance.json"])
    require(all(provenance.get(k) == v for k, v in pin.items()), "source provenance")
    require(provenance["generator_flags"] == provenance["extractor_flags"] == [], "default source flags")
    require(provenance["client_line"] == "retail", "source client line")
    raw = archive[SOURCE + "api-changes.wikitext"].encode()
    require(sha(raw) == pin["wikitext_sha256"] and len(raw) == pin["wikitext_bytes"], "wikitext identity")
    response_bytes = (page / "source-response.json").read_bytes()
    require(sha(response_bytes) == pin["response_sha256"], "source response identity")
    pages = json.loads(response_bytes)["query"]["pages"]
    require(list(pages) == [str(pin["pageid"])], "source response page set")
    response_page = pages[str(pin["pageid"])]
    require(response_page["pageid"] == pin["pageid"] and response_page["title"] == pin["title"], "source response page")
    require(len(response_page["revisions"]) == 1, "source response revisions")
    revision = response_page["revisions"][0]
    require(revision["revid"] == pin["revid"] and revision["timestamp"] == pin["timestamp"], "source response revision")
    require(revision["slots"]["main"]["*"] == raw.decode(), "source response wikitext")
    register = json.loads(archive[SOURCE + "wikitext-register.json"])
    generator = archived_tool(archive, "tools/gen_patch_wikitext_register.py")
    buckets = generator.split_sections(raw.decode())
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section("cvars" if section == "commands" else section, buckets.get(section, []))
        if section == "commands":
            for count in counts:
                count["section"] = "commands"
        entries.extend(rows)
        headers.extend(counts)
    replay = {"schema": "patch-api-wikitext-register/v1", "patch": pin["version"], "source": {"path": SOURCE + "api-changes.wikitext", "revid": pin["revid"], "sha256": sha(raw)}, "header_counts": headers, "entries": entries}
    require(register == replay, "default register replay")
    require((json.dumps(replay, indent=2, ensure_ascii=False) + "\n") == archive[SOURCE + "wikitext-register.json"], "register serialized replay")
    extractor = archived_tool(archive, "tools/extract_patch_non_inventory.py")
    text = extractor.extract_text(raw.decode())
    require(text == archive[SOURCE + "api-changes.txt"], "default extract replay")
    for header in headers:
        count = sum(row["section"] == header["section"] and row["direction"] == header["direction"] for row in entries)
        require(count == header["parsed_count"] == header["header_count"], "header counts")
    return register, extractor.seed_rows(text, patch=pin["version"])


def indexed(rows, field):
    result = {row[field]: row for row in rows}
    require(len(result) == len(rows), f"duplicate {field}")
    return result


def validate_successors(accounting, archive, own):
    records = accounting["later_registers"]
    paths = [record["path"] for record in records]
    archived = {path for path in archive if path.endswith("-wikitext-register.json") and path != SOURCE + "wikitext-register.json"}
    require(len(paths) == len(set(paths)) and set(paths) == archived, "successor path set")
    latest = {}
    versions = []
    for record in records:
        require(record["sha256"] == sha(archive[record["path"]].encode()), "successor identity")
        register = json.loads(archive[record["path"]])
        require(record["patch"] == register["patch"] and record["client_line"] == register.get("client_line", "retail") == "retail", "successor client line")
        versions.append(tuple(map(int, register["patch"].split("."))))
        for row in register["entries"]:
            latest[(row["section"], row["symbol"])] = row
    require(versions == sorted(set(versions)) and all(v > (4, 3, 4) for v in versions), "successor chronology")
    supersessions = {}
    for key, row in own.items():
        successor = latest.get((row["section"], row["symbol"]))
        if successor:
            supersessions[key] = successor
    return supersessions


def validate_observations(results, rows, supersessions):
    require(set(results) == set(rows), "observation cardinality/IDs")
    for key, row in rows.items():
        result = results[key]
        expected, observed = result["expected"], result["observed"]
        successor = supersessions.get(key)
        publication = "absent" if (successor or row)["direction"] == "removed" else "published"
        contract = {"direction": row["direction"], "page_default": None, "publication": publication, "section": row["section"], "superseded_by": successor["id"] if successor else None, "symbol": row["symbol"]}
        require(expected == contract, f"observation contract: {key}")
        require(observed == {"default": None, "default_mismatch": False, "detail": "raw=nil; lookup=nil", "kind": "global", "value": None}, f"historical observation: {key}")
        require(result["ok"] is (publication == "absent"), f"observation outcome: {key}")
    return sorted(key for key, result in results.items() if not result["ok"])


def validate_accounting(page, archive, register, context):
    accounting = load(page, "accounting.json")
    rows = indexed(register["entries"], "id")
    successors = validate_successors(accounting, archive, rows)
    supersessions = {key: row["id"] for key, row in successors.items()}
    own = load(page, "own-results.json")
    gaps = validate_observations(own, rows, successors)
    require(load(page, "discovery-results.json") == own, "discovery observations")
    fixture = json.loads(archive["tests/data/patch_4_3_4_sweep_known_gaps.json"])
    require(fixture == gaps == accounting["gaps"], "historical gaps")
    ledger = json.loads(archive[SOURCE + "page-coverage.json"])
    ledger_rows = indexed(ledger["source_rows"], "source_id")
    context_ids = {row["source_id"] for row in context}
    require(set(ledger_rows) == set(rows) | context_ids and not set(rows) & context_ids, "ledger IDs/context")
    for key, ledger_row in ledger_rows.items():
        status = "metadata-only" if key in context_ids else "publication-gap" if key in gaps else "superseded-publication" if key in supersessions else "absence-only"
        require(ledger_row["status"] == status and ledger_row["capabilities"] == [] and bool(ledger_row["note"]), f"ledger status: {key}")
    require(ledger["source_sha256"] == sha(archive[SOURCE + "wikitext-register.json"].encode()), "ledger source pin")
    require(ledger["non_inventory_source"] == {"path": SOURCE + "api-changes.txt", "sha256": sha(archive[SOURCE + "api-changes.txt"].encode())}, "ledger extract pin")
    derived = {"inventory": len(rows), "directions": dict(Counter(row["direction"] for row in rows.values())), "header_counts": register["header_counts"], "ledger_ids": len(ledger_rows), "ledger_statuses": dict(Counter(row["status"] for row in ledger_rows.values())), "observations": len(own), "ok_absence_only": len(own) - len(gaps), "supersessions": supersessions, "non_inventory_metadata": len(context_ids), "prose_contracts": 0, "signature_contracts": 0, "retirements": [], "runtime_fixes": [], "native_parity": False}
    for key, value in derived.items():
        require(accounting[key] == value, f"accounting {key}")
    negative_register = load(page, "negative-register.json")
    negative_rows = indexed(negative_register["entries"], "id")
    removed, added = set(rows) - set(negative_rows), set(negative_rows) - set(rows)
    require(len(negative_rows) == len(rows) and len(removed) == len(added) == 1, "negative same-cardinality replacement")
    removed_id, added_id = next(iter(removed)), next(iter(added))
    require(rows[removed_id]["symbol"] == "ComplainChat" and added_id == "negative-fabricated-global" and negative_rows[added_id]["symbol"] == "P434FabricatedMissingAPI" and negative_rows[added_id]["direction"] == "added", "negative fabricated control")
    require(all(rows[key] == negative_rows[key] for key in set(rows) & set(negative_rows)), "negative unchanged rows")
    require(all(negative_register[key] == register[key] for key in register if key != "entries"), "negative register metadata")
    negative = load(page, "negative-results.json")
    negative_gaps = validate_observations(negative, negative_rows, successors)
    require(set(negative_gaps) == set(gaps) | {added_id} and len(negative_gaps) == accounting["negative_gap_count"], "negative gap delta")
    require(all(negative[key] == own[key] for key in set(rows) & set(negative_rows)), "negative unchanged observations")
    return {"inventory": len(rows), "ledger_ids": len(ledger_rows), "successors": len(accounting["later_registers"]), "gaps": len(gaps), "negative_gaps": len(negative_gaps), "supersessions": len(supersessions), "boundary": BOUNDARY}


def validate_receipts(page, archive, pins):
    cwd = "/home/osso/.worktrees/wow-ui-sim-p434-source"
    sweep = ["cargo", "test", "--test", "prefork_full_ui", "--", "patch_4_3_4_publication_sweep", "--nocapture"]
    for name, exit_code in [("discovery", 1), ("own", 0), ("negative", 1)]:
        proof = load(page, name + ".proof.json")
        require(proof["command"] == sweep, "receipt command")
        # Relocation affects the validator path, never the original receipt cwd.
        evidence = cwd + "/data/patch-api/evidence/4.3.4-session-2026-10-09/"
        environment = {
            "CARGO_TARGET_DIR": cwd + "/target",
            "P434_SWEEP_OUT": evidence + name + "-results.json",
        }
        if name == "negative":
            environment["P434_SWEEP_REGISTER"] = cwd + "/data/patch-api/evidence/4.3.4-session-2026-10-09/negative-register.json"
        require(proof["environment"] == environment, "receipt environment")
        require(proof["exit"] == exit_code, "receipt exit")
        log = (page / proof["log"]).read_text()
        summary = "test result: ok. 1 passed; 0 failed; 1 total" if not exit_code else "test result: FAILED. 0 passed; 1 failed; 1 total"
        require(summary in log, "receipt log summary")
        require(proof["revision"] == ("5c3bccdf7c2ecb6c957c2a300e6bbc02fd9448ec" if name == "discovery" else pins["revision_label"]), "receipt revision")
    reproduction = load(page, "reproduction.proof.json")
    require(reproduction["generator_flags"] == reproduction["extractor_flags"] == [], "reproduction flags")
    for kind, suffix in [("register", "wikitext-register.json"), ("extract", "api-changes.txt")]:
        require(reproduction[kind + "_byte_identical"] is True and reproduction[kind + "_sha256"] == sha(archive[SOURCE + suffix].encode()), "reproduction identity")
    expected_commands = [
        [
            "python3", cwd + "/tools/gen_patch_wikitext_register.py", "4.3.4",
            cwd + "/" + SOURCE + "api-changes.wikitext", "3743181",
            evidence + "reproduced-register.json",
        ],
        [
            "python3", cwd + "/tools/extract_patch_non_inventory.py",
            "--patch", "4.3.4", "--text-only", "--check",
        ],
    ]
    require(
        [proof["command"] for proof in reproduction["proofs"]] == expected_commands,
        "reproduction receipt commands",
    )
    require(
        all(proof["exit"] == 0 for proof in reproduction["proofs"]),
        "reproduction receipt exits",
    )
    formatter = load(page, "format.proof.json")
    require(formatter["exit"] == 0, "format receipt exit")
    require(
        formatter["command"] == [
            "rustfmt", "--edition", "2024",
            cwd + "/tests/patch_4_3_4_publication_sweep.rs",
        ],
        "format receipt command",
    )
    require(
        all(proof["revision"] == pins["revision_label"]
            for proof in reproduction["proofs"] + [formatter]),
        "reproduction/format receipt revision",
    )
    for filename in ("own.proof.json", "negative.proof.json", "discovery.proof.json", "format.proof.json", "reproduction.proof.json"):
        container = load(page, filename)
        for proof in container.get("proofs", [container]):
            require(proof["cwd"] == cwd, "receipt cwd")
            expected_log = (
                f"reproduction-{reproduction['proofs'].index(proof)}.log"
                if filename == "reproduction.proof.json"
                else filename.replace(".proof.json", ".log")
            )
            require(proof["log"] == expected_log, "receipt log name")
            require(
                sha((page / proof["log"]).read_bytes()) == proof["log_sha256"],
                "receipt log hash",
            )
            require(bool(proof["command"]) and re.fullmatch(r"[0-9a-f]{40}", proof["revision"]), "receipt schema")


def validate(page=None):
    page = Path(page) if page is not None else Path(__file__).resolve().parent
    validate_seals(page, load(page, "session-seals.json"), "sealed artifact")
    archive, pins = archived_inputs(page)
    register, context = validate_source(page, archive)
    result = validate_accounting(page, archive, register, context)
    validate_receipts(page, archive, pins)
    acceptance = load(page, "validator-acceptance-pins.json")
    require(set(acceptance["files"]) == {"validate.py", "test_validate.py", "session-seals.json", "validation-note.md"}, "acceptance file set")
    validate_seals(page, acceptance, "acceptance artifact")
    return result


if __name__ == "__main__":
    try:
        print(json.dumps(validate(), sort_keys=True))
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"historical validation rejected: {error}", file=sys.stderr)
        sys.exit(1)
