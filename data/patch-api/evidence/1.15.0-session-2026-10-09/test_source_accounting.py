"""Own 1.15.0 literal SOURCE fixtures; not native/model tests."""
import copy
import json
from pathlib import Path
import unittest
import audit

EVIDENCE = Path(__file__).resolve().parent


class SourceAccounting(unittest.TestCase):
    def test_exact_rows(self):
        expected = [(n, s) for n, s in enumerate((EVIDENCE / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in audit.build().get('source_rows', [])], expected)
        self.assertTrue(all(r['capabilities'] == [] for r in audit.build()['source_rows']))

    def test_inventory_headers_and_navigation(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11500)
        self.assertEqual(ledger.get('literal_navigation'), {'patch': '1.15.0', 'prev': '1.14.4', 'next': '1.15.1'})
        self.assertEqual([(h['line'], h['label'], h['numerical_count']) for h in ledger.get('headers', [])], [(3, 'Summary', None), (9, 'Resources', None)])
        self.assertEqual(ledger.get('transclusions'), [{'line': 1, 'literal': '{{apichanges|1.15.0|prev=1.14.4|next=1.15.1}}', 'role': 'navigation-metadata-only', 'expansion': 'UNPROVEN'}])

    def test_distinct_inclusion_claims(self):
        prose = audit.build().get('summary_prose', [])
        self.assertEqual([(p['line'], p['scope']) for p in prose], [(4, 'all-Wrath-3.4.3-claim'), (5, 'unspecified-Dragonflight-10.1.7-through-10.2.0-subset'), (6, 'linked-guidance'), (7, 'seasonal-rune-namespace-addition')])
        self.assertTrue(all(p['expanded_members'] == [] and p['status'] == 'UNPROVEN' for p in prose))
        self.assertEqual(audit.build().get('literal_clients'), ['Wrath Classic', 'Dragonflight'])

    def test_links_exact_unexpanded(self):
        links = [c for c in audit.build().get('contracts', []) if c['kind'].startswith('unexpanded-')]
        self.assertEqual([(c['line'], c['label'], c['target']) for c in links], [
            (4, 'Wrath Classic', 'World of Warcraft: Wrath of the Lich King Classic'),
            (4, 'Patch 3.4.3', 'Patch 3.4.3/API changes'),
            (5, 'Patch 10.1.7', 'Patch 10.1.7/API changes'),
            (5, 'Patch 10.2.0', 'Patch 10.2.0/API changes'),
            (11, 'wow-ui-source', 'https://github.com/Gethe/wow-ui-source/compare/1.14.4..1.15.0'),
            (11, 'BlizzardInterfaceResources', 'https://github.com/Ketho/BlizzardInterfaceResources/compare/1.14.4..1.15.0'),
            (12, 'Blizzard_Deprecated', 'https://github.com/Gethe/wow-ui-source/tree/classic_era_ptr/Interface/AddOns/Blizzard_Deprecated/Deprecated_1_15_0.lua')])
        self.assertTrue(all(c['status'] == 'UNPROVEN' for c in links))

    def test_namespace_not_callable_or_modeled(self):
        ledger = audit.build()
        self.assertEqual([(a['line'], a['symbol'], a['kind'], a['direction']) for a in ledger.get('api_occurrences', [])], [(7, 'C_Engraving', 'namespace', 'added')])
        self.assertEqual(ledger.get('signatures'), [{'line': 7, 'symbol': 'C_Engraving', 'kind': 'namespace-functions-unspecified', 'functions': None, 'arguments': None, 'returns': None, 'status': 'UNPROVEN'}])
        self.assertEqual(ledger.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertTrue(all(c['arguments'] is None and c['returns'] is None and c['state_transitions'] is None and c['security_rules'] is None and c['native_equivalence'] is None for c in ledger['contracts']))

    def test_full_counts(self):
        self.assertEqual(audit.build().get('totals'), {'physical_lines': 12, 'nonblank_rows': 10, 'metadata_rows': 4, 'unproven_rows': 6, 'contracts': 11, 'headers': 2, 'numerical_inventory_headers': 0, 'api_occurrences': 1, 'namespace_occurrences': 1, 'callable_occurrences': 0, 'event_occurrences': 0, 'cvar_occurrences': 0, 'widget_method_occurrences': 0, 'command_occurrences': 0, 'signature_declarations': 0, 'unspecified_signature_records': 1, 'prose_contracts': 4, 'unexpanded_linked_contracts': 7, 'unexpanded_transclusions': 1})

    def test_configuration_and_successors(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([(s['patch'], s['state']) for s in ledger.get('queued_successors', [])], [(f'1.15.{n}', 'in-flight-pending-main-integration' if n == 1 else 'queued-pending-main-integration' if n == 2 else 'integrated-canonical-input-not-applied') for n in range(1, 10)])
        self.assertTrue(all(not s['applied'] and not s['native_proof'] for s in ledger['queued_successors']))

    def test_each_omission_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        for field in ['source_rows', 'contracts', 'headers', 'summary_prose', 'transclusions', 'api_occurrences', 'signatures', 'configured_profiles', 'queued_successors']:
            for index in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(index)
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed)

    def test_fabricated_credit_and_history_collapse_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        for field, value in [('later_registers', ['3.4.3']), ('later_registers', ['10.1.7', '10.2.0']), ('measurements', {'runtime': 1, 'model': 1, 'native': 1}), ('signatures', [{'symbol': 'C_Engraving.Invented'}])]:
            changed = copy.deepcopy(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for c in original['contracts']:
            changed = copy.deepcopy(original)
            next(row for row in changed['contracts'] if row['id'] == c['id'])['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        changed = copy.deepcopy(original)
        changed['summary_prose'][0]['scope'] = changed['summary_prose'][1]['scope']
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed)

    def test_identity_and_content_tamper(self):
        self.assertIn('source', audit.build())
        response = json.loads((EVIDENCE / 'source-response.json').read_bytes())
        response['query']['pages']['564510']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(EVIDENCE / 'source.wikitext').read_bytes() + b'\n')


if __name__ == '__main__':
    unittest.main(verbosity=2)
