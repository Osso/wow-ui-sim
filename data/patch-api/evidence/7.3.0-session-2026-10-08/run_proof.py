"""Run one argv command with complete logs and a revision/scope receipt."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    if sys.argv[1] == '--detach':
        label = sys.argv[2]
        command = [sys.executable, '-B', str(Path(__file__).resolve()), *sys.argv[2:]]
        with (HERE / (label + '-launch.log')).open('wb') as handle:
            process = subprocess.Popen(command, cwd=ROOT, stdout=handle,
                                       stderr=subprocess.STDOUT, start_new_session=True)
        (HERE / (label + '-job.json')).write_text(json.dumps({'pid': process.pid, 'argv': command}) + '\n')
        print(f'Launched {label}: PID {process.pid}')
        return
    label, *command = sys.argv[1:]
    env = os.environ.copy()
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    env['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/p730-page'
    env['P730_SWEEP_OUT'] = str(HERE / (label + '-results.json'))
    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        import re
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match and match[1] != 'P730_SWEEP_OUT':
            env[match[1]] = str(HERE / (path.stem + '-results.json'))
    names = subprocess.check_output(['git', 'ls-files', 'src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs'], cwd=ROOT, text=True).splitlines()
    names.extend(str(p.relative_to(ROOT)) for p in (ROOT / 'tests').glob('patch_7_3_0*'))
    scope = {name: digest(ROOT / name) for name in sorted(set(names)) if (ROOT / name).is_file()}
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    log = HERE / (label + '.log')
    with log.open('wb') as handle:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=handle, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'scope': scope,
               'exit': result.returncode, 'log': log.name, 'log_sha256': digest(log)}
    (HERE / (label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'label': label, 'exit': result.returncode}), flush=True)


if __name__ == '__main__':
    main()
