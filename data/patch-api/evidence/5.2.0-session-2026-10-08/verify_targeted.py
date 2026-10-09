"""Sequential bounded verification; launched asynchronously through run_proof."""
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('proof', HERE / 'run_proof.py')
PROOF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROOF)


def verify_targeted():
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', 'HEAD', 'tests'], cwd=ROOT, text=True).splitlines()
    for name in names:
        if not re.fullmatch(r'tests/patch_.*_publication_sweep.rs', name):
            continue
        source = (ROOT / name).read_text()
        match = re.search(r'out_env: "([A-Z0-9_]+)"', source)
        if match:
            os.environ[match[1]] = str(HERE / (Path(name).stem + '-results.json'))
    commands = [
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('own-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_2_0']),
        ('raid-integration', ['cargo', 'test', '--test', 'integration', 'patch_5_2_0_raid_difficulty_backing_integration']),
        ('cooldown-integration', ['cargo', 'test', '--test', 'integration', 'cooldown_widget']),
        ('school-integration', ['cargo', 'test', '--test', 'integration', 'test_spell_get_school_string']),
        ('register-lib', ['cargo', 'test', '--lib', 'lua_api::globals::register::tests']),
        ('format', ['cargo', 'fmt', '--check']),
        ('mists', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ]
    failed = []
    for name, command in commands:
        if PROOF.run_proof(name, command):
            failed.append(name)
    print(json.dumps({'failed': failed}))
    return int(bool(failed))


if __name__ == '__main__':
    sys.exit(verify_targeted())
