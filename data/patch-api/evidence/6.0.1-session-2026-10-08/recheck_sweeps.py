"""Recheck the corrected expected-gap fixture; retain initial failure separately."""
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
commands = [
    ('discovery', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_0_1']),
    ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
]
for label, command in commands:
    result = subprocess.run([sys.executable, '-B', str(HERE / 'run_proof.py'), 'p601-' + label, *command], cwd=ROOT)
    if result.returncode:
        sys.exit(result.returncode)
