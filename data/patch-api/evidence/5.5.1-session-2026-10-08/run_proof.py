"""Launch bounded proof groups asynchronously and retain complete logs/receipts."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def run(label, command, profile='retail', extra=None, expected=0):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    target = '/home/osso/.cache/wow-ui-sim-targets/p551-page' + ('-mists' if profile == 'mists' else '')
    results = HERE / 'results'
    results.mkdir(exist_ok=True)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', CARGO_TARGET_DIR=target, CARGO_BUILD_JOBS='4')
    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        for name in re.findall(r'out_env: "([^"]+)"', path.read_text()):
            suffix = '' if name.endswith('_SWEEP_OUT') else '-' + name
            environment[name] = str(results / (path.stem + suffix + '-results.json'))
    environment.update(extra or {})
    log = HERE / (label + '.txt')
    with log.open('wb') as output:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=output, stderr=subprocess.STDOUT)
    receipt = {'command': command, 'revision': revision, 'profile': profile, 'target': target,
               'cwd': str(ROOT), 'exit': result.returncode, 'expected_exit': expected, 'log': log.name,
               'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
               'environment': {k: v for k, v in environment.items() if k.endswith('_SWEEP_OUT') or k.endswith('_CONTROL_OUT') or k == 'P551_SWEEP_REGISTER'}}
    (HERE / (label + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    if result.returncode != expected:
        raise RuntimeError(f'{label}: exit {result.returncode}, expected {expected}')


def worker(group):
    global ROOT, HERE
    if group == 'retail':
        run('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
        run('client-lines', ['cargo', 'test', '--test', 'integration', 'publication_sweep_client_lines', '--', '--nocapture'])
        worker('baseline')
    elif group == 'mists':
        args = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
        run('mists-pages', ['cargo', 'test', *args, '--test', 'integration', 'patch_5_5_', '--', '--nocapture'], 'mists')
        run('mists-check', ['cargo', 'check', *args, '--tests'], 'mists')
        run('negative', ['cargo', 'test', *args, '--test', 'integration', 'patch_5_5_1_publication_sweep', '--', '--nocapture'], 'mists', extra={'P551_SWEEP_REGISTER': str(HERE / 'negative-register.json'), 'P551_SWEEP_OUT': str(HERE / 'negative-results.json')}, expected=101)
    elif group == 'baseline':
        import tempfile
        original_root, original_here = ROOT, HERE
        with tempfile.TemporaryDirectory(prefix='p551-master-') as directory:
            checkout = Path(directory) / 'checkout'
            subprocess.check_call(['git', 'worktree', 'add', '--detach', str(checkout), '0469e5592fa74e051d9b01d85368b351f24e7d09'], cwd=original_root)
            try:
                ROOT = checkout
                HERE = original_here / 'master'
                HERE.mkdir(exist_ok=True)
                run('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
            finally:
                ROOT, HERE = original_root, original_here
                subprocess.check_call(['git', 'worktree', 'remove', '--force', str(checkout)], cwd=original_root)
    elif group == 'mists-refresh':
        args = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
        run('mists-pages-final', ['cargo', 'test', *args, '--test', 'integration', 'patch_5_5_', '--', '--nocapture'], 'mists')
        run('format-final', ['cargo', 'fmt', '--check'])
    elif group == 'gate':
        run('gate-report', ['python3', '-B', str(ROOT / 'tools/check_patch_validators.py'), 'HEAD'])
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
