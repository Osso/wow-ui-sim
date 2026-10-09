"""Record one bounded proof against an explicit revision; stream both channels to log."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    name, revision, *command = sys.argv[1:]
    if not command:
        raise ValueError('proof command required')
    log = HERE / (name + '.log')
    started = time.time()
    with log.open('w') as output:
        result = subprocess.run(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
    receipt = {'revision': revision, 'command': command, 'exit': result.returncode,
               'elapsed_seconds': time.time() - started, 'log': log.name,
               'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'env': {name: value for name, value in os.environ.items()
                       if name.startswith(('CARGO_TARGET_DIR', 'P510_', 'WOW_SIM_'))}}
    (HERE / (name + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))
    return result.returncode


if __name__ == '__main__':
    sys.exit(main())
