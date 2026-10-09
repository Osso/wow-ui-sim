"""Bounded literal SOURCE fixtures; no native, expanded or name-factory credit."""
from collections import Counter
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):

    def test_identity(self):
        p = audit.build().get('source', {})
        self.assertEqual((p.get('pageid'), p.get('revid'), p.get('timestamp')), (115161, 6471351, '2025-09-13T09:41:57Z'))
        self.assertEqual(p.get('wikitext_bytes'), 171914)
        self.assertEqual(p.get('wikitext_sha256'), 'ea1badd4412eb6d50257ae97b77d4155c96c5043469c099957a8d85744332526')
        self.assertEqual(p.get('response_sha256'), '81bb1f86038a137f16c0c4b80d1a89ac5e5b8a3eea234500ed3b5ca423eaf4d3')

    def test_all_raw_rows_and_default_bytes(self):
        d = audit.build()
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(i['line'], i['literal']) for i in d.get('source_rows', [])], expected)
        self.assertEqual(len(expected), 2838)
        self.assertEqual(d.get('default_extract'), json.loads((E / 'default-extract-outcome.json').read_bytes()))
        self.assertEqual(d['default_extract']['result'], 'existing-failure')
        self.assertEqual(d.get('default_inventory'), json.loads((E / 'default-register.json').read_bytes())['entries'])
        self.assertEqual(d.get('extracted_rows'), [])
        self.assertEqual(d.get('default_omissions'), [])
        self.assertFalse((E / 'default-extract.txt').exists())

    def test_full_inventory_and_original_link_disagreements(self):
        d = audit.build()
        self.assertEqual(Counter(((i['section'], i['direction']) for i in d.get('inventory', []))), {('global-api', 'added'): 121, ('global-api', 'removed'): 2008, ('widgets', 'added'): 4, ('widgets', 'removed'): 68, ('events', 'added'): 8, ('events', 'removed'): 449, ('cvars', 'added'): 30, ('cvars', 'removed'): 70})
        byline = {i['line']: i for i in d['inventory']}
        self.assertEqual(byline[144]['target'], 'API SelectTCradeSkill')
        self.assertEqual(byline[144]['symbol'], 'SelectTradeSkill')
        self.assertEqual(byline[2200]['target'], 'API DressupModel GetSlotTransmogSources')
        self.assertEqual(byline[2200]['symbol'], 'GameTooltip:GetSlotTransmogSources')
        self.assertEqual(sum((i['kind'] == 'widget-type' for i in d['inventory'])), 6)
        self.assertTrue(all((i['status'] == 'UNPROVEN' and (not i['capabilities']) for i in d['inventory'])))

    def test_headers_counts_and_captions(self):
        d = audit.build()
        self.assertEqual([i['label'] for i in d.get('headers', [])], ['[[FrameXML]] diffs', 'Changes', 'API', 'Cast Bars', 'CVars', 'Spells', 'Global API', 'Widgets', 'Events', 'CVars', 'References'])
        self.assertEqual([(i['count'], i['observed'], i['kind']) for i in d.get('inventory_headers', [])], [(121, 121, 'functions'), (2008, 2008, 'functions'), (4, 4, 'methods'), (62, 62, 'methods'), (6, 6, 'widgets'), (8, 8, 'events'), (449, 449, 'events'), (30, 30, 'CVars'), (70, 70, 'CVars')])
        self.assertEqual(len(d.get('captions', [])), 4)
        self.assertTrue(all((i['before_patch'] == '8.1.5' and i['after_patch'] == '1.13.2' for i in d['captions'])))
        self.assertEqual(d.get('navigation'), {'patch': '1.13.2', 'next': '1.13.3'})
        self.assertEqual(d.get('source_toc'), 11302)

    def test_prose_defaults_and_signature_limits(self):
        d = audit.build()
        self.assertEqual([i['line'] for i in d.get('prose', []) if i['role'] == 'behavior'], [14, 15, 16, 19, 20, 21, 24, 25, 28, 29, 30, 33, 34])
        self.assertEqual(d.get('defaults'), [{'line': 29, 'symbol': 'chatClassColorOverride', 'default_literal': '1', 'prior_literal': '0', 'scope': 'CVar default', 'status': 'UNPROVEN'}, {'line': 30, 'symbol': 'minimumAutomaticUiScale', 'default_literal': '0.9', 'prior_literal': '0.64', 'scope': 'default UI scale statement; full CVar contract unspecified', 'status': 'UNPROVEN'}])
        self.assertEqual([i['fragment'] for i in d.get('examples', [])], ['UnitCastingInfo("player")'])
        self.assertEqual(len([i for i in d.get('signatures', []) if i['kind'] == 'inventory-unspecified']), 2758)
        self.assertEqual([i['fragment'] for i in d['signatures'] if i['kind'] == 'prose-empty-parentheses'], ['()'] * 5)
        self.assertTrue(all((i['arguments'] is None and i['returns'] is None and (i['declaration'] is None) for i in d['signatures'])))
        self.assertTrue(all((not i['expanded'] for f in ['links', 'templates', 'references'] for i in d[f])))
        self.assertEqual(len(d['references']), 1)
        self.assertTrue(all((i['status'] == 'UNPROVEN' for i in d['contracts'])))

    def test_current_profiles_and_separate_successors(self):
        d = audit.build()
        self.assertEqual(d.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual([(i['feature'], i['configured_interface']) for i in d.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(d.get('later_registers'), [])
        self.assertEqual(d.get('integration_order'), ['1.13.2', '1.12.0'])
        self.assertEqual([i['patch'] for i in d.get('successors', [])], ['1.13.3', '1.13.4', '1.13.5', '1.13.6', '1.13.7'] + [f'1.14.{n}' for n in range(5)] + [f'1.15.{n}' for n in range(10)])
        self.assertTrue(all((not i['applied'] and (not i['native_proof']) for i in d['successors'])))
        self.assertEqual(d.get('foreign_history_policy'), 'Retail1.12.0 is a separate parallel audit; no Retail/TBC/Wrath/Forever semantic supersession.')
        self.assertTrue(d.get('model_review'))

    def test_every_omission_and_fabricated_credit_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        controls = 0
        for field in audit.OCCURRENCE_FIELDS:
            for n in range(len(original[field])):
                changed = dict(original)
                changed[field] = original[field][:n] + original[field][n + 1:]
                with self.assertRaises(AssertionError, msg=f'{field}/{n}'):
                    audit.validate_ledger(changed, expected=original)
                controls += 1
        self.assertEqual(controls, original['totals']['omission_controls'])
        for field, value in [('measurements', {'runtime': 1, 'model': 1, 'native': 1}), ('later_registers', ['1.12.0']), ('source_toc', 11507), ('defaults', [])]:
            changed = dict(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed, expected=original)
        for field, key, value in [('signatures', 'arguments', []), ('inventory', 'alias', 'SelectTradeSkill'), ('links', 'expanded', True), ('prose', 'native_proof', True)]:
            changed = dict(original)
            changed[field] = copy.deepcopy(original[field])
            changed[field][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed, expected=original)

    def test_source_tamper_rejected(self):
        self.assertIn('source', audit.build())
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['115161']['revisions'][0]['revid'] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(json.dumps(response).encode(), (E / 'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError):
            audit.validate_source((E / 'source-response.json').read_bytes(), (E / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
