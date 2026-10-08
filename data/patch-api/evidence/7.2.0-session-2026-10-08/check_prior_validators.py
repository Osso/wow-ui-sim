"""Validate every retained audit from the fixed base, without modifying its evidence."""
from pathlib import Path
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '1ade15b526f897e33c588409b64d8799d6942fa1'


def main():
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BASE,
                                     'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    results = {}
    for path in paths:
        if not path.endswith('/validate.py'):
            continue
        result = subprocess.run([sys.executable, '-B', str(ROOT / path)], cwd=ROOT,
                                capture_output=True, text=True)
        results[path] = {'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}
        print(path, result.returncode, flush=True)
    (HERE / 'p720-prior-validator-matrix.json').write_text(json.dumps(results, indent=2) + '\n')
    assert results and all(row['exit'] == 0 for row in results.values()), results


if __name__ == '__main__':
    main()
