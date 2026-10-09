"""Own frozen SOURCE fixtures; no simulator, Git or native client required."""
import copy
import json
from pathlib import Path
import unittest
import audit
EVIDENCE = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):

    def test_exact_literal_rows(self):
        rows = audit.build().get('source_rows', [])
        self.assertEqual([(r['line'], r['literal']) for r in rows], [(n, s) for n, s in enumerate((EVIDENCE / 'source.wikitext').read_text().splitlines(), 1) if s.strip()])
        self.assertTrue(all((r['capabilities'] == [] for r in rows)))

    def test_full_scope_counts(self):
        self.assertEqual(audit.build().get('totals'), {'physical_lines': 5, 'nonblank_rows': 4, 'metadata_rows': 3, 'unproven_rows': 1, 'contracts': 2, 'headers': 1, 'numerical_inventory_headers': 0, 'api_occurrences': 0, 'event_occurrences': 0, 'cvar_occurrences': 0, 'widget_method_occurrences': 0, 'command_occurrences': 0, 'signature_declarations': 0, 'prose_contracts': 0, 'unexpanded_linked_contracts': 2, 'unexpanded_transclusions': 1})

    def test_literal_toc_does_not_invent_client_name(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11506)
        self.assertEqual(ledger.get('client_line'), 'UNPROVEN-unexpanded-apichanges')
        self.assertEqual(ledger.get('literal_clients'), [])
        self.assertEqual(ledger.get('literal_navigation'), {'patch': '1.15.6', 'prev': '1.15.5', 'next': '1.15.7'})

    def test_link_occurrences_do_not_expand_into_api_contracts(self):
        contracts = audit.build().get('contracts', [])
        self.assertEqual([(c['label'], c['target']) for c in contracts], [('wow-ui-source', 'https://github.com/Gethe/wow-ui-source/compare/1.15.5..1.15.6'), ('BlizzardInterfaceResources', 'https://github.com/Ketho/BlizzardInterfaceResources/compare/1.15.5..1.15.6')])
        self.assertTrue(all((c['status'] == 'UNPROVEN' and c['arguments'] is None and (c['returns'] is None) for c in contracts)))

    def test_headers_and_unexpanded_navigation_preserved(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('headers'), [{'line': 3, 'literal': '==Resources==', 'label': 'Resources', 'numerical_count': None, 'status': 'metadata-only'}])
        self.assertEqual(ledger.get('transclusions'), [{'line': 1, 'literal': '{{apichanges|1.15.6|prev=1.15.5|next=1.15.7}}', 'expansion': 'UNPROVEN', 'role': 'navigation-metadata-only'}])

    def test_configured_and_queued_not_native_or_supersession(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(ledger.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([q['patch'] for q in ledger.get('queued_successors', [])], ['1.15.7', '1.15.8', '1.15.9'])
        self.assertTrue(all((q['applied'] is False and q['native_proof'] is False for q in ledger['queued_successors'])))
        self.assertEqual(ledger['queued_successors'][0]['state'], 'inflight-pending-main-integration')

    def test_omissions_fabricated_credit_and_foreign_history_rejected(self):
        expected = audit.build()
        self.assertIn('contracts', expected)
        for field in ['source_rows', 'contracts', 'headers', 'transclusions', 'queued_successors']:
            for index in range(len(expected[field])):
                changed = copy.deepcopy(expected)
                changed[field].pop(index)
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed)
        for field, value in [('client_line', 'classic-era'), ('client_line', 'retail'), ('later_registers', ['2.5.6']), ('later_registers', ['2.4.2']), ('later_registers', ['3.4.3']), ('later_registers', ['1.60.1']), ('measurements', {'runtime': 1, 'model': 0, 'native': 0})]:
            changed = copy.deepcopy(expected)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for index in range(len(expected['contracts'])):
            changed = copy.deepcopy(expected)
            changed['contracts'][index]['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_frozen_identity_and_content_tampering(self):
        self.assertIn('source', audit.build())
        response = json.loads((EVIDENCE / 'source-response.json').read_bytes())
        response['query']['pages']['619994']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(EVIDENCE / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
