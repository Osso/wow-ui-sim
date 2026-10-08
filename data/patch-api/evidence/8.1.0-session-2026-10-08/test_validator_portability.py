"""Exercise acceptance from a relocated checkout with later shared-file edits."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[4]
RELATIVE = Path('data/patch-api/evidence/8.1.0-session-2026-10-08')


class PortabilityTest(unittest.TestCase):
    def test_relocated_checkout_and_later_mutable_inputs(self):
        with tempfile.TemporaryDirectory(prefix='p810-validator-portability-') as directory:
            destination = Path(directory)
            # Only read original git objects; no other worktree/index is changed.
            (destination / '.git').symlink_to(ROOT / '.git')
            shutil.copytree(ROOT / 'data/patch-api/sources', destination / 'data/patch-api/sources')
            shutil.copytree(ROOT / RELATIVE, destination / RELATIVE)
            shutil.copytree(ROOT / 'tools', destination / 'tools',
                            ignore=shutil.ignore_patterns('__pycache__'))
            fixture = Path('tests/data/patch_8_1_0_sweep_known_gaps.json')
            (destination / fixture).parent.mkdir(parents=True)
            shutil.copy2(ROOT / fixture, destination / fixture)
            # These paths are original-input preservation targets, legitimately
            # editable by later audits. Their current content must not be frozen.
            shared = ('api-change-pages-remaining.json', '8.2.5-page-coverage.json')
            for name in shared:
                (destination / 'data/patch-api/sources' / name).write_text(
                    json.dumps({'later_merged_audit': name}) + '\n')
            result = subprocess.run([sys.executable, str(destination / RELATIVE / 'validate.py')],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(json.loads(result.stdout)['status'], 'PASS')


if __name__ == '__main__':
    unittest.main()
