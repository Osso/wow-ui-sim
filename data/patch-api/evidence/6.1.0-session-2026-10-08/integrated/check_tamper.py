"""Reject altered own historical discovery evidence, restoring original bytes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def main():
    path = HERE.parent / 'p610-discovery.txt'
    original = path.read_bytes()
    try:
        path.write_bytes(original + b'\nTampered own historical discovery.\n')
        result = subprocess.run([sys.executable, '-B', str(HERE.parent / 'validate.py')],
                                cwd=ROOT, env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'),
                                capture_output=True, text=True)
        assert result.returncode != 0 and 'AssertionError' in result.stderr
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original
    receipt = {'exit': result.returncode, 'stderr': result.stderr,
               'restored_sha256': hashlib.sha256(original).hexdigest(),
               'tampered_path': path.relative_to(ROOT).as_posix()}
    (HERE / 'tamper-proof.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('PASS: own historical log tampering rejected; original bytes restored')


if __name__ == '__main__':
    main()
