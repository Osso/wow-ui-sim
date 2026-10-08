"""Run bounded committed-source proof sequentially, without polling builds."""
import json
import os
from pathlib import Path
import re
import sys

from run_proof import ROOT, EVIDENCE, run_proof


def capture_publication_outputs():
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        source = path.read_text()
        name = re.search(r'out_env: "([^"]+)"', source)[1]
        os.environ[name] = str(EVIDENCE / (path.stem + '-results.json'))


def main():
    capture_publication_outputs()
    commands = [
        ('own-green', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_1']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ]
    results = {name: run_proof(name, command) for name, command in commands}
    os.environ['P541_SWEEP_REGISTER'] = str(EVIDENCE / 'negative-register.json')
    os.environ['P541_SWEEP_OUT'] = str(EVIDENCE / 'negative-results.json')
    results['negative'] = run_proof('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                                               'patch_5_4_1_publication_sweep'])
    del os.environ['P541_SWEEP_REGISTER']
    for path in sorted((ROOT / 'tools').glob('test_*.py')):
        results[path.stem] = run_proof(path.stem, ['python3', '-B', str(path)])
    results['reproduction-final'] = run_proof('reproduction-final',
        ['python3', '-B', str(EVIDENCE / 'reproduce_sources.py')])
    results['format'] = run_proof('format', ['cargo', 'fmt', '--check'])
    results['mists'] = run_proof('mists', ['cargo', 'check', '--no-default-features', '--features',
                                         'sound,gui,casc,client-mists', '--tests'])
    (EVIDENCE / 'targeted-results.json').write_text(json.dumps(results, indent=2) + '\n')
    return int(any(code != (1 if name == 'negative' else 0) for name, code in results.items()))


if __name__ == '__main__':
    sys.exit(main())
