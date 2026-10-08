"""Launch integrated proof groups; retain commands, pinned revisions and complete logs."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = Path('/home/osso/.worktrees/wow-ui-sim-p554-master-ref')


def run(label, command, profile='retail', master=False, extra=None, expected=0):
    source = MASTER if master else ROOT
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    target = '/home/osso/.cache/wow-ui-sim-targets/' + ('master-ref' if master else 'p554-page' + ('-mists' if profile == 'mists' else ''))
    directory = HERE / ('master' if master else 'results')
    directory.mkdir(exist_ok=True)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', CARGO_TARGET_DIR=target)
    for path in (source / 'tests').glob('patch_*_publication_sweep.rs'):
        for name in re.findall(r'out_env: "([^"]+)"', path.read_text()):
            suffix = '' if name.endswith('_SWEEP_OUT') else '-' + name
            environment[name] = str(directory / (path.stem + suffix + '-results.json'))
    environment.update(extra or {})
    log = HERE / (label + '.txt')
    with log.open('wb') as output:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=output, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'profile': profile, 'target': target,
               'cwd': str(ROOT), 'exit': result.returncode, 'expected_exit': expected, 'log': log.name,
               'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'environment': {k: v for k, v in environment.items() if k.endswith('_SWEEP_OUT') or k.endswith('_CONTROL_OUT') or k == 'P554_SWEEP_REGISTER'}}
    (HERE / (label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    if result.returncode != expected:
        raise RuntimeError(f'{label}: exit {result.returncode}, expected {expected}')


def worker(group):
    if group == 'retail':
        run('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
        run('client-lines', ['cargo', 'test', '--test', 'integration', 'publication_sweep_client_lines', '--', '--nocapture'])
    elif group == 'master':
        run('master-all-sweeps', ['cargo', 'test', '--manifest-path', str(MASTER / 'Cargo.toml'), '--test', 'prefork_full_ui', '--', 'publication_sweep'], master=True)
    elif group == 'mists':
        args = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
        run('mists-page', ['cargo', 'test', *args, '--test', 'integration', 'patch_5_5_4', '--', '--nocapture'], 'mists')
        run('mists-check', ['cargo', 'check', *args, '--tests'], 'mists')
        run('negative', ['cargo', 'test', *args, '--test', 'integration', 'patch_5_5_4_publication_sweep', '--', '--nocapture'], 'mists', extra={'P554_SWEEP_REGISTER': str(HERE / 'negative-register.json')}, expected=101)
    elif group == 'tools':
        run('tools-tests', ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_*.py'])
        run('format', ['cargo', 'fmt', '--check'])
    else:
        raise ValueError(group)


if __name__ == '__main__':
    group = sys.argv[1]
    if '--worker' in sys.argv:
        worker(group)
    else:
        with (HERE / (group + '.launcher.txt')).open('wb') as output:
            process = subprocess.Popen([sys.executable, '-B', __file__, group, '--worker'], cwd=ROOT, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        print(json.dumps({'group': group, 'pid': process.pid}))
