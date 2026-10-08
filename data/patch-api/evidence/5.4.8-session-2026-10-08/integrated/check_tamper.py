"""Own-log tampering must fail before historical or shared-input checks."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent

if __name__ == '__main__':
    path = HERE / 'all-sweeps.txt'
    original = path.read_bytes()
    try:
        path.write_bytes(original + b'\nSynthetic own-log tamper.\n')
        result = subprocess.run([sys.executable, '-B', str(HERE / 'validate.py')], cwd=ROOT,
                                capture_output=True, text=True)
        print(result.stdout, result.stderr)
        assert result.returncode != 0
        assert 'own evidence tamper: all-sweeps.txt' in result.stderr
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original
    print('PASS: tampering rejected; original bytes restored')
