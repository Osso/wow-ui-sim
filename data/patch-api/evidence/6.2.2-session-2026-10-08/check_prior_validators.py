"""Record the complete prior-validator set at the fixed audit base revision."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = 'dfede62de1f48ef2af1a4d37a18d274e26df3d81'


def main():
    names = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', BASE, 'data/patch-api/evidence'],
        cwd=ROOT, text=True,
    ).splitlines()
    validators = [name for name in names if name.endswith('/validate.py')]
    rows = []
    for number, name in enumerate(validators):
        log = HERE / f'p622-prior-validator-{number:02}.log'
        with log.open('wb') as handle:
            result = subprocess.run([sys.executable, '-B', str(ROOT / name)], cwd=ROOT,
                                    stdout=handle, stderr=subprocess.STDOUT)
        rows.append({'path': name, 'exit': result.returncode, 'log': log.name,
                     'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
    (HERE / 'p622-prior-validator-matrix.json').write_text(
        json.dumps({'scope_revision': BASE, 'validators': rows}, indent=2) + '\n')
    print(json.dumps({'validators': len(rows),
                      'failed': [row['path'] for row in rows if row['exit'] != 0]}))
    assert all(row['exit'] == 0 for row in rows)


if __name__ == '__main__':
    main()
