"""Bounded serialized SOURCE contracts; no runtime or native measurements."""
import copy
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        source = ROOT / 'data/patch-api/sources'
        ledger_path = source / '2.5.3-page-coverage.json'
        self.assertTrue(ledger_path.exists(), 'missing Patch 2.5.3 source ledger')
        from validate import validate
        self.validate = validate
        self.inputs = {
            'raw': (source / '2.5.3-api-changes.wikitext').read_bytes(),
            'response': (EVIDENCE / 'source-response.json').read_bytes(),
            'pin': json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
            'ledger': json.loads(ledger_path.read_bytes()),
            'text': (source / '2.5.3-api-changes.txt').read_bytes(),
            'profile': json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
            'register': json.loads((source / '2.5.3-wikitext-register.json').read_bytes()),
        }

    def check(self, **changes):
        return self.validate(**dict(self.inputs, **changes))

    def test_exact_counts_and_native_limits(self):
        result = self.check()
        self.assertEqual(result['source_rows'], 103)
        self.assertEqual(result['inventory_occurrences'], 49)
        self.assertEqual(result['headers'], 8)
        self.assertEqual(result['summary_contracts'], 1)
        self.assertEqual(result['explicit_signatures'], 0)
        self.assertEqual(result['unspecified_callable_signatures'], 25)
        self.assertEqual(result['cvar_defaults'], 15)
        self.assertEqual(result['transclusions'], 0)
        self.assertEqual(result['native_observations'], 0)

    def test_every_row_required(self):
        for index in range(103):
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger['source_rows'].pop(index)
            with self.subTest(index=index), self.assertRaises(AssertionError):
                self.check(ledger=ledger)

    def test_every_literal_row_and_credit_rejected(self):
        for index in range(103):
            for key, value in [('status', 'bounded-coverage'), ('capabilities', ['native']),
                               ('source_text', 'invented'), ('wikitext_line', 999),
                               ('note', '')]:
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['source_rows'][index][key] = value
                with self.subTest(index=index, key=key), self.assertRaises(AssertionError):
                    self.check(ledger=ledger)

    def test_inventory_headers_defaults_and_signatures_required(self):
        for field in ['entries', 'header_counts']:
            for index in range(len(self.inputs['register'][field])):
                register = copy.deepcopy(self.inputs['register'])
                register[field].pop(index)
                with self.subTest(field=field, index=index), self.assertRaises(AssertionError):
                    self.check(register=register)
        for field in ['inventory', 'contracts', 'linked_contracts']:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger[field] = []
            with self.subTest(field=field), self.assertRaises(AssertionError):
                self.check(ledger=ledger)
        for index, contract in enumerate(self.inputs['ledger']['contracts']):
            for field in contract:
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['contracts'][index][field] = 'invented'
                with self.subTest(index=index, field=field), self.assertRaises(AssertionError):
                    self.check(ledger=ledger)

    def test_foreign_successor_and_wrong_profile_rejected(self):
        for field, value in [('client_line', 'retail'), ('profile', 'anniversary'),
                             ('later_registers', ['3.3.5']), ('later_registers', ['4.0.1']),
                             ('later_registers', ['3.4.3']), ('later_registers', ['1.15.9']),
                             ('later_registers', [])]:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(AssertionError):
                self.check(ledger=ledger)

    def test_identity_and_plaintext_tampering_rejected(self):
        response = json.loads(self.inputs['response'])
        response['query']['pages']['53568']['revisions'][0]['revid'] += 1
        for changes in [{'response': json.dumps(response).encode()},
                        {'raw': self.inputs['raw'] + b'\n'},
                        {'text': self.inputs['text'] + b'Native proof\n'}]:
            with self.subTest(changes=list(changes)), self.assertRaises(AssertionError):
                self.check(**changes)

    def test_config_is_not_native_or_unsupported_proof(self):
        for key, value in [('source_toc', 11507),
                           ('native_profile_correspondence', 'anniversary'),
                           ('unsupported_client_diagnosis', 'no-profile')]:
            profile = copy.deepcopy(self.inputs['profile'])
            profile[key] = value
            with self.subTest(key=key), self.assertRaises(AssertionError):
                self.check(profile=profile)
        for index in range(7):
            profile = copy.deepcopy(self.inputs['profile'])
            profile['configured_profiles'][index]['configured_interface'] = 20503
            with self.subTest(index=index), self.assertRaises(AssertionError):
                self.check(profile=profile)
        for key in self.inputs['profile']['measurements']:
            profile = copy.deepcopy(self.inputs['profile'])
            profile['measurements'][key] = 'PASS'
            with self.subTest(key=key), self.assertRaises(AssertionError):
                self.check(profile=profile)


if __name__ == '__main__':
    unittest.main(verbosity=2)
