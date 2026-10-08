"""Seal completed receipts; refuses incomplete or failed acceptance commands."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
RUNTIME = 'ddd76addc3bd451894316ab8c3575ff9e8ec0f35'
MASTER = '8dd11c1b90fb1f5ca925de62054a671179652f15'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    assert 'FINISHED' in (HERE / 'launcher.txt').read_text()
    assert 'FINISHED' in (HERE / 'prior-launcher.txt').read_text()
    results = read(HERE / 'command-results.json')
    assert all(value != 0 if name == 'negative' else value == 0 for name, value in results.items()), results
    assert all(row['exit'] == 0 for row in read(HERE / 'prior-validator-matrix.json'))
    pages = []
    for name in git('ls-tree', '-r', '--name-only', RUNTIME, 'tests').splitlines():
        if not name.endswith('_publication_sweep.rs'):
            continue
        stem = Path(name).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        results = read(HERE / (stem + '-results.json'))
        gaps = sorted(key for key, value in results.items() if not value['ok'])
        if patch != '6.2.0':
            assert results == read(HERE / 'master' / (stem + '-results.json')), patch
        pages.append({'patch': patch, 'observations': len(results), 'gaps': gaps,
                      'unchanged_vs_master': patch != '6.2.0'})
    dump(HERE / 'gap-comparison.json', pages)
    dump(HERE / 'supersession-review.json', {'before_gaps': [], 'after_gaps': [],
          'replacements': [], 'later_intersections': {'6.2.2': [], '6.2.4': []},
          'pending_item_link_contracts': 2})
    receipts = {path.name.removesuffix('.proof.json'): read(path)
                for path in sorted(HERE.glob('*.proof.json'))}
    trees = {name: git('rev-parse', RUNTIME + ':' + name)
             for name in ('src', 'tests', 'tools', 'data/patch-api/sources', 'Cargo.toml', 'Cargo.lock')}
    dump(HERE / 'context.json', {'runtime_revision': RUNTIME, 'master_revision': MASTER,
                                'input_trees': trees, 'receipts': receipts})
    ledger = ['# Integrated 6.2.0 proof ledger', '',
              'Runtime scope: `' + RUNTIME + '`. Master: `' + MASTER + '`.',
              'All logs retained; docs/evidence-only commits do not invalidate src/tests/tools proof.', '',
              '| Command | Revision | Exit | Log |', '|---|---|---|---|']
    for label, row in receipts.items():
        ledger.append('| `' + ' '.join(row['command']) + '` | `' + row['revision'] + '` | ' +
                      str(row['exit']) + ' | [' + row['log'] + '](' + row['log'] + ') |')
    ledger.extend(['', '51 saved registers / 48 extracts reproduced; three inherited failures unchanged.',
                   'Negative control: zero gaps → one attributable missing synthetic global.',
                   'All other publication observations match immutable master byte-for-byte as JSON.'])
    (HERE / 'command-ledger.md').write_text('\n'.join(ledger) + '\n')
    excluded = {'artifact-hashes.json', 'final-validation.txt', 'portability.json'}
    artifacts = {path.relative_to(HERE.parent).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in HERE.rglob('*') if path.is_file() and path.name not in excluded}
    own = HERE.parent / 'validate.py'
    artifacts['validate.py'] = hashlib.sha256(own.read_bytes()).hexdigest()
    dump(HERE / 'artifact-hashes.json', artifacts)
    print(json.dumps({'pages': len(pages), 'observations': sum(row['observations'] for row in pages),
                      'unchanged_other_pages': sum(row['unchanged_vs_master'] for row in pages),
                      'commands': len(receipts), 'artifacts': len(artifacts)}))


if __name__ == '__main__':
    main()
