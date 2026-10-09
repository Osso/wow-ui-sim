"""Pyrun execution helper; cli is supplied by the host, never a shell."""

from pathlib import Path
import datetime
import hashlib
import json
import os

ROOT = Path("/home/osso/.worktrees/wow-ui-sim-p140-page")
EVIDENCE = ROOT / "data/patch-api/evidence/1.4.0-session-2026-10-09"


def execute(label, argv, ledger="proof-ledger.json", scope=None, env=None):
    destination = EVIDENCE / ledger
    records = json.loads(destination.read_bytes()) if destination.exists() else []
    assert not any(row["label"] == label for row in records), "duplicate proof label"
    paths = scope or [
        p
        for p in EVIDENCE.rglob("*")
        if p.is_file()
        and p.suffix in {".py", ".json", ".wikitext", ".txt"}
        and "__pycache__" not in p.parts
        and p.name not in {ledger, "session_receipts.py"}
    ]
    hashes = {
        str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p): hashlib.sha256(
            p.read_bytes()
        ).hexdigest()
        for p in paths
    }
    revision = (
        cli.git("rev-parse", "HEAD").cwd(str(ROOT)).capture().run().stdout.strip()
    )
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    command = cli.command(*argv).cwd(str(ROOT))
    if env:
        command = command.env(env)
    result = command.capture().run()
    ended = datetime.datetime.now(datetime.timezone.utc).isoformat()
    row = {
        "label": label,
        "revision": revision,
        "cwd": str(ROOT),
        "argv": argv,
        "started": started,
        "ended": ended,
        "exit_code": result.exit_code,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "scope_hashes": hashes,
        "environment_key_names": sorted(os.environ),
        "explicit_environment": env or {},
        "log_order": "stdout then stderr, not chronological interleaving",
    }
    records.append(row)
    destination.write_text(json.dumps(records, indent=2) + "\n")
    (EVIDENCE / (label + ".log")).write_text(result.stdout + result.stderr)
    print(label, revision, result.exit_code, result.stdout, result.stderr)
    return result
