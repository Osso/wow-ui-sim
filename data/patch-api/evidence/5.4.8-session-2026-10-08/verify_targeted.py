"""Requested targeted acceptance; asynchronous launcher, no full integration suite."""
import json
import os
from pathlib import Path
import re
import sys

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    for path in ROOT.joinpath('tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        assert match, path
        os.environ[match[1]] = str(HERE / (path.stem + '-results.json'))
    os.environ['P548_SWEEP_REGISTER'] = str(HERE / 'p548-negative-register.json')
    os.environ['P548_SWEEP_OUT'] = str(HERE / 'p548-negative-results.json')
    negative = run_proof('p548-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                          'patch_5_4_8_publication_sweep'])
    assert negative != 0, 'missing API control unexpectedly passed'
    del os.environ['P548_SWEEP_REGISTER']
    os.environ['P548_SWEEP_OUT'] = str(HERE / 'patch_5_4_8_publication_sweep-results.json')
    os.environ['P548_COMBAT_OUT'] = str(HERE / 'p548-cached-combat-observations.json')
    commands = [('p548-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
                ('p548-cached-combat', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                        'patch_5_4_8_cached_combat_restrictions'])]
    for area in ['cvar', 'world_map', 'keybindings']:
        commands.append(('p548-prefork-' + area, ['cargo', 'test', '--test', 'prefork_full_ui', '--', area]))
    outcomes = {name: run_proof(name, command) for name, command in commands}
    os.environ['P548_COMBAT_OUT'] = str(HERE / 'p548-bare-combat-observations.json')
    for area in ['patch_5_4_8', 'set_cvar_global', 'cvar_bitfields',
                 'test_cvar_display_settings', 'ui_visibility_globals']:
        outcomes['integration-' + area] = run_proof('p548-integration-' + area,
                                                   ['cargo', 'test', '--test', 'integration', area])
    outcomes['startup'] = run_proof('p548-branch-startup-driver',
                                    ['python3', '-B', str(HERE / 'startup_compare.py'), 'branch'])
    outcomes['format'] = run_proof('p548-format', ['cargo', 'fmt', '--check'])
    outcomes['mists'] = run_proof('p548-mists-check', ['cargo', 'check', '--no-default-features',
                                                  '--features', 'sound,gui,casc,client-mists', '--tests'])
    print(json.dumps(outcomes))
    return int(any(outcomes.values()))


if __name__ == '__main__':
    sys.exit(main())
