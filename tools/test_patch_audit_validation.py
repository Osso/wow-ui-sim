"""Behavioral checks for historical patch-audit validation."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from patch_audit_validation import (
    historical_json,
    historical_registers,
    historical_sweep_tests,
    preserved_input_matches,
    read_audit_json,
)

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
        bounded = sum(row['status'] == 'bounded-coverage' for row in ledger['source_rows'])
        self.assertEqual(bounded, 33)
        self.assertEqual(len(gaps), 34)
        self.assertIn('wt-global-api-C_ClubFinder.ReportPosting-117', gaps)

    def test_snapshot_registers_exclude_later_added_pages(self):
        paths = historical_registers(ROOT, '465c908ce6c88b70a94f4ee6fdb6c0928c69a42e')
        self.assertEqual(len(paths), 26)
        self.assertIn(ROOT / 'data/patch-api/sources/10.0.0-wikitext-register.json', paths)
        self.assertNotIn(ROOT / 'data/patch-api/sources/8.2.5-wikitext-register.json', paths)
        self.assertTrue(all(path.is_file() for path in paths))

    def test_sweep_scope_is_complete_at_audit_revision(self):
        paths = historical_sweep_tests(ROOT, '56a1b8e6cacc1acc115956a1fac2497600178762')
        self.assertEqual(len(paths), 34)
        self.assertIn(ROOT / 'tests/patch_12_1_0_publication_sweep.rs', paths)
        self.assertNotIn(ROOT / 'tests/patch_8_2_5_publication_sweep.rs', paths)
        self.assertTrue(all(path.is_file() for path in paths))

    def test_read_audit_json_preserves_later_closure_provenance(self):
        revision = 'b3f5906c0dc1432c8e5123b6748bed10f086d271'
        self.assertEqual(len(read_audit_json(ROOT, ROOT / GAPS, revision)), 34)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'unrelated.json'
            path.write_text(json.dumps({'current': 2}))
            self.assertEqual(read_audit_json(root, path, revision), {'current': 2})

    def test_910_validates_historical_gap_fixture(self):
        validator = ROOT / 'data/patch-api/evidence/9.1.0-session-2026-10-07/validate.py'
        result = subprocess.run(['python3', '-B', str(validator)], cwd=ROOT,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('PASS:', result.stdout)

    def test_1000_validator_does_not_rewrite_historical_result(self):
        directory = ROOT / 'data/patch-api/evidence/10.0.0-session-2026-10-07'
        result = directory / 'p1000-validation-result.json'
        before = result.read_bytes()
        subprocess.run(['python3', '-B', str(directory / 'validate.py')],
                       cwd=ROOT, check=True, capture_output=True)
        self.assertEqual(result.read_bytes(), before)

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
