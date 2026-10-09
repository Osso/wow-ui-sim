"""External serialized source/proof contracts; no simulator behavior claims."""
import copy
import json
from pathlib import Path
import unittest

from validate import validate

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        self.response = (EVIDENCE / 'source-response.json').read_bytes()
        self.raw = (ROOT / 'data/patch-api/sources/3.4.3-api-changes.wikitext').read_bytes()
        self.pin = json.loads((EVIDENCE / 'source-pin.json').read_bytes())
        self.ledger = json.loads((ROOT / 'data/patch-api/sources/3.4.3-page-coverage.json').read_bytes())
        self.text = (ROOT / 'data/patch-api/sources/3.4.3-api-changes.txt').read_bytes()
        self.profile = json.loads((EVIDENCE / 'profile-observation.json').read_bytes())

    def check(self, **changes):
        inputs = dict(response=self.response, raw=self.raw, pin=self.pin,
                      ledger=self.ledger, text=self.text, profile=self.profile)
        inputs.update(changes)
        return validate(**inputs)

    def test_exact_source_accounting_derives_counts(self):
        self.assertEqual(self.check(), {
            'source_rows': 7, 'statuses': {'metadata-only': 5, 'UNPROVEN': 2},
            'enumerated_api_occurrences': 0, 'header_counts': [],
            'removals': 0, 'runtime_observations': 0,
        })

    def test_revision_and_source_tampering_rejected(self):
        response = json.loads(self.response)
        response['query']['pages']['152751']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            self.check(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            self.check(raw=self.raw + b'\n')

    def test_all_nonblank_rows_required(self):
        for index in range(len(self.ledger['source_rows'])):
            with self.subTest(index=index):
                ledger = copy.deepcopy(self.ledger)
                ledger['source_rows'].pop(index)
                with self.assertRaisesRegex(AssertionError, 'row accounting'):
                    self.check(ledger=ledger)

    def test_prose_publication_or_behavior_credit_rejected(self):
        for index, row in enumerate(self.ledger['source_rows']):
            if row['status'] != 'UNPROVEN':
                continue
            for field, value in [('status', 'bounded-coverage'), ('capabilities', ['publication'])]:
                with self.subTest(index=index, field=field):
                    ledger = copy.deepcopy(self.ledger)
                    ledger['source_rows'][index][field] = value
                    with self.assertRaises(AssertionError):
                        self.check(ledger=ledger)

    def test_wrong_client_line_and_foreign_successor_rejected(self):
        for field, value in [('client_line', 'retail'), ('profile', 'mists'),
                             ('later_registers', ['10.1.7']), ('later_registers', ['4.4.0'])]:
            with self.subTest(field=field, value=value):
                ledger = copy.deepcopy(self.ledger)
                ledger[field] = value
                with self.assertRaisesRegex(AssertionError, 'client history'):
                    self.check(ledger=ledger)

    def test_injected_inventory_rejected(self):
        ledger = copy.deepcopy(self.ledger)
        ledger['inventory']['enumerated_api_occurrences'] = 1
        with self.assertRaisesRegex(AssertionError, 'inventory'):
            self.check(ledger=ledger)

    def test_profile_support_not_native_credit(self):
        for field, value in [('profile', 'retail'), ('native_observations', 1),
                             ('runtime_observations', 1), ('supported_profile', False)]:
            with self.subTest(field=field):
                profile = copy.deepcopy(self.profile)
                profile[field] = value
                with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                    self.check(profile=profile)


if __name__ == '__main__':
    unittest.main(verbosity=2)
