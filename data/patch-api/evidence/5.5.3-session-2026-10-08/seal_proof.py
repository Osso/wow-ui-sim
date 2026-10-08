"""Seal completed logs and pinned shared inputs; fail if any proof is unfinished."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '4d046d99d29f71edcd83e55076fd0b07d8f9ff4f'
LABELS = ['all-sweeps', 'client-lines', 'mists-pages-final', 'mists-check', 'negative', 'tools-tests', 'format-final']


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def main():
    receipts = {label: read(label + '.proof.json') for label in LABELS}
    revision = receipts['mists-pages-final']['revision']
    for receipt in receipts.values():
        assert receipt['exit'] == receipt['expected_exit']
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
    baseline = read('master-baseline.json')
    comparison = []
    for row in baseline['records']:
        before = read(row['path'])
        after = read('results/' + Path(row['path']).name)
        assert before == after, row['patch']
        comparison.append({'patch': row['patch'], 'observations': len(after),
                           'gaps': sorted(key for key, value in after.items() if not value['ok']), 'identical': True})
    (HERE / 'gap-comparison.json').write_text(json.dumps(comparison, indent=2) + '\n')
    proofs = {}
    for label, receipt in receipts.items():
        policy = {'command': receipt['command'], 'exit': receipt['expected_exit']}
        if label in ['all-sweeps', 'client-lines', 'mists-pages-final']:
            text = (HERE / receipt['log']).read_text()
            policy['passed_cases'] = sorted(re.findall(r'^test (\S+) \.\.\. ok$', text, re.M))
        proofs[label] = policy
    shared_paths = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').decode().splitlines()
    shared_paths.extend(['tests/patch_5_5_3_publication_sweep.rs', 'tests/patch_5_5_4_publication_sweep.rs',
                         'tests/publication_sweep_client_lines.rs', 'tests/data/patch_5_5_3_sweep_known_gaps.json',
                         'tools/extract_patch_non_inventory.py', 'tools/gen_patch_wikitext_register.py'])
    session = {}
    for path in sorted(HERE.rglob('*')):
        if not path.is_file() or path.name in ['context.json', 'gate-report.json', 'gate-summary.json', 'tamper-proof.json', 'validator-output.txt']:
            continue
        if path.name.endswith('.launcher.txt'):
            continue
        session[path.relative_to(HERE).as_posix()] = digest(path.read_bytes())
    context = {'runtime_revision': revision, 'master_revision': BASE, 'proofs': proofs,
               'shared_sha256': {path: digest(git('show', revision + ':' + path)) for path in shared_paths},
               'session_sha256': session}
    (HERE / 'context.json').write_text(json.dumps(context, indent=2) + '\n')
    print(json.dumps({'runtime_revision': revision, 'sealed_session_inputs': len(session),
                      'retail_pages': len(comparison), 'retail_observations': sum(row['observations'] for row in comparison)}))


if __name__ == '__main__':
    main()
