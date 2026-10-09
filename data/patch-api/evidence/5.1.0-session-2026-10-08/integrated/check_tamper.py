"""Reject historical and integrated owned-log mutations, restoring exact bytes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def check_log(validator, log, output, expected_error):
    before = log.read_bytes()
    try:
        log.write_bytes(before + b'\nP510 controlled owned-log tamper\n')
        result = subprocess.run([sys.executable, '-B', str(validator)], cwd=ROOT,
                                capture_output=True, text=True)
        output.write_text(result.stdout + result.stderr)
        assert result.returncode != 0, 'tampered log was accepted'
        assert expected_error in result.stderr and log.name in result.stderr, result.stderr
    finally:
        log.write_bytes(before)
    assert digest(log.read_bytes()) == digest(before), 'log restoration failed'
    return {'validator': str(validator.relative_to(ROOT)), 'log': str(log.relative_to(ROOT)),
            'exit': result.returncode, 'expected_error': expected_error,
            'restored_sha256': digest(before), 'restored': True}


def main():
    rows = [
        check_log(HISTORY / 'validate.py', HISTORY / 'p510-all-sweeps.log',
                  HERE / 'historical-tamper-validator.txt', 'own input changed'),
        check_log(HERE / 'validate.py', HERE / 'own-sweep.txt',
                  HERE / 'tamper-validator.txt', 'integrated artifact drift'),
    ]
    (HERE / 'tamper-proof.json').write_text(json.dumps({'status': 'PASS', 'controls': rows}, indent=2) + '\n')
    print(json.dumps({'status': 'PASS', 'rejected_and_restored': len(rows)}))


if __name__ == '__main__':
    main()
