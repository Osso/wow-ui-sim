"""Seal finished integrated proof without rerunning commands."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def read(path):
    return json.loads(path.read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main():
    results = read(HERE / 'command-results.json')
    assert results['negative'] == 1
    assert results['master-all-sweeps'] == 0
    assert all(code == 0 for label, code in results.items() if label != 'negative')
    revision = read(HERE / 'own-sweep.proof.json')['revision']
    master = git('rev-parse', '787b47591')
    receipts = {label: read(HERE / (label + '.proof.json')) for label in results}
    paths = git('ls-tree', '-r', '--name-only', revision, 'tests').splitlines()
    pages = []
    for path in paths:
        if not path.endswith('_publication_sweep.rs'):
            continue
        stem = Path(path).stem
        results = read(HERE / (stem + '-results.json'))
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        if patch != '6.1.0':
            assert results == read(HERE / 'master' / (stem + '-results.json')), patch
        pages.append({'patch': patch, 'observations': len(results),
                      'gaps': sorted(key for key, value in results.items() if not value['ok']),
                      'unchanged_vs_master': patch != '6.1.0'})
    dump('gap-comparison.json', pages)
    dump('context.json', {'master_revision': master, 'runtime_revision': revision,
                          'receipts': receipts,
                          'input_trees': {scope: git('rev-parse', revision + ':' + scope)
                                          for scope in ('src', 'tests', 'tools')}})
    excluded = {'artifact-hashes.json', 'context.json', 'seal_receipts.py',
                'portability.json', 'portability.txt', 'portability.proof.json',
                'portability.launcher.txt', 'final-validation.txt', 'tamper-proof.json'}
    hashes = {path.relative_to(HERE).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in HERE.rglob('*') if path.is_file() and path.name not in excluded}
    dump('artifact-hashes.json', hashes)
    print(json.dumps({'pages': len(pages), 'other_pages_unchanged': len(pages) - 1,
                      'observations': sum(row['observations'] for row in pages),
                      'receipts': len(receipts), 'sealed_artifacts': len(hashes)}))


if __name__ == '__main__':
    main()
