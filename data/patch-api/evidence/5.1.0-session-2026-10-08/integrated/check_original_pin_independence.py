"""Reject every Git access to original audit commits while replaying historical proof."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def main():
    mapping = json.loads((HERE / 'rebase-mapping.json').read_text())
    pins = [row['recorded_revision'] for row in mapping['commits'] + mapping['external_pins']]
    directory = ROOT / 'target/p510-refresh/pin-denial'
    directory.mkdir(parents=True, exist_ok=True)
    wrapper = directory / 'git'
    wrapper.write_text('''#!/usr/bin/python3
import json, os, sys
pins = json.loads(os.environ['P510_DENIED_ORIGINAL_PINS'])
for argument in sys.argv[1:]:
    revision = argument.split(':', 1)[0].split('^', 1)[0]
    if len(revision) >= 7 and any(pin.startswith(revision) for pin in pins):
        sys.exit('pre-rebase commit access rejected: ' + argument)
os.execv('/usr/bin/git', ['/usr/bin/git', *sys.argv[1:]])
''')
    wrapper.chmod(0o755)
    environment = dict(os.environ, PATH=str(directory) + os.pathsep + os.environ['PATH'],
                       P510_DENIED_ORIGINAL_PINS=json.dumps(pins))
    result = subprocess.run([sys.executable, '-B', str(HERE.parent / 'validate.py')],
                            cwd=ROOT, env=environment)
    assert result.returncode == 0, 'historical validator still needs an original commit'
    print(json.dumps({'original_commits_denied': len(pins), 'status': 'PASS'}))


if __name__ == '__main__':
    main()
