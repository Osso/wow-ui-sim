"""Behavioral checks for historical patch-audit validation."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from patch_audit_validation import historical_json, historical_registers, preserved_input_matches

ROOT = Path(__file__).resolve().parents[1]
LEDGER = 'data/patch-api/sources/9.2.5-page-coverage.json'
GAPS = 'tests/data/patch_9_2_5_sweep_known_gaps.json'
PRIOR = '7ff3dc540^'
LATER = '127aa3724035d9e668143d9b28aaa4cf6e176fa1'


def git_bytes(root, *args):
    return subprocess.check_output(['git', *args], cwd=root)


class PatchAuditValidationTests(unittest.TestCase):
    def test_only_exact_recorded_later_bytes_replace_frozen_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in (LEDGER, GAPS):
                old = git_bytes(ROOT, 'show', f'{PRIOR}:{path}')
                current = git_bytes(ROOT, 'show', f'{LATER}:{path}')
                digest = hashlib.sha256(old).hexdigest()
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                for accepted in (old, current):
                    target.write_bytes(accepted)
                    self.assertTrue(preserved_input_matches(root, path, digest))
                target.write_bytes(current + b'\n')
                self.assertFalse(preserved_input_matches(root, path, digest))
                target.write_bytes(current)
                self.assertFalse(preserved_input_matches(root, path, '0' * 64))
                other = root / 'unrelated.json'
                other.write_bytes(current)
                self.assertFalse(preserved_input_matches(root, 'unrelated.json', digest))

    def test_historical_accounting_retains_old_counts_and_gap(self):
        revision = 'b3f5906c0dc1432c8e5123b6748bed10f086d271'
        ledger = historical_json(ROOT, LEDGER, revision)
        gaps = historical_json(ROOT, GAPS, revision)
        self.assertEqual(sum(row['status'] == 'bounded-coverage' for row in ledger['source_rows']), 33)
        self.assertEqual(len(gaps), 34)
        self.assertIn('wt-global-api-C_ClubFinder.ReportPosting-117', gaps)

    def test_snapshot_registers_exclude_later_added_pages(self):
        paths = historical_registers(ROOT, '465c908ce6c88b70a94f4ee6fdb6c0928c69a42e')
        self.assertEqual(len(paths), 26)
        self.assertIn(ROOT / 'data/patch-api/sources/10.0.0-wikitext-register.json', paths)
        self.assertNotIn(ROOT / 'data/patch-api/sources/8.2.5-wikitext-register.json', paths)
        self.assertTrue(all(path.is_file() for path in paths))

    def test_unrecorded_json_change_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git_bytes(root, 'init', '-q')
            path = root / 'protected.json'
            path.write_text(json.dumps({'protected': 1}))
            git_bytes(root, 'add', 'protected.json')
            git_bytes(root, '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                      'commit', '-qm', 'Original input')
            revision = git_bytes(root, 'rev-parse', 'HEAD').decode().strip()
            self.assertEqual(historical_json(root, 'protected.json', revision), {'protected': 1})
            path.write_text(json.dumps({'protected': 2}))
            with self.assertRaises(AssertionError):
                historical_json(root, 'protected.json', revision)


if __name__ == '__main__':
    unittest.main()
