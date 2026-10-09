"""Own literal SOURCE tests: no runtime, native, linked or successor credit."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent
GLOBALS = ['C_SummonInfo.CancelSummon', 'C_SummonInfo.ConfirmSummon', 'C_SummonInfo.GetSummonConfirmAreaName', 'C_SummonInfo.GetSummonConfirmSummoner', 'C_SummonInfo.GetSummonConfirmTimeLeft', 'C_SummonInfo.GetSummonReason', 'C_SummonInfo.IsSummonSkippingStartExperience', 'C_VoiceChat.IsChannelJoinPending', 'GetCombatRating', 'GetCombatRatingBonus', 'HasKey', 'KeyRingButtonIDToInvSlotID', 'CancelSummon', 'ConfirmSummon', 'GetSummonConfirmAreaName', 'GetSummonConfirmSummoner', 'GetSummonConfirmTimeLeft', 'GetTotemCannotDismiss', 'GetTotemInfo', 'GetTotemTimeLeft', 'IsKioskModeEnabled', 'TargetTotem']
CVARS = ['KioskCanSessionExpire', 'KioskCharacterTemplateSet', 'KioskLobbyKickSeconds', 'showKeyring']

class SourceAccounting(unittest.TestCase):

    def test_exact_frozen_identity(self):
        source = audit.build().get('source', {})
        self.assertEqual((source.get('pageid'), source.get('revid'), source.get('timestamp')), (455918, 6472111, '2025-09-13T22:49:36Z'))
        self.assertEqual(source.get('wikitext_bytes'), 2568)
        self.assertEqual(source.get('wikitext_sha256'), '5aac9c7ecacf17d3d3f2961045252ac27b91884a4a2abde60f817ee2f099e85b')

    def test_every_raw_row_and_historical_default_boundary(self):
        ledger = audit.build()
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('source_rows', [])], expected)
        self.assertEqual(len(expected), 56)
        self.assertEqual(ledger.get('extracted_rows'), [])
        self.assertEqual(ledger.get('default_extract'), json.loads((E / 'default-extract-outcome.json').read_bytes()))
        self.assertEqual(ledger['default_extract']['result'], 'existing-failure')
        self.assertFalse((E / 'default-extract.txt').exists())

    def test_literal_inventory_not_truncated_default_register(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], GLOBALS + ['CORPSE_POSITION_UPDATE'] + CVARS)
        self.assertEqual([(i['section'], i['direction']) for i in ledger['inventory']], [('global-api', 'added')] * 12 + [('global-api', 'removed')] * 10 + [('events', 'added')] + [('cvars', 'added')] * 4)
        self.assertEqual([i['symbol'] for i in ledger.get('default_inventory', [])], GLOBALS)
        self.assertEqual([i['symbol'] for i in ledger.get('default_omissions', [])], ['CORPSE_POSITION_UPDATE'] + CVARS)
        self.assertEqual(ledger.get('defaults'), [])

    def test_signature_fragments_do_not_invent_parameters(self):
        ledger = audit.build()
        self.assertEqual(len(ledger.get('signatures', [])), 31)
        self.assertEqual([s['symbol'] for s in ledger['signatures'] if s['kind'] == 'prose-empty-parentheses'], ['SendChatMessage', 'SendAddonMessage', 'UnitHealth', 'UnitHealthMax'])
        self.assertTrue(all((s['arguments'] is None and s['returns'] is None and (s['declaration'] is None) for s in ledger['signatures'])))
        self.assertTrue(all((s['fragment'] == '()' for s in ledger['signatures'] if s['kind'] == 'prose-empty-parentheses')))

    def test_heading_counts_captions_navigation(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger.get('headers', [])], ['Diffs', 'Changes', 'Global API', 'Events', 'Added', 'CVars', 'Added', 'References'])
        self.assertEqual([(h['count'], h['observed'], h['direction']) for h in ledger.get('inventory_headers', [])], [(12, 12, 'added'), (10, 10, 'removed')])
        self.assertEqual([c['literal'] for c in ledger.get('captions', [])], ['1.13.2 (32421) to 1.13.3 (32790)', '|+ Global API 1.13.2 (32421) &rarr; 1.13.3 (32790)'])
        self.assertEqual(ledger.get('navigation'), dict(patch='1.13.3', prev='1.13.2', next='1.13.4'))
        self.assertEqual(ledger.get('source_toc'), 11303)

    def test_prose_and_reference_boundaries(self):
        ledger = audit.build()
        self.assertEqual([x['line'] for x in ledger.get('prose', [])], [10, 11, 12, 13])
        self.assertEqual(ledger['prose'][2]['removed_chat_types'], ['CHANNEL'])
        self.assertEqual(ledger['prose'][2]['added_chat_types'], ['SAY', 'YELL'])
        self.assertEqual(ledger['prose'][3]['subjects'], ['UnitHealth', 'UnitHealthMax'])
        self.assertEqual(ledger['prose'][3]['scope'], 'NPCs; hotfixed; values again instead of percentages')
        self.assertEqual(len(ledger.get('links', [])), 7)
        self.assertEqual(len(ledger.get('templates', [])), 35)
        self.assertEqual(len(ledger.get('references', [])), 2)
        self.assertTrue(all((not x['expanded'] for field in ['links', 'templates', 'references'] for x in ledger[field])))
        self.assertEqual(len(ledger.get('contracts', [])), 38)
        self.assertTrue(all((c['status'] == 'UNPROVEN' for c in ledger['contracts'])))

    def test_current_state_and_successor_limits(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual(len(ledger.get('model_review', [])), 7)
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertEqual(ledger.get('later_registers'), [])
        q = ledger.get('queued_successor', {})
        self.assertEqual((q.get('patch'), q.get('state'), q.get('register')), ('1.13.4', 'concurrently-active-not-integrated', None))
        self.assertEqual(q.get('integration_order'), ['1.13.4', '1.13.3'])
        self.assertEqual([s['patch'] for s in ledger.get('successors', [])], ['1.13.5', '1.13.6', '1.13.7'] + [f'1.14.{n}' for n in range(5)] + [f'1.15.{n}' for n in range(10)])
        self.assertTrue(all((not s['applied'] and (not s['native_proof']) for s in ledger['successors'])))

    def test_all_occurrence_omissions_and_fabricated_credit_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        for field in audit.OCCURRENCE_FIELDS:
            for n in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(n)
                with self.assertRaises(AssertionError, msg=f'{field}/{n}'):
                    audit.validate_ledger(changed)
        for field, value in [('defaults', ['1']), ('measurements', dict(runtime=1, model=1, native=1)), ('later_registers', ['2.5.1']), ('source_toc', 11507)]:
            changed = copy.deepcopy(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field, key, value in [('signatures', 'arguments', []), ('inventory', 'alias', 'ConfirmSummon'), ('links', 'expanded', True), ('prose', 'default', 100)]:
            changed = copy.deepcopy(original)
            changed[field][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_identity_and_content_tamper_rejected(self):
        self.assertIn('source', audit.build())
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['455918']['revisions'][0]['revid'] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(json.dumps(response).encode(), (E / 'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError):
            audit.validate_source((E / 'source-response.json').read_bytes(), (E / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
