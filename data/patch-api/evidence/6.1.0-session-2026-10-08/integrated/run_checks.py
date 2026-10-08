"""Sequential file-backed integration proof; launch once, never poll-wait."""
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = '787b47591'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p610-page'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def environment(output, tests):
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1')
    env.pop('WOW_SIM_NO_ADDONS', None)
    env.pop('WOW_SIM_NO_SAVED_VARS', None)
    for path in tests:
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            env[match[1]] = str(output / (path.stem + '-results.json'))
    return env


def proof(label, command, env, revision):
    log = HERE / (label + '.txt')
    with log.open('wb') as output:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'exit': result.returncode,
               'log': log.name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()}
    dump(HERE / (label + '.proof.json'), receipt)
    print(label, result.returncode, flush=True)
    return result.returncode


def main():
    revision = git('rev-parse', 'HEAD')
    env = environment(HERE, sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')))
    commands = [
        ('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_1_0_publication_sweep']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('prefork-patch', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_1_0']),
        ('prefork-recap', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'death_recap']),
        ('integration-recap', ['cargo', 'test', '--test', 'integration', 'c_death_recap_probes']),
        ('integration-absence', ['cargo', 'test', '--test', 'integration', 'p1200_removed_plain_globals']),
        ('format', ['cargo', 'fmt', '--check']),
        ('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ]
    commands += [(path.stem, ['python3', '-B', str(path)]) for path in sorted((ROOT / 'tools').glob('test_*.py'))]
    results = {}
    for label, command in commands:
        results[label] = proof(label, command, env, revision)
        dump(HERE / 'command-results.json', results)
    negative = dict(env, P610_SWEEP_REGISTER=str(HERE / 'negative-register.json'),
                    P610_SWEEP_OUT=str(HERE / 'negative-results.json'))
    results['negative'] = proof('negative', commands[0][1], negative, revision)
    # Immutable master archive; runtime commands still execute from the owned worktree.
    with tempfile.TemporaryDirectory(prefix='p620-master-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        output = HERE / 'master'
        output.mkdir(exist_ok=True)
        master_env = environment(output, sorted((snapshot / 'tests').glob('patch_*_publication_sweep.rs')))
        command = ['cargo', 'test', '--manifest-path', str(snapshot / 'Cargo.toml'), '--test', 'prefork_full_ui', '--', 'publication_sweep']
        results['master-all-sweeps'] = proof('master-all-sweeps', command, master_env, git('rev-parse', MASTER))
    dump(HERE / 'command-results.json', results)
    print('FINISHED', flush=True)


if __name__ == '__main__':
    main()
