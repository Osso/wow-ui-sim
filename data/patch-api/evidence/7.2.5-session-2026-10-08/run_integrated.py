"""Refresh integrated proof separately from immutable original audit receipts."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FRESH = HERE / 'integrated'
MASTER = Path('/tmp/p725-master-1ade15b52')
MASTER_REVISION = '1ade15b526f897e33c588409b64d8799d6942fa1'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p725-page'


def read(path):
    return json.loads(path.read_text())


def dump(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n')


def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def prepare_sources():
    FRESH.mkdir(exist_ok=True)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    reproduction = load('reproduce_sources')
    reproduction.HERE = FRESH
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    print(reproduction.reproduce(extractor, revision))
    extension = FRESH / 'extension'
    extension.mkdir(exist_ok=True)
    dump(extension / 'p725-register-reproduction.json', read(HERE / 'p725-register-reproduction.json'))
    summaries = []
    for row in read(HERE / 'p725-sweep-summary.json'):
        patch = row['test'].removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        summaries.append({'patch': patch, 'rows': row['rows'], 'ok': row['ok'],
                          'gaps': row['gaps'], 'result': 'pass'})
    dump(extension / 'p725-sweep-summary.json', summaries)
    return revision


def run_commands():
    revision = prepare_sources()
    proof = load('run_proof')
    proof.EVIDENCE = FRESH
    os.environ['CARGO_TARGET_DIR'] = TARGET
    os.environ['PYTHONDONTWRITEBYTECODE'] = '1'
    results = {}

    def run(label, command, source_revision=revision):
        code = proof.run_proof(label, command)
        receipt_path = FRESH / (label + '.proof.json')
        receipt = read(receipt_path)
        receipt['source_revision'] = source_revision
        dump(receipt_path, receipt)
        results[label] = code
        return code

    # Verify the exact archived master source before exercising its failure boundary.
    tracked = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', MASTER_REVISION], cwd=ROOT, text=True)
    verified = 0
    for relative in tracked.splitlines():
        path = MASTER / relative
        if path.is_symlink():
            continue
        expected = subprocess.check_output(['git', 'show', f'{MASTER_REVISION}:{relative}'], cwd=ROOT)
        assert path.read_bytes() == expected, relative
        verified += 1
    dump(FRESH / 'master-source-verification.json',
         {'source_revision': MASTER_REVISION, 'files_verified': verified, 'exit': 0})
    master_args = ['--manifest-path', str(MASTER / 'Cargo.toml')]
    run('master-garrison', ['cargo', 'test', *master_args, '--test', 'prefork_full_ui', '--',
                            'blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors'], MASTER_REVISION)
    run('master-build', ['cargo', 'build', *master_args, '--bin', 'wow-sim'], MASTER_REVISION)
    run('master-startup-addons', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'], MASTER_REVISION)

    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        source = path.read_text()
        out_env = re.search(r'out_env: "([A-Z0-9_]+)"', source).group(1)
        os.environ[out_env] = str(FRESH / (path.stem + '-results.json'))
    commands = [('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
                ('integration-p725', ['cargo', 'test', '--test', 'integration', '--', 'patch_7_2_5']),
                ('prefork-p725', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_2_5'])]
    for harness in ('integration', 'prefork_full_ui'):
        for selection in ('garrison', 'order_hall', 'anima'):
            commands.append((harness + '-' + selection,
                             ['cargo', 'test', '--test', harness, '--', selection]))
    for name in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        commands.append((name + '-fixtures', ['python3', '-B', str(ROOT / 'tools' / ('test_' + name + '.py'))]))
    commands.extend([
        ('format', ['cargo', 'fmt', '--check']),
        ('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
        ('retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
        ('startup-addons', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
        ('extend-receipts', ['python3', '-B', str(ROOT / 'tools/extend_patch_audit_receipts.py'),
                             str(ROOT), str(FRESH / 'extension'), 'p725',
                             'Merged 7.3.0 at ' + MASTER_REVISION, '7.2.5']),
    ])
    for label, command in commands:
        run(label, command)
    os.environ['P725_SWEEP_REGISTER'] = str(HERE / 'p725-negative-register.json')
    os.environ['P725_SWEEP_OUT'] = str(FRESH / 'negative-results.json')
    run('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_2_5_publication_sweep'])
    dump(FRESH / 'command-results.json', results)
    dump(FRESH / 'context.json', {'runtime_revision': revision, 'master_revision': MASTER_REVISION,
                                'retained_historical_context': '../p725-context.json',
                                'expected_exits': {label: (1 if label in ('master-garrison', 'prefork_full_ui-garrison', 'negative') else 0)
                                                   for label in results}})
    print(json.dumps(results, sort_keys=True))
    return 0 if results == read(FRESH / 'context.json')['expected_exits'] else 1


if __name__ == '__main__':
    sys.exit(run_commands())
