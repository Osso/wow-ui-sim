"""Run exactly the validator inventory at pinned integration master."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
BASE = 'bebcc5830dcb21e768a2e3c365105351a400fccb'


def main():
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BASE,
                                    'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    validators = [path for path in paths if path.endswith('/validate.py')]
    inventory = {'revision': BASE, 'paths': validators}
    (HERE / 'prior-validator-inventory.json').write_text(json.dumps(inventory, indent=2) + '\n')
    results = []
    for number, path in enumerate(validators):
        result = subprocess.run([sys.executable, '-B', str(ROOT / path)], cwd=ROOT, capture_output=True)
        log = HERE / f'prior-{number}.txt'
        log.write_bytes(result.stdout + result.stderr)
        results.append({'path': path, 'exit': result.returncode, 'log': log.name,
                        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
    (HERE / 'prior-results.json').write_text(json.dumps(results, indent=2) + '\n')
    assert results and all(row['exit'] == 0 for row in results), results
    print(f'PASS: {len(results)} pinned prior validators')


if __name__ == '__main__':
    main()
