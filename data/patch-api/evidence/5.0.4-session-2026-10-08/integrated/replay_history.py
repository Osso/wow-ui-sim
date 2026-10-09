"""Replay the unchanged historical validator against its exact tree/blob inputs.

Prior validators are historical inputs, not requirements on later live audits.
Only their reads are redirected to the blobs already sealed by finalization.json.
All original invariant functions execute unchanged.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import types

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def main():
    preservation = json.loads((HERE / 'historical-preservation.json').read_text())
    source = (HERE / 'historical-validator.py.txt').read_bytes()
    assert hashlib.sha256(source).hexdigest() == preservation['validate.py']
    pins = json.loads((HISTORY / 'finalization.json').read_text())
    original_read = Path.read_bytes
    original_subprocess = sys.modules['subprocess']
    native_check_output = subprocess.check_output
    prior = {ROOT / path: blob for path, blob in pins['prior_validators'].items()}
    denied = set(json.loads((HERE / 'rebase-mapping.json').read_text())['original_commits'])

    def pinned_read(path):
        if path in prior:
            return native_check_output(['git', 'show', prior[path]], cwd=ROOT)
        return original_read(path)

    def pinned_output(command, *args, **kwargs):
        if command[0] == 'git':
            for argument in command[1:]:
                revision = argument.split(':', 1)[0].split('^', 1)[0]
                assert not (len(revision) >= 7 and any(pin.startswith(revision) for pin in denied)), ('original commit access', argument)
        return native_check_output(command, *args, **kwargs)

    proxy = types.ModuleType('subprocess')
    proxy.__dict__.update(subprocess.__dict__)
    proxy.check_output = pinned_output
    namespace = {'__name__': 'historical_504_validator', '__file__': str(HISTORY / 'validate.py')}
    sys.modules['subprocess'] = proxy
    Path.read_bytes = pinned_read
    try:
        exec(compile(source, 'historical-validator.py.txt', 'exec'), namespace)
        namespace['validate']()
    finally:
        Path.read_bytes = original_read
        sys.modules['subprocess'] = original_subprocess
    print(json.dumps({'status': 'PASS', 'original_commits_denied': len(denied), 'prior_validator_blobs': len(prior)}))


if __name__ == '__main__':
    main()
