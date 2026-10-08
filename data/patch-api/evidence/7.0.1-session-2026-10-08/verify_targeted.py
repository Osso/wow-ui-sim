"""Run requested bounded proof sequentially; launch with run_proof.py --start."""
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
    # Read the source-defined environment names, not a patch-number naming guess.
    for path in ROOT.joinpath('tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        assert match, path
        os.environ[match[1]] = str(HERE / f'{path.stem}-results.json')
    commands = [
        ('p701-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p701-format', ['cargo', 'fmt', '--check']),
        ('p701-mists-check', ['cargo', 'check', '--no-default-features', '--features',
                              'sound,gui,casc,client-mists', '--tests']),
    ]
    results = {name: run_proof(name, command) for name, command in commands}
    print(json.dumps(results))
    return int(any(results.values()))


if __name__ == '__main__':
    sys.exit(main())
