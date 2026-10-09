"""Sequential worktree-local acceptance; each command has a revision-scoped receipt."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True


def outputs(directory):
    directory.mkdir(exist_ok=True)
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        variables = re.findall(r'out_env:\s*"(P\d+_SWEEP_OUT)"', path.read_text())
        assert len(variables) == 1, (path, variables)
        os.environ[variables[0]] = str(directory / (path.stem + '-results.json'))
    os.environ['MISTS_LINE_CONTROL_OUT'] = str(directory / 'mists-line-control-results.json')


def main():
    outputs(HERE)
    jobs = [
        ('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_1_0_publication_sweep']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('prefork-patch_5_1_0', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_1_0']),
        ('integration-patch_5_1_0', ['cargo', 'test', '--test', 'integration', '--', 'patch_5_1_0']),
        ('integration-pet-info', ['cargo', 'test', '--test', 'integration', 'pet_info', '--', '--test-threads=4']),
        ('integration-pet-stats', ['cargo', 'test', '--test', 'integration', 'pet_stats', '--', '--test-threads=4']),
        ('integration-namespace', ['cargo', 'test', '--test', 'integration', 'namespace_stubs_patched', '--', '--test-threads=4']),
        ('prefork-pet', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'pet']),
        ('lib-namespace', ['cargo', 'test', '--lib', 'namespace', '--', '--test-threads=4']),
    ]
    # Capture independent failures too; an unrelated host failure must not hide later gates.
    failures = []
    for name, command in jobs:
        if run_proof(name, command):
            failures.append(name)
    os.environ['P510_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P510_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    if run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_1_0_publication_sweep']) not in (1, 101):
        failures.append('negative')
    del os.environ['P510_SWEEP_REGISTER']
    outputs(HERE)
    for script in sorted((ROOT / 'tools').glob('test_*.py')):
        if run_proof(script.stem, ['python3', '-B', str(script)]):
            failures.append(script.stem)
    if run_proof('format', ['cargo', 'fmt', '--check']):
        failures.append('format')
    features = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    if run_proof('mists-check', ['cargo', 'check', *features, '--tests']):
        failures.append('mists-check')
    if run_proof('mists-all-sweeps', ['cargo', 'test', *features, '--test', 'integration', '--', 'publication_sweep']):
        failures.append('mists-all-sweeps')
    print(json.dumps({'failures': failures}))
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
