"""Run the approved bounded gates asynchronously via run_proof.py --start."""
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
    ('p624-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('p624-cached-identity', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_4_cached']),
    ('p624-bare-identity', ['cargo', 'test', '--test', 'integration', 'patch_6_2_4']),
    ('p624-bnet-model', ['cargo', 'test', '--test', 'integration', 'c_battle_net_probes::']),
    ('p624-deprecated-bnet', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_deprecated_battle_net']),
    ('p624-final-format', ['cargo', 'fmt', '--check']),
    ('p624-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
]
results = []
for name, command in commands:
    code = proof.run_proof(name, command)
    results.append({'name': name, 'exit': code})
os.environ['P624_SWEEP_REGISTER'] = str(HERE / 'p624-negative-register.json')
os.environ['P624_SWEEP_OUT'] = str(HERE / 'p624-negative-results.json')
code = proof.run_proof('p624-negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_4_publication_sweep'])
results.append({'name': 'p624-negative', 'exit': code, 'expected': 1})
(HERE / 'p624-verification-results.json').write_text(json.dumps(results, indent=2) + '\n')
sys.exit(int(any(r['exit'] != r.get('expected', 0) for r in results)))
