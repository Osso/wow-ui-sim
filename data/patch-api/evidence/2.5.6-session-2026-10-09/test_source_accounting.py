"""Serialized SOURCE contracts only; no simulator or native measurements."""
import copy
import json
from pathlib import Path
import unittest

from validate import validate

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        source = ROOT / 'data/patch-api/sources'
        self.inputs = {
            'response': (EVIDENCE / 'source-response.json').read_bytes(),
            'raw': (source / '2.5.6-api-changes.wikitext').read_bytes(),
            'pin': json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
            'ledger': json.loads((source / '2.5.6-page-coverage.json').read_bytes()),
            'text': (source / '2.5.6-api-changes.txt').read_bytes(),
            'profile': json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
        }

    def check(self, **changes):
        return validate(**dict(self.inputs, **changes))

    def test_exact_source_accounting_derives_counts(self):
        self.assertEqual(self.check(), {
            'source_rows': 6, 'statuses': {'metadata-only': 5, 'UNPROVEN': 1},
            'enumerated_api_occurrences': 0, 'header_counts': [],
            'explicit_signatures': 0, 'local_summary_contracts': 0,
            'unexpanded_contracts': 1, 'removals': 0,
            'configured_profiles': 7, 'runtime_observations': 0,
            'native_observations': 0, 'model_observations': 0,
        })

    def test_revision_and_source_tampering_rejected(self):
        response = json.loads(self.inputs['response'])
        response['query']['pages']['685353']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            self.check(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            self.check(raw=self.inputs['raw'] + b'\n')

    def test_all_nonblank_rows_required(self):
        for index in range(6):
            with self.subTest(index=index):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['source_rows'].pop(index)
                with self.assertRaisesRegex(AssertionError, 'row accounting'):
                    self.check(ledger=ledger)

    def test_every_row_credit_and_literal_tampering_rejected(self):
        for index in range(6):
            for field, value in [('status', 'bounded-coverage'),
                                 ('capabilities', ['native-parity']),
                                 ('source_text', 'fabricated source'),
                                 ('wikitext_line', 999), ('note', '')]:
                with self.subTest(index=index, field=field):
                    ledger = copy.deepcopy(self.inputs['ledger'])
                    ledger['source_rows'][index][field] = value
                    with self.assertRaises(AssertionError):
                        self.check(ledger=ledger)

    def test_wrong_history_or_foreign_successor_rejected(self):
        for field, value in [('client_line', 'retail'), ('profile', 'anniversary'),
                             ('later_registers', ['3.3.5']),
                             ('later_registers', ['4.0.1']),
                             ('later_registers', ['3.4.3']),
                             ('later_registers', ['1.15.9']),
                             ('later_registers', ['2.5.5'])]:
            with self.subTest(field=field, value=value):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger[field] = value
                with self.assertRaisesRegex(AssertionError, 'client history'):
                    self.check(ledger=ledger)

    def test_explicit_inventory_signature_summary_or_header_credit_rejected(self):
        for field, value in [('enumerated_api_occurrences', 1), ('removals', 1),
                             ('register_created', True), ('header_counts', [1]),
                             ('explicit_signatures', 1), ('local_summary_contracts', 1)]:
            with self.subTest(field=field):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['inventory'][field] = value
                with self.assertRaisesRegex(AssertionError, 'inventory'):
                    self.check(ledger=ledger)

    def test_transclusion_contract_fields_cannot_be_invented_or_omitted(self):
        ledger = copy.deepcopy(self.inputs['ledger'])
        ledger['contracts'] = []
        with self.assertRaisesRegex(AssertionError, 'transclusion contract'):
            self.check(ledger=ledger)
        for field in self.inputs['ledger']['contracts'][0]:
            with self.subTest(field=field):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['contracts'][0][field] = 'invented'
                with self.assertRaisesRegex(AssertionError, 'transclusion contract'):
                    self.check(ledger=ledger)

    def test_configuration_is_not_native_model_or_unsupported_proof(self):
        for field, value in [('source_toc', 11509),
                             ('native_profile_correspondence', 'anniversary'),
                             ('unsupported_client_diagnosis', 'no-profile')]:
            with self.subTest(field=field):
                profile = copy.deepcopy(self.inputs['profile'])
                profile[field] = value
                with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                    self.check(profile=profile)
        for measurement in self.inputs['profile']['measurements']:
            profile = copy.deepcopy(self.inputs['profile'])
            profile['measurements'][measurement] = 'PASS'
            with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                self.check(profile=profile)
        for index in range(7):
            profile = copy.deepcopy(self.inputs['profile'])
            profile['configured_profiles'][index]['configured_interface'] = 20506
            with self.assertRaisesRegex(AssertionError, 'configured profiles'):
                self.check(profile=profile)

    def test_plaintext_transclusion_boundary_and_transform_are_exact(self):
        self.assertIn(b'[Transcluded source: Patch_1.15.9/API_changes; not expanded]',
                      self.inputs['text'])
        with self.assertRaisesRegex(AssertionError, 'plaintext reproduction'):
            self.check(text=self.inputs['text'].replace(b'not expanded', b'native proved'))
        ledger = copy.deepcopy(self.inputs['ledger'])
        ledger['non_inventory_source']['local_transform']['to'] = 'Native proof'
        with self.assertRaisesRegex(AssertionError, 'local transform'):
            self.check(ledger=ledger)


if __name__ == '__main__':
    unittest.main(verbosity=2)
