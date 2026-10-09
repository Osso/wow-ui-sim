"""Frozen SOURCE contracts only; no runtime/native assertions."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):
    def test_literal_rows(self):
        expected = [(i, s) for i, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(x['line'], x['literal']) for x in audit.build().get('source_rows', [])], expected)

    def test_inventory_and_signature_limits(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('api_occurrences'), [
            {'line': 5, 'name': 'Enum.SeasonID.SeasonOfDiscovery', 'role': 'added-finite-enum-member', 'numeric_value': None, 'status': 'UNPROVEN'},
            {'line': 6, 'name': 'Enum.SeasonID.Placeholder', 'role': 'deprecated-alias', 'numeric_value': None, 'status': 'UNPROVEN'}])
        self.assertEqual(ledger.get('signatures'), [])
        self.assertEqual(ledger.get('totals'), {'physical_lines': 11, 'nonblank_rows': 9, 'metadata_rows': 4, 'unproven_rows': 5, 'api_occurrences': 2, 'signature_declarations': 0, 'prose_contracts': 3, 'unexpanded_linked_contracts': 4, 'contracts': 7, 'headers': 2, 'templates': 1})

    def test_concrete_alias_not_invented_value(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('alias'), {'name': 'Enum.SeasonID.Placeholder', 'target': 'Enum.SeasonID.SeasonOfDiscovery', 'literal_contract': 'continue working as-is for detecting Season of Discovery realms', 'numeric_value': None, 'status': 'UNPROVEN'})
        self.assertEqual([c['kind'] for c in ledger.get('contracts', [])], ['unspecified-retail-subset', 'unexpanded-retail-page', 'added-enum-member', 'deprecated-alias-continuity', 'unexpanded-diff', 'unexpanded-diff', 'unexpanded-deprecated-api'])

    def test_links_headers_and_template(self):
        ledger = audit.build()
        self.assertEqual([(c['line'], c['target']) for c in ledger.get('contracts', []) if c['kind'].startswith('unexpanded')], [
            (4, 'Patch 10.2.5/API changes'),
            (10, 'https://github.com/Gethe/wow-ui-source/compare/1.15.0..classic_era_ptr'),
            (10, 'https://github.com/Ketho/BlizzardInterfaceResources/compare/1.15.0..1.15.1'),
            (11, 'https://github.com/Gethe/wow-ui-source/tree/classic_era_ptr/Interface/AddOns/Blizzard_Deprecated/Deprecated_1_15_1.lua')])
        self.assertEqual(ledger.get('headers'), [{'line': 3, 'label': 'Summary', 'numerical_count': None}, {'line': 8, 'label': 'Resources', 'numerical_count': None}])
        self.assertEqual(ledger.get('templates'), [{'line': 1, 'literal': '{{apichanges|1.15.1|prev=1.15.0|next=1.15.2}}', 'expansion': 'UNPROVEN'}])

    def test_identity_and_source_tampering(self):
        self.assertEqual(audit.build().get('source', {}).get('revid'), 5998991)
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['577687']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(E / 'source.wikitext').read_bytes() + b'\n')

    def test_configuration_successors_not_native_credit(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11501)
        self.assertEqual(ledger.get('configured_interfaces'), {'era': 11507, 'anniversary': 11507})
        self.assertEqual(ledger.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual(ledger.get('later_registers'), [])
        successors = ledger.get('successors', [])
        self.assertEqual([s['patch'] for s in successors], [f'1.15.{n}' for n in range(2, 10)])
        self.assertEqual([s['state'] for s in successors], ['inflight', 'queued'] + ['integrated-canonical-not-applied'] * 6)
        self.assertTrue(all(s['applied'] is False and s['native_proof'] is False for s in successors))

    def test_every_omission_rejected(self):
        expected = audit.build()
        self.assertIn('contracts', expected)
        for field in ['source_rows', 'contracts', 'api_occurrences', 'headers', 'templates', 'successors']:
            for index in range(len(expected[field])):
                changed = copy.deepcopy(expected)
                changed[field].pop(index)
                with self.assertRaises(AssertionError, msg=f'{field}/{index}'):
                    audit.validate_ledger(changed)

    def test_fabricated_credit_values_foreign_supersession_rejected(self):
        expected = audit.build()
        self.assertIn('alias', expected)
        changes = [('alias', dict(expected['alias'], numeric_value=2)), ('later_registers', ['10.2.5']), ('later_registers', ['2.5.6']), ('measurements', {'runtime': 1, 'model': 1, 'native': 1}), ('signatures', [{'arguments': [], 'returns': [2]}])]
        for field, value in changes:
            changed = copy.deepcopy(expected)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for i in range(len(expected['contracts'])):
            changed = copy.deepcopy(expected)
            changed['contracts'][i]['status'] = 'bounded-coverage'
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

if __name__ == '__main__':
    unittest.main(verbosity=2)
