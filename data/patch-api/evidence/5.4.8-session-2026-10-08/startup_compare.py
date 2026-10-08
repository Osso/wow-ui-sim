"""Capture addons-enabled startup JSON with isolated CVar overrides and a 90s bound."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from run_proof import run_proof, TARGET

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = 'd0fabed03cd6534b73341fbf569e09c6c388202e'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['master', 'branch'])
    args = parser.parse_args()
    name = 'p548-' + args.mode + '-startup'
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if args.mode == 'master':
        changed = subprocess.check_output(['git', 'diff', '--name-only', BASE, '--',
                                          'src', 'crates', 'Cargo.toml', 'Cargo.lock'], cwd=ROOT, text=True)
        assert not changed, 'baseline runtime differs from pinned master: ' + changed
    code = run_proof(name + '-build', ['cargo', 'build', '--bin', 'wow-sim'])
    if code:
        return code
    command = ['timeout', '90', str(Path(TARGET) / 'debug/wow-sim'), '--no-saved-vars', 'lua-errors']
    with tempfile.TemporaryDirectory(prefix='p548-cvars-') as directory:
        environment = dict(os.environ, XDG_DATA_HOME=directory)
        environment.pop('WOW_SIM_NO_ADDONS', None)
        with (HERE / (name + '.json')).open('w') as output, (HERE / (name + '.stderr.txt')).open('w') as errors:
            result = subprocess.run(command, cwd=ROOT, env=environment, stdout=output, stderr=errors)
    output = (HERE / (name + '.json')).read_text()
    normalized = json.loads(output) if output.strip() else []
    receipt = {'command': command, 'revision': revision, 'base_revision': BASE,
               'exit': result.returncode, 'addons_enabled': True, 'cvar_storage': 'isolated empty XDG_DATA_HOME',
               'output_sha256': hashlib.sha256(output.encode()).hexdigest(), 'errors': normalized}
    if args.mode == 'branch':
        baseline = json.loads((HERE / 'p548-master-startup.receipt.json').read_text())
        receipt['matches_master'] = normalized == baseline['errors']
        assert receipt['matches_master'], (normalized, baseline['errors'])
    (HERE / (name + '.receipt.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))
    return result.returncode


if __name__ == '__main__':
    sys.exit(main())
