"""Pinned-master comparisons in the same worktree and target, after branch jobs finish.

Main thread copies this and run_proof.py under target/p510-refresh/master-runner,
with explicit worktree ROOT and target-only EVIDENCE, then detaches this worktree
at the pinned master. This script never switches branches or touches sibling trees.
"""
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

MASTER = '7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a'


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
        ('master-integration-pet-info', ['cargo', 'test', '--test', 'integration', 'pet_info', '--', '--test-threads=4']),
        ('master-integration-pet-stats', ['cargo', 'test', '--test', 'integration', 'pet_stats', '--', '--test-threads=4']),
        ('master-integration-namespace', ['cargo', 'test', '--test', 'integration', 'namespace_stubs_patched', '--', '--test-threads=4']),
        ('master-prefork-pet', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'pet']),
        ('master-lib-namespace', ['cargo', 'test', '--lib', 'namespace', '--', '--test-threads=4']),
        ('master-mists-all-sweeps', ['cargo', 'test', '--no-default-features', '--features',
                                  'sound,gui,casc,client-mists', '--test', 'integration', '--', 'publication_sweep']),
    ]
    failures = []
    for name, command in jobs:
        if run_proof(name, command):
            failures.append(name)
    print({'failures': failures})
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
