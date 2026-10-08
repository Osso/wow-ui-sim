"""Retain untruncated output for every prior historical/integrated validator."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    matrix = {}
    for path in sorted((ROOT / 'data/patch-api/evidence').rglob('validate*.py')):
        if path.parent == HERE:
            continue
        result = subprocess.run([sys.executable, '-B', str(path)], cwd=ROOT,
                                capture_output=True, text=True)
        matrix[str(path.relative_to(ROOT))] = {
            'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
        }
    (HERE / 'p703-prior-validator-matrix.json').write_text(json.dumps(matrix, indent=2) + '\n')
    print(json.dumps({path: row['exit'] for path, row in matrix.items()}, indent=2))
    raise SystemExit(any(row['exit'] for row in matrix.values()))


if __name__ == '__main__':
    main()
