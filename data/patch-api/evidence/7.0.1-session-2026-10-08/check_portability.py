"""Prove checkout portability, expanded future scope and source-tamper rejection."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
RELATIVE = HERE.relative_to(ROOT)


def run(command, cwd):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    return {'command': command, 'exit': result.returncode,
            'stdout': result.stdout, 'stderr': result.stderr}


def validate(checkout):
    return run([sys.executable, '-B', str(checkout / RELATIVE / 'validate.py')], checkout)


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    before = {p.relative_to(HERE).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
              for p in HERE.iterdir() if p.is_file()}
    with tempfile.TemporaryDirectory(prefix='p701-portability-') as directory:
        checkout = Path(directory) / 'checkout'
        clone = run(['git', 'clone', '--shared', '--quiet', str(ROOT), str(checkout)], ROOT)
        assert clone['exit'] == 0, clone
        initial = validate(checkout)
        assert initial['exit'] == 0, initial
        # A subsequent audit adds both input and sweep; historical proof stays fixed.
        (checkout / 'data/patch-api/sources/0.0.0-wikitext-register.json').write_text(
            json.dumps({'schema': 'patch-api-wikitext-register/v1', 'patch': '0.0.0',
                        'entries': []}) + '\n')
        (checkout / 'tests/patch_0_0_0_publication_sweep.rs').write_text(
            '// Future audit scope must not retroactively expand historical proof.\n')
        wiki = checkout / 'docs/wiki/index.md'
        original_wiki = wiki.read_text()
        wiki.write_text('Future wiki update\n' + original_wiki.replace('integration', 'integration update', 1))
        expanded = validate(checkout)
        assert expanded['exit'] == 0, expanded
        source = checkout / 'data/patch-api/sources/7.1.0-api-changes.txt'
        original_source = source.read_bytes()
        source.write_bytes(original_source + b'\n')
        tampered = validate(checkout)
        assert tampered['exit'] != 0, tampered
        assert 'AssertionError' in tampered['stderr'], tampered
        source.write_bytes(original_source)
        restored = validate(checkout)
        assert restored['exit'] == 0, restored
        # Read-only proof: identical own evidence before/after every execution.
        for name, digest in before.items():
            copied = checkout / RELATIVE / name
            assert hashlib.sha256(copied.read_bytes()).hexdigest() == digest, name
    result = {'revision': revision, 'initial': initial, 'expanded_scope_and_wiki': expanded,
              'protected_input_tampered': tampered, 'restored': restored,
              'read_only': True, 'clone': clone}
    (HERE / 'p701-validator-portability.json').write_text(json.dumps(result, indent=2) + '\n')
    print('PASS: independent clone, added audit/wiki edits, protected-source rejection, read-only evidence')


if __name__ == '__main__':
    main()
