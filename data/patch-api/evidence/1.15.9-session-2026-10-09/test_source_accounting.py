"""Own literal SOURCE fixtures; never invokes simulator or native client."""
import copy
import json
from pathlib import Path
import unittest
import audit
EVIDENCE = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):

    def test_full_literal_rows_and_contract_counts(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('totals'), {'nonblank_rows': 37, 'contracts': 14, 'explicit_api_reference_occurrences': 4, 'distinct_api_names': 2, 'cvar_argument_occurrences': 3, 'chat_command_occurrences': 3, 'event_occurrences': 0, 'widget_method_occurrences': 0, 'enumerated_inventory_entries': 0, 'numerical_inventory_headers': 0, 'explicit_signature_declarations': 0, 'partial_call_examples': 3, 'partial_return_removal_notes': 1, 'prose_contracts': 8, 'unexpanded_linked_contracts': 2, 'runtime_observations': 0, 'model_observations': 0, 'native_observations': 0})

    def test_every_literal_row_occurrence_retained(self):
        rows = audit.build().get('source_rows', [])
        raw = (EVIDENCE / 'source.wikitext').read_text()
        self.assertEqual([(r['line'], r['literal']) for r in rows], [(n, s) for n, s in enumerate(raw.splitlines(), 1) if s.strip()])
        self.assertTrue(all((r['capabilities'] == [] for r in rows)))

    def test_scope_not_number_or_shared_code_equivalence(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('client_line'), 'classic-era')
        self.assertEqual(ledger.get('source_toc'), 11509)
        self.assertEqual(ledger.get('literal_clients'), ['Classic Era', 'Season of Discovery', 'Hardcore'])
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual(ledger.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})

    def test_partial_calls_are_tbc_prose_not_era_defaults(self):
        calls = [c for c in audit.build().get('contracts', []) if c['kind'] == 'partial-chat-call']
        self.assertEqual([(c['cvar'], c['value']) for c in calls], [('nameplateShowClassColor', 1), ('nameplateShowFriendlyClassColor', 1), ('raidFramesDispelIndicatorOverlay', 0)])
        self.assertTrue(all((c['context'] == 'quoted-tbc-2.5.6' and c['default'] is None and (c['status'] == 'UNPROVEN') for c in calls)))

    def test_removal_does_not_guess_return_position(self):
        contracts = audit.build().get('contracts', [])
        notes = [c for c in contracts if c['kind'] == 'partial-return-removal']
        self.assertEqual(len(notes), 1)
        self.assertEqual(notes[0]['symbol'], 'UnitAura')
        self.assertEqual(notes[0]['removed_return'], 'shouldConsolidate')
        self.assertIsNone(notes[0]['remaining_returns'])
        self.assertIsNone(notes[0]['removed_position'])

    def test_configured_profiles_are_not_native_measurements(self):
        configured = audit.build().get('configured_profiles', [])
        self.assertEqual([(p['feature'], p['configured_interface']) for p in configured], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertTrue(all((p['native_correspondence'] == 'UNPROVEN' for p in configured)))

    def test_serialized_contract_omission_credit_and_foreign_history_rejected(self):
        expected = audit.build()
        self.assertIn('contracts', expected)
        for field in ['source_rows', 'contracts', 'headers', 'api_occurrences']:
            for index in range(len(expected[field])):
                changed = copy.deepcopy(expected)
                changed[field].pop(index)
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed)
        for field, value in [('client_line', 'retail'), ('later_registers', ['2.5.6']), ('later_registers', ['3.4.3']), ('later_registers', ['1.60.1']), ('measurements', {'runtime': 1, 'model': 0, 'native': 0})]:
            changed = copy.deepcopy(expected)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for index in range(len(expected['contracts'])):
            changed = copy.deepcopy(expected)
            changed['contracts'][index]['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_response_revision_content_and_raw_hash_controls(self):
        self.assertIn('source', audit.build())
        response = json.loads((EVIDENCE / 'source-response.json').read_bytes())
        response['query']['pages']['685352']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(EVIDENCE / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
