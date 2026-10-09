"""Literal 1.14.0 SOURCE tests; no runtime/native/model assertions."""
import copy
import json
from pathlib import Path
import re
import unittest
import audit
E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):

    def test_all_literal_rows(self):
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(x['line'], x['literal']) for x in audit.build().get('source_rows', [])], expected)
        self.assertTrue(all((x['capabilities'] == [] for x in audit.build()['source_rows'])))

    def test_inventory_columns_and_literal_identities(self):
        ledger = audit.build()
        entries = ledger.get('inventory', [])
        self.assertEqual(len(entries), 723)
        expected = {('Global API', 'added'): 385, ('Global API', 'removed'): 39, ('Widgets', 'added'): 37, ('Widgets', 'removed'): 8, ('Events', 'added'): 53, ('Events', 'removed'): 38, ('CVars', 'added'): 130, ('CVars', 'removed'): 33}
        self.assertEqual({k: sum(((e['section'], e['direction']) == k for e in entries)) for k in expected}, expected)
        self.assertEqual([(e['line'], e['symbol']) for e in entries if e['kind'] == 'widget-script'], [(488, 'Frame OnGamePadButtonDown'), (489, 'Frame OnGamePadButtonUp'), (490, 'Frame OnGamePadStick'), (491, 'Model OnAnimStarted')])
        self.assertIn('Frame:Frame:GetBackdrop', [e['symbol'] for e in entries])
        self.assertIn('GetBackdropBorderColor', [e['symbol'] for e in entries])
        self.assertNotIn('Scripts', [e['symbol'] for e in entries])
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('category_labels', [])], [(487, ': Scripts')])

    def test_headers_caption_and_navigation(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('source_toc'), 11400)
        self.assertEqual(ledger.get('navigation'), dict(patch='1.14.0', prev='1.13.7', next='1.14.1', like='2.5.2', expanded=False))
        self.assertEqual([(h['line'], h['label']) for h in ledger.get('headers', [])], [(3, 'Summary'), (10, 'Global API'), (447, 'Widgets'), (505, 'Events'), (608, 'CVars')])
        self.assertEqual([(h['header_count'], h['parsed_count']) for h in ledger.get('count_headers', [])], [(385, 385), (39, 39), (37, 37), (8, 8), (53, 53), (38, 38), (130, 130), (33, 33)])
        self.assertEqual(ledger.get('caption'), dict(line=12, literal='|+ 1.13.7 (39692) &rarr; PTR 1.14.0 (39958)', previous_patch='1.13.7', previous_build=39692, current_patch='1.14.0', current_build=39958, channel='PTR', native_proof=False))

    def test_baseline_is_not_successor_or_imported_behavior(self):
        ledger = audit.build()
        self.assertEqual([(p['line'], p['scope'], p['expanded_members']) for p in ledger.get('prose', [])], [(7, 'foreign-2.5.2-baseline-synchronization-claim', [])])
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([(s['patch'], s['state']) for s in ledger.get('successors', [])], [(v, 'in-flight-pending-main-integration' if v == '1.14.1' else 'queued-pending-main-integration' if v in ('1.14.2', '1.14.3') else 'actual-input-not-applied') for v in ['1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)]])
        self.assertTrue(all((not s['applied'] and (not s['native_proof']) for s in ledger['successors'])))

    def test_all_reference_and_template_boundaries(self):
        ledger = audit.build()
        self.assertEqual(len(ledger.get('templates', [])), 720)
        links = ledger.get('links', [])
        self.assertEqual(len(links), 10)
        self.assertEqual([(x['line'], x['target']) for x in links[:6]], [(5, 'https://github.com/Meorawr/wow-ui-source/commit/71009e66851884779971d1b5dbf6a40d3a441041'), (5, 'https://github.com/Ketho/BlizzardInterfaceResources/commit/a0cace4cbd441107a3fcb2f7cd1791f86152ecba'), (6, 'https://github.com/Stanzilla/WoWUIBugs/wiki/1.14.0-Consolidated-UI-Changes'), (7, 'Global_functions/Classic'), (7, 'Patch_2.5.2/API_changes'), (8, 'https://github.com/Ketho/WowpediaApiDoc/blob/master/Projects/DiffWikitext/DiffWikitext.lua')])
        self.assertTrue(all((x['expanded'] == False for x in links + ledger['templates'])))
        self.assertEqual(links[5]['status'], 'metadata-only')

    def test_no_invented_signatures_defaults_or_credit(self):
        ledger = audit.build()
        self.assertEqual(len(ledger.get('signature_limits', [])), 560)
        self.assertEqual(len(ledger.get('default_limits', [])), 163)
        self.assertTrue(all((s['arguments'] is None and s['returns'] is None and (s['payload'] is None) for s in ledger['signature_limits'])))
        self.assertTrue(all((s['default'] is None for s in ledger['default_limits'])))
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])

    def test_complete_totals(self):
        self.assertEqual(audit.build().get('totals'), dict(physical_lines=781, nonblank_rows=776, inventory=723, api_templates=719, widget_scripts=4, category_labels=1, headers=5, count_headers=8, count_conflicts=0, prose=1, links=10, templates=720, signature_limits=560, default_limits=163))

    def test_every_omission_rejected(self):
        original = audit.build()
        self.assertIn('inventory', original)
        for field in ['source_rows', 'inventory', 'headers', 'count_headers', 'prose', 'links', 'templates', 'category_labels', 'signature_limits', 'default_limits', 'configured_profiles', 'successors']:
            for index in range(len(original[field])):
                changed = dict(original)
                changed[field] = original[field][:index] + original[field][index + 1:]
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed, expected=original)

    def test_fabricated_credit_and_normalization_rejected(self):
        original = audit.build()
        self.assertIn('inventory', original)
        for field, value in [('measurements', dict(runtime=1, model=1, native=1)), ('later_registers', ['2.5.2']), ('prose', []), ('navigation', dict(patch='1.14.0', like='2.5.2', expanded=True))]:
            changed = copy.deepcopy(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field, key, value in [('inventory', 'status', 'bounded-coverage'), ('signature_limits', 'arguments', []), ('default_limits', 'default', '0'), ('inventory', 'symbol', 'InventedAlias')]:
            changed = copy.deepcopy(original)
            changed[field][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_source_identity_and_content_rejection(self):
        self.assertIn('source', audit.build())
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['71995']['revisions'][0]['revid'] += 1
        with self.assertRaisesRegex(AssertionError, 'revid'):
            audit.validate_source(response=json.dumps(response).encode())
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            audit.validate_source(raw=(E / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
