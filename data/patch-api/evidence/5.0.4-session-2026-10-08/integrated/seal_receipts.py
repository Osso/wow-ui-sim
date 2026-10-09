"""Seal completed proof and compare exact observation bytes to pinned master."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = '1c9984d2e8a5e672c5109d2234197303e6190317'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read(path):
    return json.loads(path.read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def main():
    revision = git('rev-parse', 'HEAD').decode().strip()
    comparisons = []
    for path in sorted(HERE.glob('patch_*_publication_sweep-results.json')):
        patch = path.name.removeprefix('patch_').removesuffix('_publication_sweep-results.json').replace('_', '.')
        observations = read(path)
        identical = None if patch == '5.0.4' else path.read_bytes() == (HERE / 'master' / path.name).read_bytes()
        if patch != '5.0.4':
            assert identical, ('other page changed', patch)
        comparisons.append({'patch': patch, 'observations': len(observations),
                            'gaps': sorted(key for key, value in observations.items() if not value['ok']),
                            'unchanged_vs_master': identical})
    dump('gap-comparison.json', comparisons)
    receipts, counts = {}, {}
    failures = {'reproduction': 1, 'initial-all-sweeps': 1, 'initial-own-prefork': 1,
                'historical-replay-after-closure': 1, 'checks': 1, 'negative': 1}
    ledger = ['# Integrated 5.0.4 proof ledger', '',
              f'Pinned master: `{MASTER}`. Integrated shared-input revision: `{revision}`.', '',
              'Historical receipts remain unchanged. Initial register/extract reproduction completed before a source-only Classic preservation probe failed; the corrected preservation/supplemental command passed without rerunning completed register/extract work. Initial sweep failures identify only two stale resolved gaps; final sweeps use the corrected fixture. No broad command repeated without that intersecting fixture change.', '',
              'The own JSON fixture change does not invalidate selected pet-battle, namespace/library, parser or historical proof. All observed Rust counts are selected behavior, never implementation-shape tests. Source-only Classic pages remain separate from retail successors. No full suite.', '',
              '| Command scope | Exact revision | Exit | Observed result | Log SHA-256 |',
              '|---|---|---:|---|---|']
    for path in sorted(HERE.glob('*.proof.json')):
        name = path.name.removesuffix('.proof.json')
        receipt = read(path)
        text = (HERE / receipt['log']).read_text()
        results = re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed', text)
        if results:
            counts[name] = list(map(int, results[-1]))
        expected = failures.get(name, 0)
        assert receipt['exit'] == expected, (name, receipt['exit'], expected)
        assert receipt['log_sha256'] == digest((HERE / receipt['log']).read_bytes())
        receipts[name] = expected
        result = f'{counts[name][0]} pass / {counts[name][1]} fail' if name in counts else 'recorded terminal output'
        fixtures = re.findall(r'Ran (\d+) tests?', text)
        if fixtures:
            result = fixtures[-1] + ' Python tests'
        ledger.append(f'| `{name}` | `{receipt["revision"]}` | {receipt["exit"]} | {result} | `{receipt["log_sha256"]}` |')
        ledger.extend(['', '```json', json.dumps({'scope': name, 'command': receipt['command'], 'environment': receipt['environment']}, sort_keys=True), '```', ''])
    (HERE / 'command-ledger.md').write_text('\n'.join(ledger) + '\n')
    prior = {}
    for path in git('ls-tree', '-r', '--name-only', MASTER, 'data/patch-api/evidence').decode().splitlines():
        if path.endswith('/validate.py'):
            prior[path] = git('rev-parse', MASTER + ':' + path).decode().strip()
    guards = ['data/patch-api/sources/5.0.4-api-changes.provenance.json',
              'data/patch-api/sources/5.0.4-api-changes.txt',
              'data/patch-api/sources/5.0.4-api-changes.wikitext',
              'data/patch-api/sources/5.0.4-wikitext-register.json',
              'data/patch-api/sources/5.0.4-page-coverage.json',
              'tests/data/patch_5_0_4_sweep_known_gaps.json']
    context = {'master_revision': MASTER, 'runtime_revision': revision,
               'dispatcher_sha256': digest((HERE.parent / 'validate.py').read_bytes()),
               'directory_trees': {directory: git('rev-parse', revision + ':' + directory).decode().strip()
                                   for directory in ['src', 'tests', 'tools', 'data/patch-api/sources']},
               'current_source_guards': {path: digest(git('show', revision + ':' + path)) for path in guards},
               'prior_validator_blobs': prior,
               'negative_id': 'diff-wt-global-api-C_PetBattles.GetPetType-62',
               'proofs': receipts, 'test_counts': counts,
               'own_files': {path.relative_to(HERE).as_posix(): digest(path.read_bytes())
                             for path in sorted(HERE.rglob('*')) if path.is_file()
                             and path.name not in ['context.json', 'gate-report.json', 'gate-summary.json', 'tamper-proof.json']
                             and not path.name.startswith(('validator-gate', 'tamper-validator'))}}
    dump('context.json', context)
    print(json.dumps({'revision': revision, 'pages': len(comparisons),
                      'other_observations': sum(row['observations'] for row in comparisons if row['patch'] != '5.0.4'),
                      'proof_commands': len(receipts)}))


if __name__ == '__main__':
    main()
