"""Relocate audit files; accept later scope, reject preserved-input and receipt tampering."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
RELATIVE = Path('data/patch-api/evidence/7.3.2-session-2026-10-08')


def run_validator(root):
    return subprocess.run([sys.executable, '-B', str(root / RELATIVE / 'validate.py')],
                          cwd=root, capture_output=True, text=True)


def main():
    with tempfile.TemporaryDirectory(prefix='p732-relocated-') as directory:
        relocated = Path(directory)
        for name in ['data/patch-api', 'tests/data']:
            shutil.copytree(ROOT / name, relocated / name)
        (relocated / 'tools').mkdir()
        for name in ['patch_audit_validation.py', 'extract_patch_non_inventory.py']:
            shutil.copy2(ROOT / 'tools' / name, relocated / 'tools' / name)
        for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
            shutil.copy2(path, relocated / 'tests' / path.name)
        # Git objects/history are read-only; do not create or mutate another worktree.
        shutil.copy2(ROOT / '.git', relocated / '.git')
        result = run_validator(relocated)
        assert result.returncode == 0, result.stderr
        extra = relocated / 'data/patch-api/sources/0.0.0-wikitext-register.json'
        extra.write_text('{"entries": []}\n')
        result = run_validator(relocated)
        assert result.returncode == 0, result.stderr
        protected = relocated / 'data/patch-api/sources/8.1.0-page-coverage.json'
        original = protected.read_bytes()
        protected.write_bytes(original + b'\n')
        result = run_validator(relocated)
        assert result.returncode != 0 and '8.1.0-page-coverage.json' in result.stderr
        protected.write_bytes(original)
        reproduction = relocated / RELATIVE / 'p732-register-reproduction.json'
        original = reproduction.read_text()
        rows = json.loads(original)
        reproduction.write_text(json.dumps(rows[1:]))
        result = run_validator(relocated)
        assert result.returncode != 0
        reproduction.write_text(original)
        result = run_validator(relocated)
        assert result.returncode == 0, result.stderr
        print('PASS: relocated checkout, later-register scope, exact input preservation, complete reproduction set')


if __name__ == '__main__':
    main()
