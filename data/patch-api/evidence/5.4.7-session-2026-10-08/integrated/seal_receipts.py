"""Seal committed integration inputs after every requested receipt exists."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = 'bebcc5830dcb21e768a2e3c365105351a400fccb'
REQUIRED = ['own-sweep', 'all-sweeps', 'prefork-patch_5_4_7', 'integration-patch_5_4_7',
            'negative', 'reproduction', 'prior-validators', 'checks', 'master-all-sweeps',
            'format', 'mists-check', 'test_check_patch_validators', 'test_extract_patch_non_inventory',
            'test_gen_patch_wikitext_register', 'test_patch_audit_validation', 'test_patch_warlords_register']


def read(name):
    return json.loads((HERE / name).read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def digest(content):
    return hashlib.sha256(content).hexdigest()


def summarize():
    receipts = {label: read(label + '.proof.json') for label in REQUIRED}
    for label, row in receipts.items():
        assert row['exit'] in ((1, 101) if label == 'negative' else (0,)), (label, row['exit'])
        assert row['log_sha256'] == digest((HERE / row['log']).read_bytes())
    dump('receipts.json', receipts)
    lines = ['# 5.4.7 integrated command ledger', '',
             'Base: `' + MASTER + '`. Own source/test scope: `' + receipts['own-sweep']['revision'] + '`.',
             'All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p547-page`.',
             'Commands ran sequentially in the logged asynchronous driver; no poll-wait or repeated broad checks.', '',
             '| Receipt | Command | Revision | Exit |', '|---|---|---|---|']
    for label, row in receipts.items():
        lines.append('| ' + label + ' | `' + ' '.join(row['command']) + '` | `' + row['revision'] + '` | ' + str(row['exit']) + ' |')
    lines.extend(['', 'Full environment, duration, log digest and revision are in each `.proof.json`.',
                  'Integration `patch_5_4_7` selects zero cases; prefork owns both 5.4.7 tests. No integration behavior claim.',
                  'Negative control must fail with exactly one added gap (2 → 3). Three inherited extract failures are unchanged.',
                  'No runtime, shared tool, vendor or retirement changes. No startup comparison or full integration-suite claim.',
                  'Later evidence/wiki/validator-only commits do not invalidate the pinned Rust, shared-tool or source scopes.'])
    (HERE / 'command-ledger.md').write_text('\n'.join(lines) + '\n')
    pages = []
    for path in sorted(HERE.glob('patch_*_publication_sweep-results.json')):
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep-results').replace('_', '.')
        results = read(path.name)
        same = None if patch == '5.4.7' else results == read('master/' + path.name)
        assert same is not False, patch
        pages.append({'patch': patch, 'observations': len(results),
                      'gaps': sorted(key for key, row in results.items() if not row['ok']),
                      'unchanged_vs_master': same})
    assert len(pages) == 56
    dump('gap-comparison.json', pages)
    print(json.dumps({'receipts': len(receipts), 'sweep_pages': len(pages),
                      'other_sweeps_unchanged': sum(row['unchanged_vs_master'] is True for row in pages)}))


def seal():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, str(HERE.relative_to(ROOT))], cwd=ROOT, text=True).splitlines()
    seals = {}
    for name in paths:
        path = ROOT / name
        if path.name in ('context.json', 'validate.py'):
            continue
        content = subprocess.check_output(['git', 'show', revision + ':' + name], cwd=ROOT)
        assert content == path.read_bytes(), name
        seals[path.relative_to(HERE).as_posix()] = digest(content)
    assert 'receipts.json' in seals and 'gap-comparison.json' in seals
    context = {'runtime_revision': read('own-sweep.proof.json')['revision'],
               'master_revision': MASTER, 'seal_revision': revision, 'required_receipts': REQUIRED,
               'expected_exits': {'negative': read('negative.proof.json')['exit']},
               'wrapper_sha256': digest((HERE.parent / 'validate.py').read_bytes()), 'seals': seals}
    dump('context.json', context)
    print('CONTEXT_SHA256 = ' + repr(digest((HERE / 'context.json').read_bytes())))


if __name__ == '__main__':
    {'summarize': summarize, 'seal': seal}[sys.argv[1]]()
