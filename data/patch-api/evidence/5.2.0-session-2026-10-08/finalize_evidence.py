"""Seal completed own evidence and pinned shared proof inputs, never moving HEAD scope."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def read(name):
    return json.loads((HERE / name).read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def finalize():
    revision = git('rev-parse', 'HEAD').decode().strip()
    base = read('context.json')['base_revision']
    assert not git('diff', '--name-only', base, revision, '--', 'src')
    expected = {name + '.proof.json': 0 for name in (
        'all-sweeps', 'own-prefork', 'raid-integration', 'cooldown-integration',
        'school-integration', 'register-lib', 'format', 'mists',
        'python-fixtures', 'reproduction', 'targeted-driver')}
    expected.update({'discovery.proof.json': 1, 'negative.proof.json': 1})
    for name, code in expected.items():
        assert read(name)['exit'] == code, (name, read(name)['exit'])
    base_names = git('ls-tree', '-r', '--name-only', base, 'data/patch-api/sources').decode().splitlines()
    base_sources = {name: digest(git('show', f'{base}:{name}')) for name in base_names}
    for name, expected_digest in base_sources.items():
        assert digest(git('show', f'{revision}:{name}')) == expected_digest, name
    names = git('ls-tree', '-r', '--name-only', revision).decode().splitlines()
    shared_names = [name for name in names if name.startswith(('src/', 'tools/', 'tests/'))
                    or name in ('Cargo.toml', 'Cargo.lock', 'build.rs', 'docs/wiki/index.md', 'docs/wiki/log.md')
                    or name.startswith('data/patch-api/sources/')]
    shared_inputs = {name: digest(git('show', f'{revision}:{name}')) for name in shared_names}
    ignored = {'finalization.json', 'portability-gate.txt', 'portability-gate.proof.json',
               'portability-gate.launcher.txt', 'local-validator.txt', 'tamper-proof.json',
               'tamper-validator.txt', 'gate-summary.json', 'command-ledger.md'}
    own_artifacts = {path.name: digest(path.read_bytes()) for path in sorted(HERE.iterdir())
                     if path.is_file() and path.name not in ignored}
    final = {'revision': revision, 'base_revision': base, 'shared_inputs': shared_inputs,
             'base_sources': base_sources, 'own_artifacts': own_artifacts, 'receipts': expected,
             'reproduction_revision': read('reproduction.proof.json')['revision'],
             'negative_id': read('negative-control.json')['id'], 'runtime_source_changed': False}
    (HERE / 'finalization.json').write_text(json.dumps(final, indent=2) + '\n')
    lines = ['# Proof ledger', '', 'Each receipt retains exact revision, argv, scope, exit, environment and log SHA-256.',
             'No runtime source changes; no master-runtime rerun needed. No full integration suite.', '',
             '| Receipt | Revision | Exit | Invalidation |', '|---|---|---|---|']
    for name in expected:
        proof = read(name)
        lines.append(f"| {name} | {proof['revision']} | {proof['exit']} | {proof['invalidated']} |")
    (HERE / 'command-ledger.md').write_text('\n'.join(lines) + '\n')
    print(json.dumps({'revision': revision, 'sealed_own_artifacts': len(own_artifacts),
                      'shared_inputs': len(shared_inputs), 'receipts': len(expected)}))


if __name__ == '__main__':
    finalize()
