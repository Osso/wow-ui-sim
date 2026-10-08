"""Run only publication sweeps and bounded backing regressions, then Mists/format."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def source_names():
    return subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', 'HEAD', 'tests'],
                                   cwd=ROOT, text=True).splitlines()


def run():
    sweep_sources = [name for name in source_names() if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', name)]
    for name in sweep_sources:
        text = (ROOT / name).read_text()
        variable = re.search(r'out_env:\s*"([^"]+)"', text)[1]
        os.environ[variable] = str(HERE / (Path(name).stem + '-results.json'))
    jobs = [
        ('p547-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p547-own-behavior', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_7']),
        ('p547-cached-bnet', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_deprecated_battle_net']),
        ('p547-specialization', ['cargo', 'test', '--test', 'integration', 'patch_11_1_0_specialization_names_use_catalog_identity']),
        ('p547-bnet-namespace', ['cargo', 'test', '--test', 'integration', 'p1200_rest_battle_net_outbound']),
        ('p547-mists', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
        ('p547-format', ['cargo', 'fmt', '--all', '--check']),
    ]
    for name, command in jobs:
        if run_proof(name, command) != 0:
            return 1
    os.environ['P547_SWEEP_REGISTER'] = str(HERE / 'p547-negative-register.json')
    os.environ['P547_SWEEP_OUT'] = str(HERE / 'p547-negative-results.json')
    exit_code = run_proof('p547-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_7_publication_sweep'])
    return 0 if exit_code in (1, 101) else 1


if __name__ == '__main__':
    sys.exit(run())
