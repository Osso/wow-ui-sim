"""Literal serialized SOURCE contracts, not simulator/native measurements."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
LEDGER = ROOT / 'data/patch-api/sources/2.5.4-page-coverage.json'


class SourceAccounting(unittest.TestCase):
    def test_complete_literal_inventory_and_rows(self):
        ledger = json.loads(LEDGER.read_bytes())
        self.assertEqual(len(ledger['source_rows']), 461)
        self.assertEqual(len(ledger['contracts']), 414)
        self.assertEqual(ledger['inventory']['counts'], {
            'global-api': {'added': 187, 'removed': 0},
            'widgets': {'added': 1, 'removed': 0},
            'events': {'added': 200, 'removed': 0},
            'cvars': {'added': 15, 'removed': 9},
        })

    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location('p254_validate', EVIDENCE / 'validate.py')
        cls.validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.validator)
        cls.inputs = cls.validator.read_inputs()

    def check(self, **changes):
        return self.validator.validate(**dict(self.inputs, **changes))

    def test_identity_plaintext_registry_and_configuration(self):
        result = self.check()
        self.assertEqual(result['statuses'], {'metadata-only': 48, 'UNPROVEN': 413})
        self.assertEqual(result['registry_pages'], 101)
        self.assertEqual(result['registry_boundary'], '1.0.0')
        self.assertEqual(result['matching_configured_profiles'], [])
        profiles = self.inputs['profile']['configured_profiles']
        self.assertEqual({p['variant']: p['configured_interface'] for p in profiles}, {
            'Retail': 120100, 'Ptr': 120105, 'Wrath': 38001, 'Mists': 50504,
            'Era': 11507, 'Anniversary': 11507, 'WowForever': 16001,
        })

    def test_every_nonblank_literal_row_required(self):
        for index in range(461):
            with self.subTest(index=index):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['source_rows'].pop(index)
                with self.assertRaisesRegex(AssertionError, 'ledger source_rows'):
                    self.check(ledger=ledger)

    def test_every_api_and_linked_contract_required(self):
        for index in range(414):
            with self.subTest(index=index):
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['contracts'].pop(index)
                with self.assertRaisesRegex(AssertionError, 'ledger contracts'):
                    self.check(ledger=ledger)

    def test_precise_state_signature_and_native_limits(self):
        for index, contract in enumerate(self.inputs['ledger']['contracts']):
            fields = ['status', 'native_equivalence', 'state_transitions']
            fields += ['signature', 'arguments', 'returns', 'event_payloads', 'security_rules'] if contract['kind'] != 'linked-source' else ['target', 'member_identities']
            for field in fields:
                with self.subTest(index=index, field=field):
                    ledger = copy.deepcopy(self.inputs['ledger'])
                    ledger['contracts'][index][field] = 'fabricated-proof'
                    with self.assertRaisesRegex(AssertionError, 'ledger contracts'):
                        self.check(ledger=ledger)
        self.assertEqual(self.check()['unspecified_callable_signatures'], 188)
        self.assertEqual(self.check()['linked_contracts'], 2)

    def test_header_default_scope_and_description_retained(self):
        for field, collection in [('header_count', 'header_counts'), ('symbol', 'entries')]:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger['inventory'][collection][0][field] = 'wrong'
            with self.assertRaisesRegex(AssertionError, 'ledger inventory'):
                self.check(ledger=ledger)
        for index, contract in enumerate(self.inputs['ledger']['contracts']):
            if contract['kind'] != 'cvar-publication':
                continue
            for field in ['literal_default', 'literal_scope', 'literal_description']:
                ledger = copy.deepcopy(self.inputs['ledger'])
                ledger['contracts'][index][field] = 'invented'
                with self.assertRaisesRegex(AssertionError, 'ledger contracts'):
                    self.check(ledger=ledger)

    def test_foreign_history_supersession_and_credit_rejected(self):
        for field, value in [('client_line', 'retail'), ('profile', 'anniversary'),
                             ('later_registers', ['3.4.0']), ('later_registers', ['4.4.0']),
                             ('later_registers', ['1.15.9']), ('later_registers', ['3.3.5']),
                             ('pending_successors', []), ('model_changes', 1),
                             ('runtime_observations', 1), ('native_observations', 1)]:
            ledger = copy.deepcopy(self.inputs['ledger'])
            ledger[field] = value
            with self.assertRaisesRegex(AssertionError, f'ledger {field}'):
                self.check(ledger=ledger)
        for field in ['matching_configured_profiles', 'native_profile_correspondence', 'unsupported_api_diagnosis']:
            profile = copy.deepcopy(self.inputs['profile'])
            profile[field] = 'wrong-profile-claim'
            with self.assertRaisesRegex(AssertionError, 'configured profile evidence'):
                self.check(profile=profile)

    def test_literal_identity_hash_and_plaintext_mutation(self):
        response = json.loads(self.inputs['response'])
        response['query']['pages']['515131']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            self.check(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            self.check(raw=self.inputs['raw'] + b'\n')
        with self.assertRaisesRegex(AssertionError, 'plaintext reproduction'):
            self.check(text=self.inputs['text'] + b'fabricated transclusion\n')


if __name__ == '__main__':
    unittest.main(verbosity=2)
