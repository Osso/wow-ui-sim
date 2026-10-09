"""Replay committed history in a relocated clone without pre-rebase objects."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def git(root, *arguments):
    return subprocess.check_output(['git', *arguments], cwd=root, text=True)


def main():
    revision = git(ROOT, 'rev-parse', 'HEAD').strip()
    mapping = json.loads((HERE / 'rebase-mapping.json').read_text())
    pins = [row['recorded_revision'] for row in mapping['commits'] + mapping['external_commits']]
    with tempfile.TemporaryDirectory(prefix='p541-history-clone-') as temporary:
        clone = Path(temporary) / 'checkout'
        subprocess.run(['git', 'clone', '--no-local', '--single-branch', '--branch', 'p541-page',
                        str(ROOT), str(clone)], cwd=ROOT, check=True)
        assert git(clone, 'rev-parse', 'HEAD').strip() == revision
        missing = []
        for pin in pins:
            result = subprocess.run(['git', 'cat-file', '-e', pin + '^{commit}'],
                                    cwd=clone, capture_output=True, text=True)
            assert result.returncode != 0, pin
            missing.append({'recorded_revision': pin, 'exit': result.returncode,
                            'stderr': result.stderr})
        validator = clone / HERE.parent.relative_to(ROOT) / 'validate.py'
        result = subprocess.run([sys.executable, '-B', str(validator)], cwd=clone,
                                capture_output=True, text=True)
        print(result.stdout + result.stderr, end='')
        (HERE / 'history-absent-object-control.json').write_text(json.dumps({
            'clone_revision': revision, 'missing_original_objects': missing,
            'historical_validator_exit': result.returncode}, indent=2) + '\n')
        assert result.returncode == 0
    return 0


if __name__ == '__main__':
    sys.exit(main())
