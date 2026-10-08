"""Retain a streamed command log and receipt; --start launches without waiting."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p530-page'


def run_proof(name, command):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    started = time.monotonic()
    log = EVIDENCE / f'{name}.txt'
    environment = {key: value for key, value in os.environ.items()
                   if key.startswith(('P530_', 'PATCH_', 'WOW_SIM_', 'PREFORK_'))
                   or key.endswith('_SWEEP_OUT')}
    with log.open('w') as output:
        result = subprocess.run(command, cwd=ROOT,
                                env=dict(os.environ, CARGO_TARGET_DIR=TARGET,
                                         PYTHONDONTWRITEBYTECODE='1'),
                                stdout=output, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'scope': name,
               'cwd': '.', 'target': 'dedicated p530-page target (external to repository)',
               'environment': environment, 'exit': result.returncode,
               'log': log.name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'seconds': time.monotonic() - started, 'invalidated': False}
    (EVIDENCE / f'{name}.proof.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(name, result.returncode)
    return result.returncode


if __name__ == '__main__':
    arguments = sys.argv[1:]
    if arguments[0] == '--start':
        name, *command = arguments[1:]
        with (EVIDENCE / f'{name}.launcher.txt').open('w') as output:
            process = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), name, *command],
                                       cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                                       start_new_session=True)
        print(json.dumps({'name': name, 'pid': process.pid, 'log': f'{name}.txt'}))
    else:
        sys.exit(run_proof(arguments[0], arguments[1:]))
