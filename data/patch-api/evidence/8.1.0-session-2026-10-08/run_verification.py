"""Ordered targeted gates, streamed receipts; launch through run_proof --start."""
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from run_proof import ROOT, EVIDENCE, run_proof


def require(name, command, expected=0):
    result = run_proof(name, command)
    if result != expected:
        raise RuntimeError(f'{name}: exit {result}, expected {expected}; see complete log')


def main():
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        text = path.read_text()
        match = re.search(r'out_env:\s*"([^"]+)"', text)
        if match:
            os.environ[match[1]] = str(EVIDENCE / (path.stem + '-results.json'))
    require('p810-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
    for selected in ('c_calendar_defaults_probes', 'c_map_probes'):
        require('p810-regression-' + selected,
                ['cargo', 'test', '--test', 'integration', selected])
    for selected in ('date_and_time_defaults', 'configuration_warnings_defaults'):
        require('p810-regression-' + selected, ['cargo', 'test', '--lib', selected])
    for selected in ('blizzard_calendar_loads', 'blizzard_world_map_loads'):
        require('p810-regression-' + selected,
                ['cargo', 'test', '--test', 'prefork_full_ui', '--', selected])
    os.environ['P810_SWEEP_REGISTER'] = str(EVIDENCE / 'p810-negative-register.json')
    os.environ['P810_SWEEP_OUT'] = str(EVIDENCE / 'p810-negative-observation.json')
    require('p810-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                              'patch_8_1_0_publication_sweep'], expected=1)
    del os.environ['P810_SWEEP_REGISTER']
    del os.environ['P810_SWEEP_OUT']
    for kind in ('extract', 'gen'):
        script = ('tools/test_extract_patch_non_inventory.py' if kind == 'extract'
                  else 'tools/test_gen_patch_wikitext_register.py')
        require('p810-' + kind + '-fixtures', ['python3', script])
    require('p810-source-reproduction-final', ['python3', str(EVIDENCE / 'reproduce_sources.py')])
    require('p810-format', ['cargo', 'fmt', '--check'])
    features = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    require('p810-mists-check', ['cargo', 'check', *features, '--tests'])
    require('p810-mists-behavior', ['cargo', 'test', *features, '--test', 'integration', 'patch_8_1_0'])
    print('PASS: ordered targeted verification', flush=True)


if __name__ == '__main__':
    main()
