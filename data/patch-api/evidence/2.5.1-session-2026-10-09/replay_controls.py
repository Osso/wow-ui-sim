#!/usr/bin/env python3
"""Fresh historical SOURCE replay plus serialized ledger/log tamper restoration.

Standalone archive cannot depend on Pi helpers. subprocess uses Python argv only,
never a shell, Git, runtime target, network or mutable current project tools.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def replay(path, cwd):
    environment = dict(os.environ, PATH='/nonexistent', HOME=str(cwd),
                       PYTHONPATH='', PYTHONNOUSERSITE='1')
    result = subprocess.run(
        [sys.executable, '-I', '-B', str(path)], cwd=cwd, env=environment,
        capture_output=True, text=True, timeout=30)
    return {'argv': [sys.executable, '-I', '-B', str(path)], 'cwd': str(cwd),
            'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
            'PATH': environment['PATH'], 'HOME': environment['HOME']}


def tamper_and_restore(path, data):
    original = path.read_bytes()
    try:
        path.write_bytes(data)
        result = replay(EVIDENCE / 'validate.py', ROOT)
        assert result['exit'] == 1, 'tampered serialized artifact accepted'
        assert f'seal: {path.relative_to(ROOT)}' in result['stderr'], result
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original, 'byte restore failed'
    assert digest(path.read_bytes()) == digest(original), 'hash restore failed'
    return dict(result, original_sha256=digest(original),
                tampered_sha256=digest(data), restored_sha256=digest(path.read_bytes()))


def copy_archive(seals, archive):
    relative_seals = (EVIDENCE / 'seals.json').relative_to(ROOT).as_posix()
    members = dict(seals, **{relative_seals: digest((EVIDENCE / 'seals.json').read_bytes())})
    with tarfile.open(archive, 'w:gz') as writer:
        for name in sorted(members):
            writer.add(ROOT / name, arcname=name, recursive=False)
    return members


def main():
    ledger_path = ROOT / 'data/patch-api/sources/2.5.1-page-coverage.json'
    ledger = json.loads(ledger_path.read_bytes())
    ledger['contracts'][0]['native_equivalence'] = 'fabricated Anniversary/TBC parity'
    ledger_control = tamper_and_restore(ledger_path, (json.dumps(ledger, indent=2) + '\n').encode())
    log_path = EVIDENCE / 'green.log'
    log_control = tamper_and_restore(log_path, log_path.read_bytes() + b'Fabricated native replay receipt\n')
    seals = json.loads((EVIDENCE / 'seals.json').read_bytes())
    archive = EVIDENCE / 'replay-archive.tar.gz'
    assert archive.relative_to(ROOT).as_posix() not in seals, 'archive already sealed'
    members = copy_archive(seals, archive)
    # Fresh filesystem tree contains only sealed historical originals, not a
    # checkout. Python isolated mode prevents external site/PYTHONPATH imports.
    with tempfile.TemporaryDirectory(prefix='.p251-replay-', dir=ROOT) as temp:
        relocated = Path(temp) / 'relocated'
        relocated.mkdir()
        with tarfile.open(archive) as reader:
            assert sorted(reader.getnames()) == sorted(members), 'archive member list'
            reader.extractall(relocated, filter='data')
        for forbidden in ['.git', 'target', 'tools', 'src', 'Cargo.toml']:
            assert not (relocated / forbidden).exists(), f'current dependency: {forbidden}'
        for name, expected in members.items():
            assert digest((relocated / name).read_bytes()) == expected, f'archive member: {name}'
        path = relocated / (EVIDENCE / 'validate.py').relative_to(ROOT)
        result = replay(path, relocated)
        assert result['exit'] == 0, result
        print(json.dumps({
            'ledger_tamper': ledger_control, 'log_tamper': log_control,
            'relocated_replay': result, 'archive_member_hashes': members,
            'archive_sha256': digest(archive.read_bytes()), 'archive_bytes': archive.stat().st_size,
            'git_present': False, 'target_present': False, 'current_tools_present': False,
            'current_runtime_present': False, 'current_config_present': False,
            'source_summary': json.loads(result['stdout']),
            'limit': 'SOURCE/configuration reproduction only; native/model/full UI not measured.',
        }, indent=2))


if __name__ == '__main__':
    main()
