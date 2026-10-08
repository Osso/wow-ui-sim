"""Seal completed 5.4.0 receipts without moving historical shared-file scope."""
import hashlib
import json
import re
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = 'bebcc5830'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def main():
    revision = git('rev-parse', 'HEAD').decode().strip()
    receipts = ['p540-python-fixtures', 'p540-reproduction', 'p540-all-sweeps',
                'p540-prefork-behavior', 'p540-integration-behavior',
                'p540-integration-instance', 'p540-integration-forbidden', 'p540-lib-frame-state',
                'p540-format', 'p540-mists-check']
    for name in receipts:
        receipt = json.loads((HERE / (name + '.proof.json')).read_text())
        assert receipt['exit'] == 0 and not receipt['invalidated'], name
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256']
        if receipt['command'][:2] == ['cargo', 'test']:
            match = re.search(r'test result: ok\. (\d+) passed', log.decode())
            assert match and int(match[1]) > 0, name
    negative = json.loads((HERE / 'p540-negative.proof.json').read_text())
    assert negative['exit'] != 0
    rows = []
    paths = git('ls-tree', '-r', '--name-only', revision, 'tests').decode().splitlines()
    for path in paths:
        if not Path(path).match('patch_*_publication_sweep.rs'):
            continue
        name = Path(path).stem + '-results.json'
        observations = json.loads((HERE / name).read_text())
        rows.append({'test': path, 'results': name, 'observations': len(observations),
                     'gaps': sum(not row['ok'] for row in observations.values())})
    dump('p540-sweep-summary.json', rows)
    shared_paths = git('ls-tree', '-r', '--name-only', revision,
                       'data/patch-api/sources', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs').decode().splitlines()
    shared = [{'path': path, 'sha256': digest(git('show', f'{revision}:{path}'))}
              for path in shared_paths if path.startswith('data/patch-api/sources/')
              or path in ('tests/common/publication_sweep.rs', 'tests/patch_5_4_0_behavior.rs')
              or Path(path).match('patch_*_publication_sweep.rs')
              or Path(path).match('patch_*_sweep_known_gaps.json')
              or path in ('Cargo.toml', 'Cargo.lock', 'build.rs', 'tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
                          'tools/patch_audit_validation.py', 'tools/check_patch_validators.py')]
    excluded = {'p540-seal.json', 'validate.py', 'seal_proofs.py', 'p540-proof-ledger.md'}
    own = [{'path': path.name, 'sha256': digest(path.read_bytes())}
           for path in sorted(HERE.iterdir()) if path.is_file() and path.name not in excluded
           and not path.name.endswith('.launcher.txt')]
    dump('p540-seal.json', {'base_revision': git('rev-parse', BASE).decode().strip(),
                          'code_revision': revision, 'own_files': own,
                          'shared_files': shared, 'passing_receipts': receipts})
    print(json.dumps({'revision': revision, 'sweeps': len(rows), 'own_files': len(own),
                      'shared_files': len(shared), 'passing_receipts': len(receipts)}))


if __name__ == '__main__':
    main()
