"""Requested bounded proof; launch through run_proof.py --start, never poll-wait."""
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    for path in ROOT.joinpath('tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        assert match, path
        os.environ[match[1]] = str(HERE / f'{path.stem}-results.json')
    os.environ['P610_SWEEP_REGISTER'] = str(HERE / 'p610-negative-register.json')
    os.environ['P610_SWEEP_OUT'] = str(HERE / 'p610-negative-results.json')
    negative_exit = run_proof('p610-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_1_0'])
    assert negative_exit != 0, 'missing API control unexpectedly passed'
    del os.environ['P610_SWEEP_REGISTER']
    os.environ['P610_SWEEP_OUT'] = str(HERE / 'patch_6_1_0_publication_sweep-results.json')
    commands = [
        ('p610-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p610-recap-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'death_recap']),
        ('p610-recap-model', ['cargo', 'test', '--test', 'integration', 'c_death_recap_probes']),
        ('p610-legacy-absence', ['cargo', 'test', '--test', 'integration', 'p1200_removed_plain_globals']),
        ('p610-format', ['cargo', 'fmt', '--check']),
        ('p610-mists-check', ['cargo', 'check', '--no-default-features', '--features',
                              'sound,gui,casc,client-mists', '--tests']),
    ]
    outcomes = {name: run_proof(name, command) for name, command in commands}
    print(json.dumps(outcomes))
    return int(any(outcomes.values()))


if __name__ == '__main__':
    sys.exit(main())
