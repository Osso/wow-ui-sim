"""Sequential logged checks; launch once, inspect receipts without poll-waiting."""
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = 'd0fabed03'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p602-page'


def environment(output, tests):
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1')
    env.pop('WOW_SIM_NO_ADDONS', None)
    env.pop('WOW_SIM_NO_SAVED_VARS', None)
    for path in tests:
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            env[match[1]] = str(output / (path.stem + '-results.json'))
    return env


if __name__ == '__main__':
    inherited = environment(HERE, sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')))
    os.environ.clear()
    os.environ.update(inherited)
    commands = [('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])]
    for target in ('integration', 'prefork_full_ui'):
        for selector in ('patch_6_0_2', 'c_scenario_info_probes', 'scenario', 'objective_tracker',
                         'blizzard_shared_map_data_providers_loads', 'blizzard_flight_map_loads',
                         'blizzard_quest_navigation_loads', 'blizzard_poi_button_loads'):
            if selector == 'c_scenario_info_probes' and target != 'integration':
                continue
            command = ['cargo', 'test', '--test', target, '--', selector]
            if target == 'integration' and selector == 'scenario':
                command.extend(['--skip', 'c_scenario_info_probes'])
            commands.append((target + '-' + selector, command))
    commands.extend([
        ('retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
        ('branch-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
        ('format', ['cargo', 'fmt', '--check']),
        ('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ])
    commands.extend((p.stem, ['python3', '-B', str(p)]) for p in sorted((ROOT / 'tools').glob('test_*.py')))
    results = {}
    for label, command in commands:
        results[label] = run_proof(label, command)
        (HERE / 'command-results.json').write_text(json.dumps(results, indent=2) + '\n')
    os.environ['P602_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P602_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    results['negative'] = run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_0_2_publication_sweep'])
    os.environ.pop('P602_SWEEP_REGISTER')
    # Snapshot commands execute from the owned worktree and only read the pinned archive.
    with tempfile.TemporaryDirectory(prefix='p602-master-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        output = HERE / 'master'
        output.mkdir(exist_ok=True)
        master_env = environment(output, sorted((snapshot / 'tests').glob('patch_*_publication_sweep.rs')))
        os.environ.clear()
        os.environ.update(master_env)
        os.environ.pop('P602_SWEEP_OUT', None)
        os.environ['PATCH_PROOF_REVISION'] = MASTER
        results['master-all-sweeps'] = run_proof('master-all-sweeps', ['cargo', 'test', '--manifest-path', str(snapshot / 'Cargo.toml'), '--test', 'prefork_full_ui', '--', 'publication_sweep'])
        results['master-retail-build'] = run_proof('master-retail-build', ['cargo', 'build', '--manifest-path', str(snapshot / 'Cargo.toml'), '--bin', 'wow-sim'])
        results['master-startup'] = run_proof('master-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])
    (HERE / 'command-results.json').write_text(json.dumps(results, indent=2) + '\n')
    print('FINISHED', flush=True)
