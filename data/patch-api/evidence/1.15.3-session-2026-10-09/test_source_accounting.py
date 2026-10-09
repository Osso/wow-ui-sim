"""Own frozen SOURCE fixtures; no runtime or native compatibility credit."""
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
        self.assertEqual(audit.build().get('totals'), {'physical_lines': 8, 'nonblank_rows': 6, 'metadata_rows': 4, 'unproven_rows': 2, 'contracts': 5, 'headers': 2, 'numerical_inventory_headers': 0, 'api_occurrences': 0, 'event_occurrences': 0, 'cvar_occurrences': 0, 'widget_method_occurrences': 0, 'command_occurrences': 0, 'signature_declarations': 0, 'prose_contracts': 1, 'unexpanded_linked_contracts': 4, 'unexpanded_transclusions': 1})

    def test_literal_toc_navigation_and_client_context(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11503)
        self.assertEqual(ledger.get('literal_clients'), ['Dragonflight', 'Cataclysm'])
        self.assertEqual(ledger.get('client_line'), 'UNPROVEN-unexpanded-apichanges')
        self.assertEqual(ledger.get('literal_navigation'), {'patch': '1.15.3', 'prev': '1.15.2', 'next': '1.15.4'})

    def test_subset_summary_and_internal_links_not_backfilled(self):
        ledger = audit.build()
        self.assertEqual([c['kind'] for c in ledger.get('contracts', [])], ['summary-prose', 'unexpanded-wiki-link', 'unexpanded-wiki-link', 'unexpanded-linked-diff', 'unexpanded-linked-diff'])
        self.assertEqual(ledger.get('summary_prose'), [{'line': 4, 'literal': (EVIDENCE / 'source.wikitext').read_text().splitlines()[3], 'status': 'UNPROVEN', 'limit': 'Subset unspecified; no linked API identities or behavioral contracts imported.'}])
        self.assertEqual([(c['label'], c['target']) for c in ledger['contracts'][1:3]], [('Patch 10.2.7', 'Patch 10.2.7/API changes'), ('Patch 4.4.0', 'Patch 4.4.0/API changes')])
        self.assertTrue(all((c['status'] == 'UNPROVEN' and c['arguments'] is None and (c['returns'] is None) for c in ledger['contracts'])))

    def test_comparison_bases_remain_distinct(self):
        diffs = audit.build().get('contracts', [])[-2:]
        self.assertEqual([(c['label'], c['target'], c['compare_base'], c['compare_head']) for c in diffs], [('wow-ui-source', 'https://github.com/Gethe/wow-ui-source/compare/4.4.0..1.15.3', '4.4.0', '1.15.3'), ('BlizzardInterfaceResources', 'https://github.com/Ketho/BlizzardInterfaceResources/compare/1.15.2..1.15.3', '1.15.2', '1.15.3')])

    def test_headers_and_unexpanded_template(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('headers'), [{'line': 3, 'literal': '==Summary==', 'label': 'Summary', 'numerical_count': None, 'status': 'metadata-only'}, {'line': 6, 'literal': '==Resources==', 'label': 'Resources', 'numerical_count': None, 'status': 'metadata-only'}])
        self.assertEqual(ledger.get('transclusions'), [{'line': 1, 'literal': '{{apichanges|1.15.3|prev=1.15.2|next=1.15.4}}', 'expansion': 'UNPROVEN', 'role': 'navigation-metadata-only'}])
        self.assertEqual(ledger.get('api_occurrences'), [])
        self.assertEqual(ledger.get('signatures'), [])

    def test_configured_and_successors_not_native_or_supersession(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(ledger.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([(q['patch'], q['state']) for q in ledger.get('queued_successors', [])], [('1.15.4', 'in-flight-pending-main-integration'), ('1.15.5', 'queued-pending-main-integration'), ('1.15.6', 'queued-pending-main-integration'), ('1.15.7', 'integrated-canonical-input-not-applied'), ('1.15.8', 'integrated-canonical-input-not-applied'), ('1.15.9', 'integrated-canonical-input-not-applied')])
        self.assertTrue(all((q['applied'] is False and q['native_proof'] is False for q in ledger['queued_successors'])))

    def test_omissions_and_fabricated_credit_rejected(self):
        expected = audit.build()
        self.assertIn('contracts', expected)
        for field in ['source_rows', 'contracts', 'headers', 'summary_prose', 'transclusions', 'queued_successors']:
            for index in range(len(expected[field])):
                changed = copy.deepcopy(expected)
                changed[field].pop(index)
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed)
        for field, value in [('client_line', 'classic-era'), ('client_line', 'retail'), ('later_registers', ['10.2.7']), ('later_registers', ['4.4.0']), ('later_registers', ['2.5.6']), ('later_registers', ['1.60.1']), ('measurements', {'runtime': 1, 'model': 0, 'native': 0})]:
            changed = copy.deepcopy(expected)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for index in range(len(expected['contracts'])):
            changed = copy.deepcopy(expected)
            changed['contracts'][index]['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_collapsed_compare_base_rejected(self):
        changed = audit.build()
        self.assertIn('contracts', changed)
        changed['contracts'][-2]['compare_base'] = '1.15.2'
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed)

    def test_frozen_identity_and_content_tampering(self):
        self.assertIn('source', audit.build())
        response = json.loads((EVIDENCE / 'source-response.json').read_bytes())
        response['query']['pages']['593663']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(EVIDENCE / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
