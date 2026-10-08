"""Run the validator set defined by the recorded master tree, once."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
REVISION = '787b47591'


def main():
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', REVISION,
                                     'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    matrix = []
    for name in sorted(path for path in paths if Path(path).name in ('validate.py', 'validate_integrated.py')):
        code = subprocess.check_output(['git', 'show', REVISION + ':' + name], cwd=ROOT)
        assert (ROOT / name).read_bytes() == code, name
        log = HERE / ('prior-' + str(len(matrix)) + '.txt')
        with log.open('wb') as output:
            result = subprocess.run([sys.executable, '-B', str(ROOT / name)], cwd=ROOT,
                                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'),
                                    stdout=output, stderr=subprocess.STDOUT)
        matrix.append({'path': name, 'revision': REVISION, 'code_sha256': hashlib.sha256(code).hexdigest(),
                       'exit': result.returncode, 'log': log.name,
                       'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
        print(name, result.returncode, flush=True)
    (HERE / 'prior-validator-matrix.json').write_text(json.dumps(matrix, indent=2) + '\n')
    print('FINISHED', flush=True)


if __name__ == '__main__':
    main()
