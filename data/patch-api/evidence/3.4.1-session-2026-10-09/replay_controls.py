#!/usr/bin/env python3
"""Own source fixtures: serialized tampering and relocated archive replay."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


def replay(path):
    result = subprocess.run(
        [sys.executable, str(path)], cwd=ROOT, capture_output=True, text=True,
        timeout=30,
    )
    return {"exit": result.returncode, "stdout": result.stdout,
            "stderr": result.stderr}


def tamper_and_restore(path, data):
    original = path.read_bytes()
    try:
        path.write_bytes(data)
        result = replay(EVIDENCE / "validate.py")
        assert result["exit"] == 1, "tampered serialized artifact accepted"
        assert f"seal: {path.relative_to(ROOT)}" in result["stderr"], result
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original, "restore failed"
    return dict(result, restored_sha256=hashlib.sha256(original).hexdigest())


def main():
    ledger_path = ROOT / "data/patch-api/sources/3.4.1-page-coverage.json"
    ledger = json.loads(ledger_path.read_bytes())
    ledger["inventory_rows"][0]["status"] = "native-covered"
    source_control = tamper_and_restore(
        ledger_path, (json.dumps(ledger, indent=2) + "\n").encode()
    )
    log_path = EVIDENCE / "green.log"
    log_control = tamper_and_restore(
        log_path, log_path.read_bytes() + b"Fabricated native replay receipt\n"
    )
    seals = json.loads((EVIDENCE / "seals.json").read_bytes())
    relative_seals = (EVIDENCE / "seals.json").relative_to(ROOT)
    with tempfile.TemporaryDirectory(prefix=".p341-archive-", dir=ROOT) as temp:
        temp = Path(temp)
        archive = temp / "proof.tar"
        with tarfile.open(archive, "w") as writer:
            for name in sorted(seals):
                writer.add(ROOT / name, arcname=name, recursive=False)
            writer.add(ROOT / relative_seals, arcname=str(relative_seals), recursive=False)
        relocated = temp / "relocated"
        relocated.mkdir()
        with tarfile.open(archive) as reader:
            reader.extractall(relocated, filter="data")
        assert not (relocated / ".git").exists(), "unexpected Git dependency"
        archived_validator = relocated / (EVIDENCE / "validate.py").relative_to(ROOT)
        archive_result = replay(archived_validator)
        assert archive_result["exit"] == 0, archive_result
        summary = json.loads(archive_result["stdout"])
        print(json.dumps({
            "source_tamper": source_control, "log_tamper": log_control,
            "relocated_replay": archive_result,
            "archive_member_hashes": seals,
            "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
            "git_present": False, "source_summary": summary,
            "cwd": str(ROOT),
            "limit": "Historical source/log accounting only; no runtime/native replay.",
        }, indent=2))


if __name__ == "__main__":
    main()
