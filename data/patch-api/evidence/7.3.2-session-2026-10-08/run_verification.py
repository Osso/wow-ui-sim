"""Run only requested publication and touched-area gates; retain each command receipt."""
import json
import os
from pathlib import Path
import re
import sys
from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
MISTS = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']


def main():
    steps = [
        ('p732-bare-final', ['cargo', 'test', '--test', 'integration', 'patch_7_3_2'], 0),
        ('p732-key-dispatch', ['cargo', 'test', '--test', 'integration', 'key_dispatch'], 0),
        ('p732-menu-bare', ['cargo', 'test', '--test', 'integration', 'blizzard_game_menu'], 0),
        ('p732-menu-cached', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_game_menu'], 0),
        ('p732-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'], 0),
        ('p732-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_3_2_publication_sweep'], 1),
        ('p732-mists-check', ['cargo', 'check', *MISTS, '--tests'], 0),
        ('p732-mists-session', ['cargo', 'test', *MISTS, '--test', 'integration', 'patch_7_3_2'], 0),
        ('p732-format', ['cargo', 'fmt', '--all', '--', '--check'], 0),
    ]
    for name, command, expected in steps:
        if name == 'p732-all-sweeps':
            for source in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
                variable = re.search(r'out_env: "([^"]+)"', source.read_text())[1]
                os.environ[variable] = str(EVIDENCE / (source.stem + '-results.json'))
        if name == 'p732-negative':
            os.environ['P732_SWEEP_REGISTER'] = str(EVIDENCE / 'p732-negative-register.json')
            os.environ['P732_SWEEP_OUT'] = str(EVIDENCE / 'p732-negative-observation.json')
        result = run_proof(name, command)
        if result != expected:
            print(json.dumps({'failed': name, 'exit': result, 'expected': expected}))
            return 1
        if name == 'p732-negative':
            del os.environ['P732_SWEEP_REGISTER']
            os.environ['P732_SWEEP_OUT'] = str(EVIDENCE / 'patch_7_3_2_publication_sweep-results.json')
    return 0


if __name__ == '__main__':
    sys.exit(main())
