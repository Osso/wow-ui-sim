"""Bounded SOURCE fixture: disk tampering/restoration and copied-root replay."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
RELATIVE = EVIDENCE.relative_to(ROOT)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def replay(root):
    argv = [sys.executable, '-B', str(root / RELATIVE / 'validate.py')]
    result = subprocess.run(argv, cwd=root, capture_output=True, text=True,
                            env={'PATH': '/nonexistent', 'PYTHONDONTWRITEBYTECODE': '1'},
                            timeout=30)
    return dict(argv=argv, cwd=str(root), exit_code=result.returncode,
                stdout=result.stdout, stderr=result.stderr, git_available=False)


def tamper_and_restore(path, changed):
    original = path.read_bytes()
    try:
        path.write_bytes(changed)
        result = replay(ROOT)
        assert result['exit_code'] == 1, 'serialized tampering accepted'
        assert f'seal: {path.relative_to(ROOT)}' in result['stderr'], result
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original, 'restore bytes'
    assert digest(path.read_bytes()) == digest(original), 'restore hash'
    return dict(result, restored_sha256=digest(original))


def main():
    assert shutil.which('git', path='/nonexistent') is None, 'Git unexpectedly available'
    original = replay(ROOT)
    assert original['exit_code'] == 0, original
    ledger_path = ROOT / 'data/patch-api/sources/2.3.0-page-coverage.json'
    ledger = json.loads(ledger_path.read_bytes())
    ledger['inventory_rows'][0]['status'] = 'native-covered'
    ledger_control = tamper_and_restore(
        ledger_path, (json.dumps(ledger, indent=2) + '\n').encode())
    log_path = EVIDENCE / 'green.log'
    log_control = tamper_and_restore(
        log_path, log_path.read_bytes() + b'Fabricated native receipt\n')
    seals_path = EVIDENCE / 'seals.json'
    seals = json.loads(seals_path.read_bytes())
    with tempfile.TemporaryDirectory(prefix='.p230-copy-', dir=ROOT) as temp:
        temp = Path(temp)
        archive = temp / 'historical.tar'
        with tarfile.open(archive, 'w') as writer:
            for name in sorted(seals):
                writer.add(ROOT / name, arcname=name, recursive=False)
            writer.add(seals_path, arcname=str(seals_path.relative_to(ROOT)), recursive=False)
        assert archive.stat().st_size < 5_000_000, 'archive exceeds 5MB'
        copied = temp / 'copied'
        copied.mkdir()
        with tarfile.open(archive) as reader:
            reader.extractall(copied, filter='data')
        assert not (copied / '.git').exists(), 'copied Git'
        assert not (copied / 'target').exists(), 'copied target'
        assert not (copied / 'data/patch-api/source-cache').exists(), 'copied cache'
        copied_result = replay(copied)
        assert copied_result['exit_code'] == 0, copied_result
        assert copied_result['stdout'] == original['stdout'], 'relocated summary drift'
        assert json.loads(copied_result['stdout'])['runtime_observations'] == 0
        member_hashes = {name: digest((copied / name).read_bytes()) for name in seals}
        assert member_hashes == seals, 'copied historical bytes'
        print(json.dumps(dict(
            original_replay=original, ledger_tamper=ledger_control, log_tamper=log_control,
            copied_replay=copied_result, archive_bytes=archive.stat().st_size,
            archive_sha256=digest(archive.read_bytes()), archive_member_hashes=member_hashes,
            seals_sha256=digest(seals_path.read_bytes()), sealed_inputs=len(seals),
            git_present=False, target_present=False, cache_present=False,
            summary=json.loads(original['stdout']),
            proof_limit='Historical SOURCE-only replay; no runtime/native/model/final credit.'
        ), indent=2))


if __name__ == '__main__':
    main()
