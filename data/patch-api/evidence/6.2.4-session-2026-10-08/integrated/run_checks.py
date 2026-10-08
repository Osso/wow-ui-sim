"""Stream bounded integration checks to receipts; --start never waits for Cargo."""
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORICAL = HERE.parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p624-page'
MASTER = '846a30663'


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def run_proof(label, command, environment=None, source_revision=None):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    log = HERE / (label + '.txt')
    started = time.monotonic()
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1')
    env.update(environment or {})
    with log.open('w') as output:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    dump(HERE / (label + '.proof.json'), {
        'command': command, 'revision': revision, 'source_revision': source_revision or revision,
        'environment': environment or {}, 'exit': result.returncode, 'log': log.name,
        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
        'seconds': time.monotonic() - started,
    })
    print(label, result.returncode, flush=True)
    return result.returncode


def output_environment(directory, tests):
    return {re.search(r'out_env: "([A-Z0-9_]+)"', path.read_text())[1]:
            str(directory / (path.stem + '-results.json')) for path in tests}


def main():
    if '--start' in sys.argv:
        with (HERE / 'launcher.txt').open('w') as output:
            process = subprocess.Popen([sys.executable, '-B', str(Path(__file__).resolve())],
                                       cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                                       start_new_session=True)
        print(json.dumps({'pid': process.pid, 'log': str(HERE / 'launcher.txt')}))
        return
    results = {}
    tests = sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs'))
    environment = output_environment(HERE, tests)
    commands = [
        ('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_4_publication_sweep']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('prefork-patch-6-2-4', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_4']),
        ('integration-patch-6-2-4', ['cargo', 'test', '--test', 'integration', 'patch_6_2_4']),
        ('bnet-model', ['cargo', 'test', '--test', 'integration', 'c_battle_net_probes::']),
        ('deprecated-bnet', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_deprecated_battle_net']),
    ]
    for label, command in commands:
        results[label] = run_proof(label, command, environment)
        if results[label]:
            dump(HERE / 'command-results.json', results)
            return
    negative_env = dict(environment, P624_SWEEP_REGISTER=str(HISTORICAL / 'p624-negative-register.json'),
                        P624_SWEEP_OUT=str(HERE / 'negative-results.json'))
    results['negative'] = run_proof('negative', commands[0][1], negative_env)
    extension = HERE / 'extension'
    extension.mkdir(exist_ok=True)
    shutil.copyfile(HISTORICAL / 'p624-register-reproduction.json', extension / 'p624-register-reproduction.json')
    rows = json.loads((HISTORICAL / 'p624-sweep-summary.json').read_text())
    dump(extension / 'p624-sweep-summary.json', [
        {'patch': row['test'].removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.'),
         **{key: row[key] for key in ('rows', 'ok', 'gaps')}, 'result': 'pass'} for row in rows])
    results['extend-receipts'] = run_proof('extend-receipts', [
        'python3', '-B', str(ROOT / 'tools/extend_patch_audit_receipts.py'), str(ROOT), str(extension),
        'p624', 'Merged 7.0.1/7.0.3 flags from pinned provenance', '6.2.4'])
    # Exact master sources, without changing this worktree or using a second target.
    master_output = HERE / 'master'
    master_output.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='p624-master-') as temporary:
        snapshot = Path(temporary)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        master_tests = sorted((snapshot / 'tests').glob('patch_*_publication_sweep.rs'))
        results['master-all-sweeps'] = run_proof('master-all-sweeps', [
            'cargo', 'test', '--manifest-path', str(snapshot / 'Cargo.toml'), '--test', 'prefork_full_ui',
            '--', 'publication_sweep'], output_environment(master_output, master_tests),
            subprocess.check_output(['git', 'rev-parse', MASTER], cwd=ROOT, text=True).strip())
    for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        label = tool + '-fixtures'
        results[label] = run_proof(label, ['python3', '-B', str(ROOT / 'tools' / ('test_' + tool + '.py'))])
    results['format'] = run_proof('format', ['cargo', 'fmt', '--check'])
    results['mists-check'] = run_proof('mists-check', [
        'cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests'])
    dump(HERE / 'command-results.json', results)
    print('FINISHED', flush=True)


if __name__ == '__main__':
    main()
