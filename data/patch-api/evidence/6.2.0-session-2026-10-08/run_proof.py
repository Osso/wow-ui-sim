"""File-backed asynchronous commands; inspect receipts without poll-waiting."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p620-page'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run_worker(label, command):
    revision = git('rev-parse', 'HEAD')
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1',
               P620_SWEEP_OUT=str(HERE / 'patch_6_2_0_publication_sweep-results.json'))
    env.pop('WOW_SIM_NO_ADDONS', None)
    env.pop('WOW_SIM_NO_SAVED_VARS', None)
    for source in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', source.read_text())
        if match:
            env[match[1]] = str(HERE / (source.stem + '-results.json'))
    log = HERE / (label + '.txt')
    with log.open('wb') as output:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=output,
                                stderr=subprocess.STDOUT, check=False)
    receipt = {'command': command, 'revision': revision, 'exit': result.returncode,
               'log': log.name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'invalidated': False}
    (HERE / (label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--worker', action='store_true')
    parser.add_argument('label')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if args.worker:
        run_worker(args.label, command)
        return
    assert command, 'command required'
    assert not (HERE / (args.label + '.proof.json')).exists(), 'proof already exists'
    with (HERE / (args.label + '.launch.txt')).open('wb') as output:
        process = subprocess.Popen([sys.executable, '-B', __file__, '--worker', args.label,
                                    '--', *command], cwd=ROOT, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'label': args.label}))


if __name__ == '__main__':
    main()
