"""Frozen literal contracts, not native/runtime acceptance."""
import copy
import json
from pathlib import Path
import unittest
import audit

E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):
    def setUp(self):
        self.assertIn('source_rows', audit.build(), 'literal accounting absent')

    def test_identity(self):
        source = audit.build()['source']
        self.assertEqual((source['version'], source['pageid'], source['revid'], source['timestamp']), ('1.14.1', 102588, 1010974, '2022-03-02T05:21:47Z'))
        self.assertEqual(audit.build()['source_toc'], 11401)

    def test_lossless_rows(self):
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in audit.build()['source_rows']], expected)
        self.assertEqual(len(expected), 114)

    def test_exact_inventory_counts_and_typo(self):
        inventory = audit.build()['inventory']
        expected = [('Global API', 'added', 14), ('Global API', 'removed', 3), ('Widgets', 'added', 2), ('Widgets', 'removed', 0), ('Events', 'added', 31), ('Events', 'removed', 1), ('CVars', 'added', 7), ('CVars', 'removed', 1)]
        for section, direction, count in expected:
            self.assertEqual(sum(r['section'] == section and r['direction'] == direction for r in inventory), count)
        self.assertEqual([r['symbol'] for r in inventory if r['section'] == 'Widgets'], ['FontString:GetTextScale', 'FontStringSetTextScale'])
        self.assertTrue(all(r['arguments'] is None and r['returns'] is None and r['status'] == 'UNPROVEN' for r in inventory))
        self.assertEqual(len(audit.build()['signatures']), 59)

    def test_headers_caption_navigation(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger['headings']], ['Summary', 'Resources', 'Global API', 'Widgets', 'Events', 'CVars'])
        self.assertEqual([h['count'] for h in ledger['inventory_headers']], [14, 3, 2, 0, 31, 1, 7, 1])
        self.assertTrue(all(h['count'] == h['observed_count'] for h in ledger['inventory_headers']))
        self.assertEqual(ledger['caption']['literal'], '|+ 1.14.0 (39958) &rarr; 1.14.1 (41030) Nov 10 2021')
        self.assertEqual(ledger['navigation'], {'patch': '1.14.1', 'prev': '1.14.0', 'next': '1.14.2', 'expanded': False})

    def test_prose_defaults_and_templates(self):
        ledger = audit.build()
        self.assertEqual([p['line'] for p in ledger['summary_prose']], [4, 5, 6])
        self.assertEqual(ledger['summary_prose'][2]['default'], 'disabled')
        self.assertEqual([p['default'] for p in ledger['cvar_contracts']], ['0.333333', '0', '0', '0', None, None, None, None])
        self.assertEqual(ledger['cvar_contracts'][3]['scope'], 'Account')
        self.assertEqual([p['symbol'] for p in ledger['template_claims']], ['SharedTooltipTemplate', 'GameTooltipTemplate'])
        self.assertTrue(all(p['no_longer_inherits'] == 'BackdropTemplate' for p in ledger['template_claims']))
        self.assertEqual(len(ledger['transclusions']), 57)
        self.assertEqual(len(ledger['links']), 14)
        self.assertTrue(all(not p['expanded'] for p in ledger['links'] + ledger['transclusions']))

    def test_boundaries(self):
        ledger = audit.build()
        self.assertEqual(ledger['measurements'], {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual(ledger['later_registers'], [])
        self.assertEqual([p['configured_interface'] for p in ledger['configured_profiles']], [11507, 11507])
        self.assertEqual([s['patch'] for s in ledger['successors']], ['1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)])
        self.assertTrue(all(not s['applied'] and not s['native_proof'] for s in ledger['successors']))

    def test_every_omission(self):
        original = audit.build()
        count = 0
        for field in ['source_rows', 'inventory', 'signatures', 'headings', 'inventory_headers', 'summary_prose', 'cvar_contracts', 'template_claims', 'transclusions', 'links', 'configured_profiles', 'successors', 'model_review']:
            for n in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(n)
                with self.assertRaises(AssertionError):
                    audit.validate_ledger(changed)
                count += 1
        print('omission controls:', count)

    def test_fabrications(self):
        for field, value in [('later_registers', ['10.1.7', '3.4.3']), ('measurements', {'runtime': 1, 'model': 1, 'native': 1})]:
            changed = audit.build()
            changed[field] = value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for field, key, value in [('inventory', 'symbol', 'FontString:SetTextScale'), ('cvar_contracts', 'default', '1'), ('summary_prose', 'default', 'enabled')]:
            changed = audit.build()
            changed[field][-1][key] = value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['102588']['revisions'][0]['revid'] += 1
        with self.assertRaises(AssertionError): audit.validate_source(response=json.dumps(response).encode())

if __name__ == '__main__':
    unittest.main(verbosity=2)
