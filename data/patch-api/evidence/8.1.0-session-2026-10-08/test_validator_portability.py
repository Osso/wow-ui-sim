"""Exercise historical scope and protected-input checks from a relocated checkout."""
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
    def test_relocated_checkout_and_new_register_outside_historical_scope(self):
        with tempfile.TemporaryDirectory(prefix='p810-validator-portability-') as directory:
            destination = Path(directory)
            (destination / '.git').symlink_to(ROOT / '.git')
            for relative in ('data/patch-api/sources', str(RELATIVE), 'tools', 'tests'):
                shutil.copytree(ROOT / relative, destination / relative,
                                ignore=shutil.ignore_patterns('__pycache__'))
            # A later page must not expand the complete pinned register/sweep set.
            (destination / 'data/patch-api/sources/1.0.0-wikitext-register.json').write_text('{}\n')
            validator = destination / RELATIVE / 'validate.py'
            command = [sys.executable, '-B', str(validator)]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(json.loads(result.stdout)['status'], 'PASS')
            # An unrelated mutation of a preserved ledger is never a later-audit allowance.
            protected = destination / 'data/patch-api/sources/8.2.5-page-coverage.json'
            original = protected.read_bytes()
            protected.write_bytes(original + b'\n')
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('8.2.5-page-coverage.json', result.stderr)
            protected.write_bytes(original)
            # Receipts must account for every historical register, not an arbitrary subset.
            reproduction = destination / RELATIVE / 'p810-register-reproduction.json'
            rows = json.loads(reproduction.read_text())
            reproduction.write_text(json.dumps(rows[1:]) + '\n')
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)


if __name__ == '__main__':
    unittest.main()
