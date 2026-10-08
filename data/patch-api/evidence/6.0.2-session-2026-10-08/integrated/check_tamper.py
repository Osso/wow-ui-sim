"""Reject alterations to own historical and integrated receipts, restoring exact bytes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def run(path):
    return subprocess.run([sys.executable, '-B', str(path)], cwd=ROOT,
                          env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'),
                          capture_output=True, text=True)


def main():
    results = []
    for validator, target in [(HISTORY / 'validate.py', HISTORY / 'p602-all-sweeps.txt'),
                              (HERE / 'validate.py', HERE / 'all-sweeps.txt')]:
        baseline = run(validator)
        assert baseline.returncode == 0, baseline.stderr
        content = target.read_bytes()
        try:
            target.write_bytes(content + b'\nTampered own receipt.\n')
            mutated = run(validator)
            assert mutated.returncode != 0 and target.name in mutated.stderr, mutated.stderr
        finally:
            tampered_bytes = target.read_bytes()
            target.write_bytes(content)
        assert target.read_bytes() == content
        results.append({'validator': str(validator.relative_to(ROOT)),
                        'receipt': str(target.relative_to(ROOT)), 'baseline_exit': baseline.returncode,
                        'tampered_exit': mutated.returncode,
                        'sha256_after_restore': hashlib.sha256(content).hexdigest(),
                        'stdout': mutated.stdout, 'stderr': mutated.stderr})
    path = HERE / 'tamper-proof.json'
    previous = path.read_bytes() if path.exists() else None
    path.write_text(json.dumps(results, indent=2) + '\n')
    print('PASS: historical and integrated receipt tampering rejected; bytes restored')


if __name__ == '__main__':
    main()
