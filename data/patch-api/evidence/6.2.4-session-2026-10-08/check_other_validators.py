"""Run every prior validator without modifying its evidence."""
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    rows = []
    for path in sorted((ROOT / 'data/patch-api/evidence').rglob('validate*.py')):
        if path.parent == HERE or path.name not in ('validate.py', 'validate_integrated.py'):
            continue
        command = ['python3', '-B', str(path)]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        rows.append({'path': str(path.relative_to(ROOT)), 'command': command, 'revision': revision,
                     'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
    (HERE / 'p624-other-validator-matrix.json').write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps({'validators': len(rows), 'passed': sum(r['exit'] == 0 for r in rows),
                      'failures': [r['path'] for r in rows if r['exit']]}))
    return int(any(r['exit'] for r in rows))


if __name__ == '__main__':
    sys.exit(main())
