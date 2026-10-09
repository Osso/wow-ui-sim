"""Seal committed-input integration receipts; counts come from observations and files."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = '7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def read(name):
    return json.loads((HERE / name).read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def main():
    revision = git('rev-parse', 'HEAD')
    rows = []
    for path in git('ls-tree', '-r', '--name-only', revision, 'tests').splitlines():
        if not re.fullmatch(r'tests/patch_.*_publication_sweep.rs', path):
            continue
        stem = Path(path).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        observations = read(stem + '-results.json')
        identical = read('master/' + stem + '-results.json') == observations if patch != '5.1.0' else None
        assert identical is not False, stem
        rows.append({'patch': patch, 'observations': len(observations),
                     'gaps': sorted(key for key, value in observations.items() if not value['ok']),
                     'unchanged_vs_master': identical})
    dump('gap-comparison.json', rows)
    diagnostic = {'diagnostic-own-sweep', 'scan-tool-build', 'extend-receipts'}
    excluded = {'context.json', 'validator-gate.txt', 'validator-gate.proof.json',
                'validator-gate.launcher.txt', 'gate-summary.json', 'tamper-proof.json',
                'tamper-validator.txt', 'historical-tamper-validator.txt'}
    proofs = sorted(path.name.removesuffix('.proof.json') for path in HERE.glob('*.proof.json')
                    if path.name.removesuffix('.proof.json') not in diagnostic
                    and path.name not in excluded)
    for name in proofs:
        receipt = read(name + '.proof.json')
        assert not receipt['invalidated']
        assert receipt['exit'] in (1, 101) if name == 'negative' else receipt['exit'] == 0
    dump('context.json', {
        'master_revision': MASTER, 'runtime_revision': revision, 'wiki_revision': revision,
        'historical_dispatcher_sha256': hashlib.sha256((HERE.parent / 'validate.py').read_bytes()).hexdigest(),
        'directory_trees': {path: git('rev-parse', revision + ':' + path)
                            for path in ['src', 'tests', 'tools', 'data/patch-api/sources']},
        'negative_id': 'wt-global-api-HasVehicleActionBar-11', 'proofs': proofs,
        'own_files': {path.relative_to(HERE).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                      for path in sorted(HERE.rglob('*'))
                      if path.is_file() and path.name not in excluded}})
    print(json.dumps({'runtime_revision': revision, 'sweeps': len(rows), 'proofs': len(proofs),
                      'largest_evidence_file': max(path.stat().st_size for path in HERE.parent.rglob('*')
                                                   if path.is_file())}))


if __name__ == '__main__':
    main()
