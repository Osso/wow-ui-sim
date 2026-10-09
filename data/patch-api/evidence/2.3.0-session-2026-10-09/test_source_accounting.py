"""Owned source fixtures; never runtime/native tests."""
import copy
from pathlib import Path
import unittest

import accounting

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
RAW = ROOT / 'data/patch-api/sources/2.3.0-api-changes.wikitext'


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        self.raw = RAW.read_text()

    def test_mixed_labels_typos_rename_and_consolidated_occurrences(self):
        ledger = accounting.derive(self.raw)
        rows = ledger.get('inventory_rows', [])
        self.assertIn('GetNumTalentTabs', [r['symbol'] for r in rows])
        self.assertIn('GetTalentPrereqs', [r['symbol'] for r in rows])
        self.assertEqual([(r['symbol'], r['direction']) for r in rows
                          if r['wikitext_line'] == 23],
                         [('GameTooltip:SetTrackingSpell', 'removed'),
                          ('GameTooltip:SetTracking', 'added')])
        self.assertEqual([r['symbol'] for r in rows if r['wikitext_line'] == 7],
                         ['/targetlastenemy', '/targetlastfriend'])
        self.assertEqual([r['symbol'] for r in rows if r['section'] == 'events'],
                         ['INSPECT_TALENT_READY'])
        self.assertEqual(sum(r['symbol'] == 'GetMacroItem' for r in rows), 2)
        self.assertEqual([r for r in rows if r['section'] == 'cvars'], [])
        self.assertEqual([r['symbol'] for r in rows if r['direction'] == 'noted'],
                         ['GetSendMailPrice', 'GetInboxHeaderInfo'])
        self.assertTrue(all(r['direction'] == 'listed' for r in rows
                            if r['origin'] == 'consolidated-link'))

    def test_literal_return_prefixes_optional_arguments_and_notes(self):
        signatures = accounting.derive(self.raw).get('signature_ledger', [])
        self.assertIn('canQuery, canQueryAll = ', [r['return_prefix'] for r in signatures])
        self.assertIn('GetItemCount(item, includeBank,[ includeUses])',
                      [r['fragment'] for r in signatures])
        self.assertIn('GetNumTalentTabs ([inspect])', [r['fragment'] for r in signatures])
        self.assertIn('GetSendMailPrice', [r['symbol'] for r in signatures])
        self.assertTrue(all(not r['complete_signature'] for r in signatures))
        self.assertTrue(all(r['status'] == 'UNPROVEN' and not r['capabilities']
                            for r in signatures))

    def test_every_nonblank_line_and_header_literal(self):
        ledger = accounting.derive(self.raw)
        expected = [(n, line) for n, line in enumerate(self.raw.splitlines(), 1)
                    if line.strip()]
        self.assertEqual([(r['wikitext_line'], r['source_text'])
                          for r in ledger.get('source_rows', [])], expected)
        self.assertEqual(len(ledger['headers']), 11)
        self.assertEqual(ledger['header_counts'], [])
        self.assertEqual(ledger['runtime_observations'], 0)
        self.assertEqual(ledger['native_observations'], 0)
        self.assertEqual(ledger['model_credit'], 0)

    def test_precise_prose_and_unexpanded_reference_limits(self):
        ledger = accounting.derive(self.raw)
        prose = ledger.get('prose_ledger', [])
        self.assertIn('mail-attachment-index-state', [r['limit_key'] for r in prose])
        self.assertIn('auction-query-throttle-and-sort-state', [r['limit_key'] for r in prose])
        self.assertIn('scroll-child-rect-and-event-lifecycle', [r['limit_key'] for r in prose])
        self.assertIn('macro-dispatch-and-conditions', [r['limit_key'] for r in prose])
        self.assertIn('unitname-absent-unit-regression', [r['limit_key'] for r in prose])
        self.assertTrue(ledger['links'])
        self.assertTrue(all(not r['expanded'] and r['status'] == 'UNPROVEN'
                            for r in ledger['links']))
        self.assertEqual(ledger['later_registers'], [])
        self.assertEqual(ledger['successor_queue'],
                         ['2.4.0', '2.4.2', '3.0.2', '3.0.3', '3.0.8'])

    def test_parenthetical_prose_is_not_api_or_command_publication(self):
        ledger = accounting.derive(self.raw)
        self.assertNotIn('/dismount', [r['symbol'] for r in ledger['inventory_rows']])
        self.assertEqual([r['symbol'] for r in ledger['inventory_rows']
                          if r['section'] == 'commands'],
                         ['/petautocasttoggle', '/targetlastenemy', '/targetlastfriend',
                          '/targetexact', '/cancelform'])
        symbols = {r['symbol'] for r in ledger['signature_ledger']}
        self.assertTrue(symbols.isdisjoint({'instantly', 'form', 'units', 'First',
                                           'second', 'data', 'for', 'retrieve',
                                           'used', 'down', 'on', 'add', 'into'}))

    def test_every_omission_and_fabricated_credit_rejected(self):
        ledger = accounting.derive(self.raw)
        self.assertEqual(accounting.summary(ledger), {
            'source_rows': 163, 'statuses': {'metadata-only': 13, 'UNPROVEN': 150},
            'inventory_occurrences': 132,
            'kinds': {'commands': 5, 'widgets': 10, 'global-api': 116, 'events': 1},
            'directions': {'added': 28, 'changed': 18, 'removed': 2,
                           'noted': 2, 'listed': 82},
            'signatures': 137, 'literal_call_fragments': 50,
            'complete_signatures': 0, 'prose_limits': 68, 'headers': 11,
            'numerical_headers': 0, 'unexpanded_links': 82, 'cvar_occurrences': 0,
            'runtime_observations': 0, 'native_observations': 0, 'model_credit': 0,
        })
        for field in ['inventory_rows', 'signature_ledger', 'prose_ledger',
                      'source_rows', 'headers', 'links']:
            for index in range(len(ledger[field])):
                changed = copy.deepcopy(ledger)
                changed[field].pop(index)
                with self.subTest(field=field, index=index), self.assertRaises(AssertionError):
                    accounting.validate_ledger(self.raw, changed)
        for field in ['inventory_rows', 'signature_ledger', 'prose_ledger']:
            for key, value in [('status', 'native-covered'), ('capabilities', ['modeled'])]:
                changed = copy.deepcopy(ledger)
                changed[field][0][key] = value
                with self.subTest(field=field, key=key), self.assertRaises(AssertionError):
                    accounting.validate_ledger(self.raw, changed)
        for field, value in [('runtime_observations', 1), ('native_observations', 1),
                             ('model_credit', 1), ('client_line', 'wrath-classic'),
                             ('later_registers', ['2.5.1']), ('later_registers', ['3.4.0']),
                             ('later_registers', ['era']), ('later_registers', ['2.4.0'])]:
            changed = copy.deepcopy(ledger)
            changed[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(AssertionError):
                accounting.validate_ledger(self.raw, changed)


if __name__ == '__main__':
    unittest.main(verbosity=2)
