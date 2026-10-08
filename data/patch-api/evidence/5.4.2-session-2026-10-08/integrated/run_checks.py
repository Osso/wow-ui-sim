"""Sequential logged acceptance; launch via run_proof.py --start, never poll-wait."""
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True
MASTER = '896086537a2b3c1ead5886d5ae3e430d56e7ef20'
MASTER_ROOT = '/home/osso/.worktrees/wow-ui-sim-p542-master-proof'


def outputs(revision, directory):
    directory.mkdir(exist_ok=True)
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, 'tests'], cwd=ROOT, text=True).splitlines()
    for name in paths:
        if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', name):
            source = subprocess.check_output(['git', 'show', revision + ':' + name], cwd=ROOT, text=True)
            variable = re.search(r'out_env:\s*"([^"]+)"', source)[1]
            os.environ[variable] = str(directory / (Path(name).stem + '-results.json'))


def main():
    os.environ['PATCH_PROOF_REVISION'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    outputs(os.environ['PATCH_PROOF_REVISION'], HERE)
    jobs = [
        ('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_2_publication_sweep']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('prefork-patch_5_4_2', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_2']),
        ('integration-patch_5_4_2', ['cargo', 'test', '--test', 'integration', '--', 'patch_5_4_2']),
    ]
    for name, command in jobs:
        if run_proof(name, command):
            return 1
    os.environ['P542_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P542_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    if run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_2_publication_sweep']) not in (1, 101):
        return 1
    del os.environ['P542_SWEEP_REGISTER']
    for script in sorted((ROOT / 'tools').glob('test_*.py')):
        if run_proof(script.stem, ['python3', '-B', str(script)]):
            return 1
    if run_proof('format', ['cargo', 'fmt', '--check']):
        return 1
    if run_proof('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']):
        return 1
    os.environ['PATCH_PROOF_REVISION'] = MASTER
    os.environ['PATCH_RUN_ROOT'] = MASTER_ROOT
    outputs(MASTER, HERE / 'master')
    return run_proof('master-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])


if __name__ == '__main__':
    sys.exit(main())
