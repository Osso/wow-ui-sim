"""Bounded integrated proof; retain failures without hiding independent gates."""
import json
import os
import re
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof


def outputs(directory):
    directory.mkdir(exist_ok=True)
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        variables = re.findall(r'out_env:\s*"(P\d+_SWEEP_OUT)"', path.read_text())
        assert len(variables) == 1, (path, variables)
        os.environ[variables[0]] = str(directory / (path.stem + '-results.json'))
    os.environ['MISTS_LINE_CONTROL_OUT'] = str(directory / 'mists-line-control-results.json')


def main():
    outputs(HERE)
    failures = []
    jobs = [
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('own-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_0_4']),
        ('own-integration', ['cargo', 'test', '--test', 'integration', 'patch_5_0_4', '--', '--test-threads=2']),
        ('pet-battles-integration', ['cargo', 'test', '--test', 'integration', 'pet_battles', '--', '--test-threads=2']),
        ('pet-battle-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'pet_battle']),
        ('retail-line-controls', ['cargo', 'test', '--test', 'integration', 'publication_sweep_client_lines', '--', '--test-threads=2']),
        ('lib-namespace', ['cargo', 'test', '--lib', 'namespace', '--', '--test-threads=2']),
        ('default-check', ['cargo', 'check', '--tests']),
        ('format', ['cargo', 'fmt', '--check']),
        ('startup-build', ['cargo', 'build', '--bin', 'wow-sim']),
        ('startup', ['timeout', '90', '/home/osso/.cache/wow-ui-sim-targets/p504-page/debug/wow-sim', '--no-addons', '--no-saved-vars', 'lua-errors']),
    ]
    for name, command in jobs:
        if run_proof(name, command):
            failures.append(name)
    os.environ['P504_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P504_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    if run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_0_4_publication_sweep']) not in (1, 101):
        failures.append('negative')
    del os.environ['P504_SWEEP_REGISTER']
    outputs(HERE)
    for script in sorted((ROOT / 'tools').glob('test_*.py')):
        if run_proof(script.stem, ['python3', '-B', str(script)]):
            failures.append(script.stem)
    os.environ['PATCH_TARGET'] = '/home/osso/.cache/wow-ui-sim-targets/p504-page-mists'
    features = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    for name, command in [
        ('mists-check', ['cargo', 'check', *features, '--tests']),
        ('mists-all-sweeps', ['cargo', 'test', *features, '--test', 'integration', 'publication_sweep', '--', '--test-threads=2']),
        ('mists-pet-type', ['cargo', 'test', *features, '--test', 'integration', 'patch_5_0_4', '--', '--test-threads=2']),
    ]:
        if run_proof(name, command):
            failures.append(name)
    print(json.dumps({'failures': failures}))
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
