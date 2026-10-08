"""Run only the requested targeted gates, without polling or a full suite."""
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
spec = importlib.util.spec_from_file_location('proof', HERE / 'run_proof.py')
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)
for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
    match = re.search(r'out_env: "([^"]+)"', path.read_text())
    os.environ[match[1]] = str(HERE / (path.stem + '-results.json'))
commands = [
    ('p710-own-green', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_1_0']),
    ('p710-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('p710-screen', ['cargo', 'test', '--test', 'integration', 'screen_mode::']),
    ('p710-items', ['cargo', 'test', '--test', 'integration', 'c_item_api::']),
    ('p710-intrinsic', ['cargo', 'test', '--test', 'integration', 'intrinsic_types::']),
    ('p710-format', ['cargo', 'fmt', '--check']),
    ('p710-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
]
results = []
for name, command in commands:
    code = proof.run_proof(name, command)
    results.append({'name': name, 'exit': code})
os.environ['P710_SWEEP_REGISTER'] = str(HERE / 'p710-negative-register.json')
os.environ['P710_SWEEP_OUT'] = str(HERE / 'p710-negative-results.json')
code = proof.run_proof('p710-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_1_0_publication_sweep'])
results.append({'name': 'p710-negative', 'exit': code, 'expected': 1})
(HERE / 'p710-verification-results.json').write_text(json.dumps(results, indent=2) + '\n')
sys.exit(int(any(r['exit'] != r.get('expected', 0) for r in results)))
