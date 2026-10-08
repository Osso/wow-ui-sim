"""One async targeted acceptance run; no full integration suite or polling."""
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def run():
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', 'HEAD', 'tests'],
                                    cwd=ROOT, text=True).splitlines()
    for name in names:
        if not re.fullmatch(r'tests/patch_.*_publication_sweep.rs', name):
            continue
        text = (ROOT / name).read_text()
        match = re.search(r'out_env:\s*"([^"]+)"', text)
        if match:
            os.environ[match[1]] = str(HERE / (Path(name).stem + '-results.json'))
    jobs = [
        ('p530-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p530-own-behavior', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_3_0_cached']),
        ('p530-bare-behavior', ['cargo', 'test', '--test', 'integration', 'patch_5_3_0_bare']),
        ('p530-pvp-lib', ['cargo', 'test', '--lib', 'test_startup_pvp_queue_surfaces_are_safe']),
        ('p530-mists', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
        ('p530-format', ['cargo', 'fmt', '--all', '--check']),
    ]
    for name, command in jobs:
        if run_proof(name, command) != 0:
            return 1
    os.environ['P530_SWEEP_REGISTER'] = str(HERE / 'p530-negative-register.json')
    os.environ['P530_SWEEP_OUT'] = str(HERE / 'p530-negative-results.json')
    status = run_proof('p530-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_3_0_publication_sweep'])
    return 0 if status in (1, 101) else 1


if __name__ == '__main__':
    sys.exit(run())
