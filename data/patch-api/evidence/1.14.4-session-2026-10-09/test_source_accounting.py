"""Frozen 1.14.4 literal SOURCE tests, not modeled/native compatibility."""
import copy
import json
from pathlib import Path
import unittest
import audit
EVIDENCE = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):
    def test_all_literal_rows(self):
        rows = audit.build().get('source_rows', [])
        self.assertEqual([(r['line'], r['literal']) for r in rows], [(n, s) for n, s in enumerate((EVIDENCE / 'source.wikitext').read_text().splitlines(), 1) if s.strip()])
        self.assertTrue(all(r['capabilities'] == [] for r in rows))

    def test_inventory_signature_and_prose_totals(self):
        self.assertEqual(audit.build().get('totals'), dict(physical_lines=10, nonblank_rows=8, metadata_rows=4, unproven_rows=4, contracts=11, headers=2, numerical_inventory_headers=0, api_occurrences=0, event_occurrences=0, cvar_occurrences=0, widget_method_occurrences=0, command_occurrences=0, signature_declarations=0, prose_contracts=3, unexpanded_linked_contracts=8, unexpanded_transclusions=1))

    def test_complete_wrath_inclusion_claim_not_erased(self):
        claims = audit.build().get('summary_prose', [])
        self.assertTrue(claims, 'inclusion claim absent')
        claim = claims[0]
        self.assertEqual(claim['scope'], 'all-API-changes-inclusion-claim')
        self.assertEqual(claim['linked_patches'], ['3.4.0', '3.4.1', '3.4.2'])
        self.assertEqual(claim['literal'], (EVIDENCE / 'source.wikitext').read_text().splitlines()[3])
        self.assertEqual(claim['status'], 'UNPROVEN')
        self.assertEqual(claim['identified_members'], [])

    def test_dragonflight_subset_separate_and_unexpanded(self):
        claims = audit.build().get('summary_prose', [])
        self.assertGreaterEqual(len(claims), 2)
        self.assertEqual(claims[1]['scope'], 'unspecified-subset-range')
        self.assertEqual(claims[1]['linked_patches'], ['10.0.0', '10.1.5'])
        self.assertEqual(claims[1]['identified_members'], [])
        self.assertEqual(claims[1]['literal'], (EVIDENCE / 'source.wikitext').read_text().splitlines()[4])

    def test_guidance_prose_retained(self):
        claims = audit.build().get('summary_prose', [])
        self.assertEqual(len(claims), 3)
        self.assertEqual(claims[2]['scope'], 'linked-article-guidance')
        self.assertEqual(claims[2]['literal'], '** Refer to the linked articles for guidance on any impactful changes.')

    def test_wiki_and_external_links_accounted_individually(self):
        links = [c for c in audit.build().get('contracts', []) if c['kind'].startswith('unexpanded-')]
        self.assertEqual([(c['line'], c['label'], c['target']) for c in links], [(4, 'Wrath Classic', 'World of Warcraft: Wrath of the Lich King Classic'), (4, 'Patch 3.4.0', 'Patch 3.4.0/API changes'), (4, 'Patch 3.4.1', 'Patch 3.4.1/API changes'), (4, 'Patch 3.4.2', 'Patch 3.4.2/API changes'), (5, 'Patch 10.0.0', 'Patch 10.0.0/API changes'), (5, 'Patch 10.1.5', 'Patch 10.1.5/API changes'), (10, 'wow-ui-source', 'https://github.com/Gethe/wow-ui-source/compare/1.14.3..classic_era_ptr'), (10, 'BlizzardInterfaceResources', 'https://github.com/Ketho/BlizzardInterfaceResources/compare/1.14.3..1.14.4')])
        self.assertTrue(all(c['status'] == 'UNPROVEN' and c['arguments'] is None and c['returns'] is None for c in links))

    def test_diff_heads_not_collapsed(self):
        links = [c for c in audit.build().get('contracts', []) if c['kind'] == 'unexpanded-linked-diff']
        self.assertEqual([(c['compare_base'], c['compare_head']) for c in links], [('1.14.3', 'classic_era_ptr'), ('1.14.3', '1.14.4')])

    def test_header_navigation_toc_and_context(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11404)
        self.assertEqual(ledger.get('literal_navigation'), dict(patch='1.14.4', prev='1.14.3', next='1.15.0'))
        self.assertEqual(ledger.get('literal_contexts'), ['Wrath Classic', 'Dragonflight'])
        self.assertEqual(ledger.get('client_line'), 'UNPROVEN-unexpanded-apichanges')
        self.assertEqual([(h['line'], h['label'], h['numerical_count']) for h in ledger.get('headers', [])], [(3, 'Summary', None), (8, 'Resources', None)])
        self.assertEqual(ledger.get('transclusions'), [dict(line=1, literal='{{apichanges|1.14.4|prev=1.14.3|next=1.15.0}}', expansion='UNPROVEN', role='navigation-metadata-only')])
        self.assertEqual(ledger.get('api_occurrences'), [])
        self.assertEqual(ledger.get('signatures'), [])

    def test_configuration_and_successors_not_native_credit(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([(s['patch'], s['state']) for s in ledger.get('queued_successors', [])], [('1.15.0', 'in-flight-pending-main-integration')] + [(f'1.15.{n}', 'queued-pending-main-integration' if n in [1,2] else 'integrated-canonical-input-not-applied') for n in range(1,10)])
        self.assertTrue(all(not s['applied'] and not s['native_proof'] for s in ledger['queued_successors']))

    def test_omissions_fabricated_members_history_and_credit_rejected(self):
        expected = audit.build()
        self.assertIn('contracts', expected)
        for field in ['source_rows', 'contracts', 'headers', 'summary_prose', 'transclusions', 'queued_successors']:
            for index in range(len(expected[field])):
                changed = copy.deepcopy(expected)
                changed[field].pop(index)
                with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for patch in ['3.4.0', '3.4.1', '3.4.2', '10.0.0', '10.1.5', '2.5.6', '1.60.1']:
            changed = copy.deepcopy(expected)
            changed['later_registers'] = [patch]
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for index in range(len(expected['contracts'])):
            changed = copy.deepcopy(expected)
            changed['contracts'][index]['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for field, value in [('scope', 'unspecified-subset'), ('identified_members', ['C_Fabricated.GetValue'])]:
            changed = copy.deepcopy(expected)
            changed['summary_prose'][0][field] = value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        changed = copy.deepcopy(expected)
        changed['contracts'][-2]['compare_head'] = '1.14.4'
        with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for credit in ['runtime', 'model', 'native']:
            changed = copy.deepcopy(expected)
            changed['measurements'][credit] = 1
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)

    def test_source_identity_and_returned_bytes(self):
        self.assertIn('source', audit.build())
        response = json.loads((EVIDENCE / 'source-response.json').read_bytes())
        response['query']['pages']['267043']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'): audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'): audit.validate_source(raw=(EVIDENCE / 'source.wikitext').read_bytes() + b'\n')

if __name__ == '__main__':
    unittest.main(verbosity=2)
