#!/usr/bin/env python3
"""Focused behavioral fixtures for the bounded 4.3.0 accounting contract."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
HERE = ROOT / 'data/patch-api/evidence/4.3.0-session-2026-10-09'
SPEC = importlib.util.spec_from_file_location('p430_validation', HERE / 'validate.py')
VALIDATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATE)


def read_json(path):
    return json.loads(path.read_text())


class AccountingTests(unittest.TestCase):
    def setUp(self):
        sources = ROOT / 'data/patch-api/sources'
        self.register = read_json(sources / '4.3.0-wikitext-register.json')
        self.ledger = read_json(sources / '4.3.0-page-coverage.json')
        self.results = read_json(HERE / 'reviewed-results.json')
        self.known = read_json(ROOT / 'tests/data/patch_4_3_0_sweep_known_gaps.json')

    def validate(self):
        return VALIDATE.validate_accounting(
            self.register, self.ledger, self.results, self.known)

    def test_pinned_source_reproduces_register_and_retained_context(self):
        VALIDATE.validate_sources()

    def test_sealed_log_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=HERE) as directory:
            root = Path(directory)
            (root / 'proof.log').write_text('original proof\n')
            receipt = {'log': 'proof.log',
                       'log_sha256': VALIDATE.digest(root / 'proof.log')}
            (root / 'own.proof.json').write_text(json.dumps(receipt))
            with patch.object(VALIDATE, 'HERE', root):
                VALIDATE.validate_receipts()
                (root / 'proof.log').write_text('altered proof\n')
                with self.assertRaisesRegex(AssertionError, 'log seal'):
                    VALIDATE.validate_receipts()

    def test_real_observations_derive_counts(self):
        summary = self.validate()
        self.assertEqual(len(self.register['entries']), summary['inventory'])
        self.assertEqual(len(self.known), summary['gaps'])
        self.assertEqual(sum(row['ok'] for row in self.results.values()),
                         summary['publication_or_absence'])

    def test_new_runtime_gap_is_rejected(self):
        passing = next(key for key, row in self.results.items() if row['ok'])
        self.results[passing]['ok'] = False
        with self.assertRaisesRegex(AssertionError, 'gap IDs'):
            self.validate()

    def test_missing_occurrence_is_rejected(self):
        self.ledger['source_rows'].pop(0)
        with self.assertRaisesRegex(AssertionError, 'ledger IDs'):
            self.validate()

    def test_false_native_credit_is_rejected(self):
        row = next(row for row in self.ledger['source_rows']
                   if row['status'] == 'bounded-coverage')
        row['capabilities'].append('native-parity')
        with self.assertRaisesRegex(AssertionError, 'publication-only credit'):
            self.validate()

    def test_duplicate_result_occurrence_is_not_collapsed(self):
        entry = copy.deepcopy(self.register['entries'][0])
        self.register['entries'].append(entry)
        with self.assertRaisesRegex(AssertionError, 'duplicate register IDs'):
            self.validate()

    def test_reviewed_gap_resolution_recomputes_counts(self):
        resolved = self.known.pop()
        self.results[resolved]['ok'] = True
        row = next(row for row in self.ledger['source_rows']
                   if row['source_id'] == resolved)
        row['status'] = 'bounded-coverage'
        row['capabilities'] = ['current-retail-publication-or-absence']
        summary = self.validate()
        self.assertEqual(len(self.known), summary['gaps'])


if __name__ == '__main__':
    unittest.main()
