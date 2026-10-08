"""Seal completed logs and pinned shared inputs; fail if any proof is unfinished."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '896086537a2b3c1ead5886d5ae3e430d56e7ef20'
LABELS = ['all-sweeps', 'client-lines', 'mists-pages', 'mists-check', 'negative', 'tools-tests', 'format']


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def main():
    receipts = {label: read(label + '.proof.json') for label in LABELS}
    revision = receipts['mists-pages']['revision']
    for receipt in receipts.values():
        assert receipt['exit'] == receipt['expected_exit']
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
    comparison = []
    names = git('ls-tree', '-r', '--name-only', BASE, 'data/patch-api/sources').decode().splitlines()
    for name in sorted(path for path in names if path.endswith('-wikitext-register.json')):
        register = json.loads(git('show', BASE + ':' + name))
        if register.get('client_line', 'retail') != 'retail':
            continue
        patch = register['patch']
        result_name = 'patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'
        before = read('master/results/' + result_name)
        after = read('results/' + result_name)
        assert before == after, patch
        comparison.append({'patch': patch, 'observations': len(after),
                           'gaps': sorted(key for key, value in after.items() if not value['ok']), 'identical': True})
    comparison_path = HERE / 'gap-comparison.json'
    previous_comparison = comparison_path.read_bytes() if comparison_path.exists() else None
    comparison_path.write_text(json.dumps(comparison, indent=2) + '\n')
    proofs = {}
    for label, receipt in receipts.items():
        policy = {'command': receipt['command'], 'exit': receipt['expected_exit']}
        if label in ['all-sweeps', 'client-lines', 'mists-pages']:
            text = (HERE / receipt['log']).read_text()
            policy['passed_cases'] = sorted(set(re.findall(r'^test (\S+) (?:- should panic )?\.\.\. ok$', text, re.M)))
        proofs[label] = policy
    shared_paths = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').decode().splitlines()
    shared_paths.extend(['tests/patch_5_5_2_publication_sweep.rs', 'tests/patch_5_5_3_publication_sweep.rs', 'tests/patch_5_5_4_publication_sweep.rs',
                         'tests/publication_sweep_client_lines.rs', 'tests/data/patch_5_5_2_sweep_known_gaps.json',
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
    context_path = HERE / 'context.json'
    previous_context = context_path.read_bytes() if context_path.exists() else None
    context_path.write_text(json.dumps(context, indent=2) + '\n')
    print(json.dumps({'runtime_revision': revision, 'sealed_session_inputs': len(session),
                      'retail_pages': len(comparison), 'retail_observations': sum(row['observations'] for row in comparison)}))


if __name__ == '__main__':
    main()
