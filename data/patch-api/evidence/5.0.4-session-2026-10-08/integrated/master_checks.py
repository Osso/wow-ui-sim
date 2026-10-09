"""Pinned-master comparisons in the same worktree and target, after branch jobs finish.

Main thread copies this and run_proof.py under target/p504-refresh/master-runner,
with explicit worktree ROOT and target-only EVIDENCE, then detaches this worktree
at the pinned master. This script never switches branches or touches sibling trees.
"""
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

MASTER = '1c9984d2e8a5e672c5109d2234197303e6190317'


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    assert revision == MASTER, ('unexpected master comparison revision', revision)
    os.environ['PATCH_PROOF_REVISION'] = revision
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        variables = re.findall(r'out_env:\s*"(P\d+_SWEEP_OUT)"', path.read_text())
        assert len(variables) == 1, (path, variables)
        os.environ[variables[0]] = str(HERE / (path.stem + '-results.json'))
    os.environ['MISTS_LINE_CONTROL_OUT'] = str(HERE / 'mists-line-control-results.json')
    jobs = [
        ('master-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('master-pet-battles-integration', ['cargo', 'test', '--test', 'integration', 'pet_battles', '--', '--test-threads=2']),
        ('master-pet-battle-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'pet_battle']),
        ('master-retail-line-controls', ['cargo', 'test', '--test', 'integration', 'publication_sweep_client_lines', '--', '--test-threads=2']),
        ('master-lib-namespace', ['cargo', 'test', '--lib', 'namespace', '--', '--test-threads=2']),
    ]
    failures = []
    for name, command in jobs:
        if run_proof(name, command):
            failures.append(name)
    os.environ['PATCH_TARGET'] = '/home/osso/.cache/wow-ui-sim-targets/p504-page-mists'
    features = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    if run_proof('master-mists-all-sweeps', ['cargo', 'test', *features, '--test', 'integration', 'publication_sweep', '--', '--test-threads=2']):
        failures.append('master-mists-all-sweeps')
    print({'failures': failures})
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
