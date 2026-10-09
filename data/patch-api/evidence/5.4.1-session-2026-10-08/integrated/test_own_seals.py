"""Reject own-log tampering at its exact seal, restoring all bytes in finally."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def main():
    rows = []
    for label, log, validator, diagnostic in [
        ('historical', HERE.parent / 'own-green.txt', HERE.parent / 'validate.py', 'own evidence drift: own-green.txt'),
        ('integrated', HERE / 'all-sweeps.txt', HERE / 'validate.py', 'own evidence tamper: all-sweeps.txt'),
    ]:
        original = log.read_bytes()
        try:
            log.write_bytes(original + b'\nOWN-LOG-TAMPER-CONTROL\n')
            result = subprocess.run([sys.executable, '-B', str(validator)], cwd=ROOT, capture_output=True, text=True)
            assert result.returncode != 0 and diagnostic in result.stderr, (label, result.stdout, result.stderr)
            output = HERE / (label + '-tamper-validator.txt')
            output.write_text(result.stdout + result.stderr)
        finally:
            log.write_bytes(original)
        assert log.read_bytes() == original
        rows.append({'case': label, 'exit': result.returncode, 'diagnostic': diagnostic,
                     'restored_sha256': hashlib.sha256(original).hexdigest(),
                     'log': output.name, 'log_sha256': hashlib.sha256(output.read_bytes()).hexdigest()})
    (HERE / 'own-log-tamper.json').write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps({'tamper_controls': len(rows), 'rejected': len(rows), 'restored': True}))


if __name__ == '__main__':
    main()
