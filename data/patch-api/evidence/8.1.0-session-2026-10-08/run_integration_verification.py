"""Stream the post-rebase checks and refresh only current 8.1.0 evidence."""
import json
import os
import re
import sys

sys.dont_write_bytecode = True
from run_proof import ROOT, EVIDENCE, run_proof


def require(name, command, expected=0):
    result = run_proof(name, command)
    if result != expected:
        raise RuntimeError(f'{name}: exit {result}, expected {expected}; see complete log')


def refresh_summary():
    rows = []
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        result_path = EVIDENCE / (path.stem + '-results.json')
        results = json.loads(result_path.read_text())
        gaps = sum(not row['ok'] for row in results.values())
        rows.append({'file': result_path.name,
                     'patch': path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.'),
                     'rows': len(results), 'ok': len(results) - gaps, 'gaps': gaps, 'result': 'pass'})
    (EVIDENCE / 'p810-sweep-summary.json').write_text(json.dumps(rows, indent=2) + '\n')


def main():
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        match = re.search(r'out_env:\s*"([^"]+)"', path.read_text())
        if match:
            os.environ[match[1]] = str(EVIDENCE / (path.stem + '-results.json'))
    require('p810-integration-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
    refresh_summary()
    run_remaining_checks()


def run_remaining_checks():
    require('p810-integration-cached-final', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                             'patch_8_1_0'])
    require('p810-integration-calendar', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                         'blizzard_calendar_loads'])
    require('p810-integration-bare-final', ['cargo', 'test', '--test', 'integration', '--',
                                     'patch_8_1_0', 'c_calendar', 'c_map_probes', 'date_and_time'])
    run_provider_checks()


def run_provider_checks():
    require('p810-integration-lib', ['cargo', 'test', '--lib', '--',
                                    'date_and_time_defaults', 'configuration_warnings_defaults'])
    os.environ['P810_SWEEP_REGISTER'] = str(EVIDENCE / 'p810-negative-register.json')
    os.environ['P810_SWEEP_OUT'] = str(EVIDENCE / 'p810-negative-observation.json')
    require('p810-integration-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                         'patch_8_1_0_publication_sweep'], expected=1)
    del os.environ['P810_SWEEP_REGISTER']
    del os.environ['P810_SWEEP_OUT']
    for kind in ('extract_patch_non_inventory', 'gen_patch_wikitext_register', 'patch_audit_validation'):
        require('p810-integration-' + kind + '-fixtures', ['python3', '-B', str(ROOT / f'tools/test_{kind}.py')])
    require('p810-integration-format', ['cargo', 'fmt', '--check'])
    require('p810-integration-mists-check', ['cargo', 'check', '--no-default-features', '--features',
                                           'sound,gui,casc,client-mists', '--tests'])
    print('PASS: requested post-rebase verification', flush=True)


if __name__ == '__main__':
    main()
