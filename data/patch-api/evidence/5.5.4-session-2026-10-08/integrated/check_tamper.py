"""Prove integrated and historical own-log seals reject tampering, then restore bytes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def main():
    results = []
    cases = [(HERE / 'all-sweeps.txt', HERE / 'validate.py'),
             (HERE.parent / 'p554-retail-final.log', HERE.parent / 'validate.py')]
    for number, (path, validator) in enumerate(cases):
        original = path.read_bytes()
        try:
            path.write_bytes(original + b'\nOwn-session tamper control.\n')
            result = subprocess.run([sys.executable, '-B', str(validator)], cwd=ROOT, capture_output=True)
        finally:
            path.write_bytes(original)
        assert result.returncode != 0 and path.name.encode() in result.stderr
        assert path.read_bytes() == original
        log = HERE / f'tamper-{number}.txt'
        log.write_bytes(result.stdout + result.stderr)
        results.append({'input': str(path.relative_to(HERE.parent)), 'validator': str(validator.relative_to(HERE.parent)),
                        'exit': result.returncode, 'restored_sha256': hashlib.sha256(original).hexdigest(),
                        'log': log.name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
    (HERE / 'tamper-results.json').write_text(json.dumps(results, indent=2) + '\n')
    print('PASS: integrated and historical tampering rejected; exact bytes restored')


if __name__ == '__main__':
    main()
