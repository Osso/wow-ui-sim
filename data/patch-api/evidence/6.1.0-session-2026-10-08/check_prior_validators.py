"""Run the exact pre-audit validator set; pin scripts, logs and exits."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '15b417367b68b9c16df9e72fb141c177a2a220de'


def main():
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BASE,
                                     'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    validators = [p for p in names if Path(p).name in ('validate.py', 'validate_integrated.py')]
    rows = []
    for index, path in enumerate(validators):
        script = subprocess.check_output(['git', 'show', f'{BASE}:{path}'], cwd=ROOT)
        assert (ROOT / path).read_bytes() == script, path
        log = HERE / f'p610-prior-validator-{index:02d}.txt'
        with log.open('w') as output:
            result = subprocess.run([sys.executable, '-B', str(ROOT / path)], cwd=ROOT,
                                    stdout=output, stderr=subprocess.STDOUT)
        rows.append({'path': path, 'exit': result.returncode, 'log': log.name,
                     'script_sha256': hashlib.sha256(script).hexdigest(),
                     'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
        print(path, result.returncode, flush=True)
    (HERE / 'p610-other-validator-matrix.json').write_text(json.dumps(rows, indent=2) + '\n')
    return int(any(row['exit'] for row in rows))


if __name__ == '__main__':
    sys.exit(main())
