"""SOURCE-only serialized contracts; no runtime/native compatibility credit."""
import copy
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        ledger_path = ROOT / 'data/patch-api/sources/2.5.1-page-coverage.json'
        self.assertTrue(ledger_path.exists(), 'missing 2.5.1 literal source ledger')
        from validate import validate
        self.validate = validate
        self.inputs = {
            'response': (EVIDENCE / 'source-response.json').read_bytes(),
            'raw': (ROOT / 'data/patch-api/sources/2.5.1-api-changes.wikitext').read_bytes(),
            'pin': json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
            'ledger': json.loads(ledger_path.read_bytes()),
            'text': (ROOT / 'data/patch-api/sources/2.5.1-api-changes.txt').read_bytes(),
            'profile': json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
        }

    def check(self, **changes):
        return self.validate(**dict(self.inputs, **changes))

    def test_exact_literal_accounting(self):
        summary = self.check()
        self.assertEqual(summary['source_rows'], 627)
        self.assertEqual(summary['statuses'], {'metadata-only': 51, 'UNPROVEN': 576})
        self.assertEqual(summary['enumerated_api_occurrences'], 573)
        self.assertEqual(summary['contracts'], 576)
        self.assertEqual(summary['explicit_signatures'], 0)
        self.assertEqual(summary['transclusions'], 0)
        self.assertEqual(summary['configured_profiles'], 7)
        self.assertEqual(summary['runtime_observations'], 0)
        self.assertEqual(summary['native_observations'], 0)
        self.assertTrue(all(h['header_count'] == h['parsed_count']
                            for h in summary['header_counts']))

    def test_response_identity_and_source_hash(self):
        response = json.loads(self.inputs['response'])
        response['query']['pages']['71215']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'identity'):
            self.check(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            self.check(raw=self.inputs['raw'] + b'\n')

    def test_every_nonblank_literal_row_required(self):
        for index in range(627):
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger['source_rows'].pop(index)
            with self.subTest(index=index), self.assertRaisesRegex(AssertionError, 'literal rows'):
                self.check(ledger=ledger)

    def test_literal_status_credit_and_notes_are_exact(self):
        # One metadata, linked summary, global, widget script, event and command row.
        for number in [1, 5, 17, 400, 425, 576]:
            index = next(i for i, row in enumerate(self.inputs['ledger']['source_rows'])
                         if row['wikitext_line'] == number)
            for field, value in [('status', 'bounded-coverage'), ('capabilities', ['native-parity']),
                                 ('source_text', 'invented'), ('note', ''), ('wikitext_line', 999)]:
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['source_rows'][index][field] = value
                with self.subTest(number=number, field=field), self.assertRaisesRegex(AssertionError, 'literal rows'):
                    self.check(ledger=ledger)

    def test_inventory_and_all_headers_reconciled(self):
        for field, value in [('entries', []), ('headers', []), ('explicit_signatures', 1),
                             ('transclusions', 1), ('source_toc', 11507)]:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger['inventory'][field] = value
            with self.subTest(field=field), self.assertRaisesRegex(AssertionError, 'inventory'):
                self.check(ledger=ledger)

    def test_precise_contract_fields_cannot_be_invented(self):
        for index in [0, 3, 342, 575]:
            for field, value in [('status', 'PASS'), ('arguments', []), ('returns', []),
                                 ('state_transitions', 'supported'), ('security_rules', 'secure'),
                                 ('native_equivalence', 'anniversary'), ('event_payloads', [])]:
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['contracts'][index][field] = value
                with self.subTest(index=index, field=field), self.assertRaisesRegex(AssertionError, 'contracts'):
                    self.check(ledger=ledger)
        ledger = copy.deepcopy(self.inputs['ledger'])
        ledger['contracts'].pop()
        with self.assertRaisesRegex(AssertionError, 'contracts'):
            self.check(ledger=ledger)

    def test_history_and_pending_successors_are_not_supersession(self):
        for field, value in [('client_line', 'retail'), ('profile', 'anniversary'),
                             ('later_registers', ['2.5.2']), ('later_registers', ['3.4.3']),
                             ('later_registers', ['4.0.1']), ('later_registers', ['1.15.9']),
                             ('pending_successors', [])]:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger[field] = value
            with self.subTest(field=field, value=value), self.assertRaisesRegex(AssertionError, 'client history'):
                self.check(ledger=ledger)

    def test_configuration_cannot_prove_native_or_unsupported_api(self):
        for field, value in [('source_toc', 11507), ('native_profile_correspondence', 'anniversary'),
                             ('unsupported_client_diagnosis', 'no-profile')]:
            profile = copy.deepcopy(self.inputs['profile'])
            profile[field] = value
            with self.subTest(field=field), self.assertRaisesRegex(AssertionError, 'profile evidence'):
                self.check(profile=profile)
        profile = copy.deepcopy(self.inputs['profile'])
        profile['configured_profiles'][5]['configured_interface'] = 20501
        with self.assertRaisesRegex(AssertionError, 'configured profiles'):
            self.check(profile=profile)
        for measurement in self.inputs['profile']['measurements']:
            profile = copy.deepcopy(self.inputs['profile'])
            profile['measurements'][measurement] = 'PASS'
            with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                self.check(profile=profile)

    def test_plaintext_is_historical_and_linked_content_unexpanded(self):
        with self.assertRaisesRegex(AssertionError, 'plaintext'):
            self.check(text=self.inputs['text'] + b'Native proof\n')


if __name__ == '__main__':
    unittest.main(verbosity=2)
