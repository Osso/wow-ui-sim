"""Record bounded argv proofs with compact committed tree identities."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p501-page'
SCOPES = ['src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs', 'data/patch-api/sources']


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main():
    label, *command = sys.argv[1:]
    if label == '--detach':
        label = command[0]
        with (HERE / (label + '-launch.log')).open('wb') as log:
            process = subprocess.Popen([sys.executable, '-B', str(Path(__file__).resolve()), *command],
                                       cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                                       start_new_session=True)
        print(json.dumps({'label': label, 'pid': process.pid}))
        return
    revision = git('rev-parse', 'HEAD')
    assert not git('diff', '--name-only', revision, '--', *SCOPES), 'dirty proof inputs'
    identities = {path: git('rev-parse', revision + ':' + path) for path in SCOPES}
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1',
               CARGO_TARGET_DIR=TARGET + ('-mists' if 'client-mists' in command else ''))
    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            env[match[1]] = str(HERE / (label + '-results.json' if match[1] == 'P501_SWEEP_OUT'
                                       else path.stem + '-results.json'))
    if label == 'p501-negative':
        env['P501_SWEEP_REGISTER'] = str(HERE / 'p501-fabricated-register.json')
    log = HERE / (label + '.log')
    with log.open('wb') as handle:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=handle, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'identities': identities,
               'exit': result.returncode, 'log': log.name,
               'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()}
    (HERE / (label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'label': label, 'exit': result.returncode}))
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
