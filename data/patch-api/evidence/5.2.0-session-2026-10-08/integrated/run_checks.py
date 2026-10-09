"""Sequential logged acceptance; launch via run_proof.py --start, never poll-wait."""
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True
MASTER = '5e4e82ef6'
MASTER_ROOT = '/home/osso/.worktrees/wow-ui-sim-p520-master-proof'


def outputs(revision, directory):
    directory.mkdir(exist_ok=True)
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, 'tests'], cwd=ROOT, text=True).splitlines()
    for name in paths:
        if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', name):
            source = subprocess.check_output(['git', 'show', revision + ':' + name], cwd=ROOT, text=True)
            variables = re.findall(r'out_env:\s*"(P\d+_SWEEP_OUT)"', source)
            assert len(variables) == 1, (name, variables)
            os.environ[variables[0]] = str(directory / (Path(name).stem + '-results.json'))
    os.environ['MISTS_LINE_CONTROL_OUT'] = str(directory / 'mists-line-control-results.json')


def main():
    os.environ['PATCH_PROOF_REVISION'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    outputs(os.environ['PATCH_PROOF_REVISION'], HERE)
    jobs = [
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('prefork-patch_5_2_0', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_2_0']),
        ('integration-patch_5_2_0', ['cargo', 'test', '--test', 'integration', '--', 'patch_5_2_0']),
    ]
    for name, command in jobs:
        if run_proof(name, command):
            return 1
    os.environ['P520_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P520_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    if run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_2_0_publication_sweep']) not in (1, 101):
        return 1
    del os.environ['P520_SWEEP_REGISTER']
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
    if run_proof('master-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']):
        return 1
    command = ['cargo', 'test', '--no-default-features', '--features',
               'sound,gui,casc,client-mists', '--test', 'integration', '--', 'publication_sweep']
    for label, revision, checkout, directory in [
        ('mists-all-sweeps', subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), str(ROOT), HERE),
        ('master-mists-all-sweeps', MASTER, MASTER_ROOT, HERE / 'master'),
    ]:
        os.environ['PATCH_PROOF_REVISION'] = revision
        os.environ['PATCH_RUN_ROOT'] = checkout
        outputs(revision, directory)
        if run_proof(label, command):
            return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
