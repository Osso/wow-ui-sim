"""Record every prior validator from a fixed Git snapshot, never a live glob."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
SCOPE_REVISION = 'f260499ee'


def main():
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', SCOPE_REVISION,
                                     'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    paths = [name for name in names if Path(name).name in ('validate.py', 'validate_integrated.py')]
    matrix = {}
    for number, name in enumerate(paths):
        log = HERE / 'validators' / (str(number) + '.txt')
        log.parent.mkdir(exist_ok=True)
        with log.open('w') as output:
            result = subprocess.run([sys.executable, '-B', str(ROOT / name)], cwd=ROOT,
                                    stdout=output, stderr=subprocess.STDOUT)
        matrix[name] = {'exit': result.returncode, 'validator_sha256': hashlib.sha256((ROOT / name).read_bytes()).hexdigest(),
                        'log': log.relative_to(HERE).as_posix(), 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()}
        print(name, result.returncode, flush=True)
    (HERE / 'prior-validator-matrix.json').write_text(json.dumps(matrix, indent=2) + '\n')
    sys.exit(int(any(row['exit'] for row in matrix.values())))


if __name__ == '__main__':
    main()
