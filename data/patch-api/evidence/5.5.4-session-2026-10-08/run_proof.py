"""Submit file-backed proof commands without poll-waiting."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGETS = {
    'retail': '/home/osso/.cache/wow-ui-sim-targets/p554-page',
    'mists': '/home/osso/.cache/wow-ui-sim-targets/p554-page-mists',
}


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run_worker(args):
    revision = git('rev-parse', 'HEAD')
    result_dir = HERE / args.label
    result_dir.mkdir(exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=TARGETS[args.profile],
               PYTHONDONTWRITEBYTECODE='1')
    for source in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', source.read_text())
        if match:
            env[match[1]] = str(result_dir / (source.stem + '-results.json'))
    log = HERE / (args.label + '.log')
    with log.open('wb') as output:
        result = subprocess.run(args.command, cwd=ROOT, env=env, stdout=output,
                                stderr=subprocess.STDOUT, check=False)
    receipt = {'command': args.command, 'revision': revision,
               'profile': args.profile, 'target': TARGETS[args.profile],
               'exit': result.returncode, 'log': log.name,
               'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'invalidated': False}
    (HERE / (args.label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--worker', action='store_true')
    parser.add_argument('--profile', choices=TARGETS, default='retail')
    parser.add_argument('label')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command[:1] == ['--']:
        args.command = args.command[1:]
    assert args.command, 'command required'
    if args.worker:
        run_worker(args)
        return
    assert not (HERE / (args.label + '.log')).exists(), 'label already used'
    with (HERE / (args.label + '.launch.log')).open('wb') as output:
        process = subprocess.Popen(
            [sys.executable, '-B', __file__, '--worker', '--profile', args.profile,
             args.label, '--', *args.command], cwd=ROOT, stdout=output,
            stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'label': args.label}))


if __name__ == '__main__':
    main()
