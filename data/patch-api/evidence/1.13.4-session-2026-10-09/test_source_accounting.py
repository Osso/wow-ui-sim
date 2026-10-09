"""Frozen 1.13.4 SOURCE contracts, independent of current model/native credit."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent
GLOBALS = ['C_Commentator.GetUnitTeamIndex', 'C_StorePublic.HasPurchaseableProducts', 'C_SummonInfo.ConfirmSummon', 'CanInitiateWarGame', 'GetInstanceLockTimeRemaining', 'GetInstanceLockTimeRemainingEncounter', 'GetTotemInfo', 'GetTotemTimeLeft', 'GetWarGameQueueStatus', 'IsUsingFixedTimeStep', 'IsWargame', 'RespondInstanceLock', 'StartSpectatorWarGame', 'StartWarGame', 'StartWarGameByName', 'TargetTotem', 'WarGameRespond']
EVENTS = ['COMMENTATOR_IMMEDIATE_FOV_UPDATE', 'INSTANCE_LOCK_START', 'INSTANCE_LOCK_STOP', 'INSTANCE_LOCK_WARNING', 'WARGAME_REQUESTED']

class SourceAccounting(unittest.TestCase):

    def test_identity_and_response(self):
        source = audit.build().get('source', {})
        self.assertEqual((source.get('pageid'), source.get('revid'), source.get('timestamp')), (333398, 3216451, '2021-05-06T13:21:20Z'))
        self.assertEqual(source.get('wikitext_bytes'), 2146)
        self.assertEqual(source.get('wikitext_sha256'), 'a8cbcc1f62bb90e33a40334dece4f943b3f44648c34d50fbbb0a55160b8e00f5')

    def test_lossless_raw_and_default_extract(self):
        ledger = audit.build()
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('source_rows', [])], expected)
        extract = audit.default_extract((E / 'source.wikitext').read_text())
        self.assertEqual(ledger.get('default_extract'), extract)
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('extracted_rows', [])], [(n, s) for n, s in enumerate(extract['text'].splitlines(), 1) if s.strip()])

    def test_inventory_and_unspecified_signatures_defaults(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], GLOBALS + EVENTS + ['ColorNameplateNameBySelection'])
        self.assertEqual([i['direction'] for i in ledger['inventory']], ['added'] * 23)
        self.assertEqual([s['symbol'] for s in ledger.get('signatures', [])], GLOBALS + EVENTS + ['ColorNameplateNameBySelection'])
        self.assertTrue(all((s['arguments'] is None and s['returns'] is None and (s['payload'] is None) and (s['declaration'] is None) for s in ledger['signatures'])))
        self.assertEqual(ledger.get('defaults'), [])

    def test_headers_counts_caption_and_navigation(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger.get('headers', [])], ['Diffs', 'Changes', 'Global API', 'Events', 'CVars', 'References'])
        self.assertEqual([(h['section'], h['count'], h['observed']) for h in ledger.get('inventory_headers', [])], [('global-api', 17, 17), ('events', 5, 5), ('cvars', 1, 1)])
        self.assertEqual(ledger.get('navigation'), dict(patch='1.13.4', prev='1.13.3', next='1.13.5'))
        self.assertEqual(ledger.get('source_toc'), 11304)
        self.assertEqual(ledger.get('captions'), [dict(line=4, literal='1.13.3 (32790) to 1.13.4 (33491)', before_patch='1.13.3', before_build=32790, after_patch='1.13.4', after_build=33491)])

    def test_prose_links_templates_and_client_limits(self):
        ledger = audit.build()
        self.assertEqual([(p['line'], p['claim']) for p in ledger.get('prose', [])], [(10, 'Totem API reinstated'), (11, 'WoW Token support in China')])
        self.assertEqual(len(ledger.get('links', [])), 5)
        self.assertEqual(len(ledger.get('templates', [])), 26)
        self.assertEqual(ledger.get('references'), [])
        self.assertTrue(all((not x['expanded'] for key in ('links', 'templates') for x in ledger[key])))
        self.assertIsNone(ledger.get('source_client_literal'))
        self.assertEqual(len(ledger.get('contracts', [])), 30)
        self.assertTrue(all((x['status'] == 'UNPROVEN' for x in ledger['contracts'])))

    def test_model_and_history_credit_separate(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([s['patch'] for s in ledger.get('successors', [])], ['1.13.5', '1.13.6', '1.13.7', '1.14.0', '1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)])
        self.assertTrue(all((not s['applied'] and (not s['native_proof']) for s in ledger['successors'])))
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        self.assertTrue(all((x['status'] == 'UNPROVEN' for x in ledger.get('model_review', []))))

    def test_each_omission_and_invented_contract_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        for field in audit.OCCURRENCE_FIELDS:
            for index in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(index)
                with self.assertRaises(AssertionError, msg=f'{field}/{index}'):
                    audit.validate_ledger(changed)
        for field, value in [('measurements', dict(runtime=1, model=1, native=1)), ('source_toc', 11507), ('defaults', [1]), ('later_registers', ['2.5.1']), ('source_client_literal', 'Retail')]:
            changed = copy.deepcopy(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field, key, value in [('signatures', 'arguments', ['slot']), ('templates', 'expanded', True), ('inventory', 'symbol', 'C_StorePublic.DoesGroupHavePurchaseableProducts')]:
            changed = copy.deepcopy(original)
            changed[field][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_source_tamper_rejected(self):
        self.assertIn('source', audit.build())
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['333398']['revisions'][0]['revid'] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(json.dumps(response).encode(), (E / 'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError):
            audit.validate_source((E / 'source-response.json').read_bytes(), (E / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
