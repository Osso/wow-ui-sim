"""Only historical SOURCE accounting; no simulator/native claims."""
from pathlib import Path
import unittest
import copy
import json
from validate import account_source, inputs, validate

EVIDENCE = Path(__file__).resolve().parent


class SourceAccounting(unittest.TestCase):
    def test_full_inventory_and_raw_rows(self):
        raw = (EVIDENCE / '2.5.2-wikitext.txt').read_text()
        result = account_source(raw)
        self.assertEqual(result.get('api_occurrences'), 145)
        self.assertEqual(result.get('source_rows'), sum(bool(s.strip()) for s in raw.splitlines()))
        self.assertEqual(result.get('removals'), 33)
        self.assertEqual(result.get('explicit_signatures'), 0)
        self.assertEqual(result.get('local_summary_contracts'), 1)
        self.assertEqual(result.get('transclusions'), 0)
        self.assertEqual(result.get('header_pairs'), [[80, 1], [2, 0], [18, 29], [12, 3]])

    def test_frozen_identity_and_text(self):
        original = inputs()
        result = validate(**original)
        self.assertEqual(result['unspecified_callable_signatures'], 83)
        self.assertEqual(result['pending_successors'], 4)
        self.assertEqual(result['native_observations'], 0)
        response = json.loads(original['response'])
        response['query']['pages']['69347']['revisions'][0]['revid'] += 1
        for change in [{'response': json.dumps(response).encode()},
                       {'raw': original['raw'] + b'\n'},
                       {'text': original['text'] + b'fabricated parity'}]:
            with self.subTest(change=next(iter(change))):
                with self.assertRaises(AssertionError):
                    validate(**dict(original, **change))

    def test_every_nonblank_row_is_required_and_literal(self):
        original = inputs()
        for index in range(len(original['ledger']['source_rows'])):
            for field, value in [(None, None), ('status', 'bounded-coverage'),
                                 ('source_text', 'fabricated'), ('note', ''),
                                 ('capabilities', ['native-parity']), ('wikitext_line', -1)]:
                ledger = copy.deepcopy(original['ledger'])
                if field is None:
                    ledger['source_rows'].pop(index)
                else:
                    ledger['source_rows'][index][field] = value
                with self.subTest(index=index, field=field):
                    with self.assertRaisesRegex(AssertionError, 'row accounting'):
                        validate(**dict(original, ledger=ledger))

    def test_every_api_and_numerical_header_required(self):
        original = inputs()
        for key in ['inventory_entries', 'inventory_headers']:
            for index in range(len(original['ledger'][key])):
                ledger = copy.deepcopy(original['ledger'])
                ledger[key].pop(index)
                with self.subTest(key=key, index=index):
                    with self.assertRaises(AssertionError):
                        validate(**dict(original, ledger=ledger))
        for key in original['ledger']['accounting']:
            ledger = copy.deepcopy(original['ledger'])
            ledger['accounting'][key] = 'fabricated'
            with self.subTest(key=key):
                with self.assertRaisesRegex(AssertionError, 'derived accounting'):
                    validate(**dict(original, ledger=ledger))

    def test_precise_unproven_contracts_cannot_be_omitted_or_invented(self):
        original = inputs()
        for index, contract in enumerate(original['ledger']['contracts']):
            for key in [None] + list(contract):
                ledger = copy.deepcopy(original['ledger'])
                if key is None:
                    ledger['contracts'].pop(index)
                else:
                    ledger['contracts'][index][key] = 'fabricated'
                with self.subTest(index=index, key=key):
                    with self.assertRaisesRegex(AssertionError, 'source contracts'):
                        validate(**dict(original, ledger=ledger))

    def test_no_foreign_successor_or_wrong_profile_credit(self):
        original = inputs()
        for key, value in [('client_line', 'retail'), ('profile', 'anniversary'),
                           ('later_registers', ['3.3.5']), ('later_registers', ['4.0.1']),
                           ('later_registers', ['3.4.3']), ('later_registers', ['1.14.0']),
                           ('pending_successors', []), ('signatures', ['fabricated()']),
                           ('measurements', {'native': 'PASS'})]:
            ledger = copy.deepcopy(original['ledger'])
            ledger[key] = value
            with self.subTest(key=key, value=value):
                with self.assertRaises(AssertionError):
                    validate(**dict(original, ledger=ledger))

    def test_configuration_is_not_native_or_unsupported_api_diagnosis(self):
        original = inputs()
        for key, value in [('source_toc', 11507), ('native_profile_correspondence', 'anniversary'),
                           ('unsupported_client_diagnosis', 'all APIs unsupported')]:
            profile = copy.deepcopy(original['profile'])
            profile[key] = value
            with self.subTest(key=key):
                with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                    validate(**dict(original, profile=profile))
        for measurement in original['profile']['measurements']:
            profile = copy.deepcopy(original['profile'])
            profile['measurements'][measurement] = 'PASS'
            with self.assertRaisesRegex(AssertionError, 'profile evidence'):
                validate(**dict(original, profile=profile))
        for index in range(len(original['profile']['configured_profiles'])):
            profile = copy.deepcopy(original['profile'])
            profile['configured_profiles'][index]['configured_interface'] = 20502
            with self.assertRaisesRegex(AssertionError, 'configured profiles'):
                validate(**dict(original, profile=profile))


if __name__ == '__main__':
    unittest.main(verbosity=2)
