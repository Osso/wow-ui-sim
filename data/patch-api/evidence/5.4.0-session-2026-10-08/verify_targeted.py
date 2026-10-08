"""Ordered targeted proofs; start via run_proof.py --start, never poll-wait."""
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from run_proof import EVIDENCE, ROOT, run_proof


def write_sweep_outputs():
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        source = path.read_text()
        variable = re.search(r'out_env: "([^"]+)"', source)[1]
        os.environ[variable] = str(EVIDENCE / (path.stem + '-results.json'))


def main():
    write_sweep_outputs()
    checks = [
        ('p540-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p540-prefork-behavior', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_0_behavior']),
        ('p540-integration-behavior', ['cargo', 'test', '--test', 'integration', 'patch_5_4_0_behavior']),
        ('p540-integration-instance', ['cargo', 'test', '--test', 'integration', 'instance_info::']),
        ('p540-integration-forbidden', ['cargo', 'test', '--test', 'integration', 'protected_frame_enforcement::']),
        ('p540-format', ['cargo', 'fmt', '--check']),
        ('p540-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ]
    results = {name: run_proof(name, command) for name, command in checks}
    (EVIDENCE / 'p540-targeted-summary.json').write_text(json.dumps(results, indent=2) + '\n')
    return int(any(results.values()))


if __name__ == '__main__':
    sys.exit(main())
