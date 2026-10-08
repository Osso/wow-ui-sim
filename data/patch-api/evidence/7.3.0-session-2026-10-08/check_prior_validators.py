"""Run each validator present at the page's base revision, retaining full output."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = 'af7a101e20824f92798885f41b47b147418a5f48'


def main():
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BASE,
                                    'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    rows = []
    for name in names:
        if not name.endswith('/validate.py'):
            continue
        command = [sys.executable, '-B', str(ROOT / name)]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        rows.append({'path': name, 'command': command, 'exit': result.returncode,
                     'stdout': result.stdout, 'stderr': result.stderr})
    (HERE / 'p730-prior-validator-matrix.json').write_text(
        json.dumps({'base_revision': BASE, 'validators': rows}, indent=2) + '\n')
    print(json.dumps({'total': len(rows), 'passed': sum(row['exit'] == 0 for row in rows),
                      'failed': [row['path'] for row in rows if row['exit'] != 0]}))
    assert all(row['exit'] == 0 for row in rows)


if __name__ == '__main__':
    main()
