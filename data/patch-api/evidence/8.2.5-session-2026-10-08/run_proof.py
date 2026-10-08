"""Run one scoped proof command, retaining complete output and revision receipt."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p825-page'


def run_proof(name, command, extra_env=None, expected_exit=0):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, **(extra_env or {}))
    started = time.monotonic()
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log = EVIDENCE / f'{name}.txt'
    log.write_text(result.stdout + result.stderr)
    receipt = {
        'command': command, 'revision': revision, 'scope': name,
        'cwd': str(ROOT), 'target': TARGET, 'environment': extra_env or {},
        'exit': result.returncode,
        'expected_exit': expected_exit, 'log': log.name,
        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
        'invalidated': False, 'seconds': time.monotonic() - started,
    }
    (EVIDENCE / f'{name}.proof.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(name, result.returncode, flush=True)
    print(result.stdout[-3000:] + result.stderr[-1000:])
    return result


if __name__ == '__main__':
    run_proof(sys.argv[1], sys.argv[2:])
