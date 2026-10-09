"""Own frozen Patch 1.13.5 SOURCE contracts; no runtime/native acceptance."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent
SYMBOLS = ['UnitDetailedThreatSituation', 'UnitThreatPercentageOfLead', 'UnitThreatSituation', 'UNIT_THREAT_LIST_UPDATE', 'UNIT_THREAT_SITUATION_UPDATE']

class SourceAccounting(unittest.TestCase):

    def test_frozen_identity_and_response(self):
        source = audit.build().get('source', {})
        self.assertEqual((source.get('pageid'), source.get('revid'), source.get('timestamp')), (398847, 3835002, '2021-05-06T13:20:38Z'))
        self.assertEqual(source.get('wikitext_bytes'), 1690)
        self.assertEqual(source.get('wikitext_sha256'), 'f8a5356cff1fc8c2188b68aa77ab60969c102ea64767a4f4721fd07532f85ac2')

    def test_lossless_rows_and_default_extract(self):
        ledger = audit.build()
        expected = [(n, s) for n, s in enumerate((E / 'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('source_rows', [])], expected)
        self.assertEqual(len(expected), 31)
        self.assertEqual(ledger.get('extracted_rows'), [])
        failure = ledger.get('default_extract', {})
        self.assertEqual(failure.get('result'), 'existing-failure')
        self.assertIsNone(failure.get('text'))
        self.assertEqual(failure.get('error_type'), 'ValueError')
        self.assertIn('unhandled template: * Reinstated [[Threat]] API', failure.get('error', ''))
        self.assertFalse((E / 'default-extract.txt').exists())

    def test_inventory_and_unknown_signatures(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], SYMBOLS)
        self.assertEqual([(i['section'], i['direction']) for i in ledger['inventory']], [('global-api', 'added')] * 3 + [('events', 'added')] * 2)
        self.assertEqual([(i['line'], i['symbol'], i['arguments'], i['returns'], i['declaration']) for i in ledger.get('signatures', [])], [(n, s, None, None, None) for n, s in zip([19, 20, 21, 31, 32], SYMBOLS)])
        self.assertEqual(ledger.get('defaults'), [])

    def test_headings_count_headers_caption_navigation(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger.get('headers', [])], ['Diffs', 'Changes', 'Global API', 'Events', 'References'])
        self.assertEqual([(h['section'], h['count'], h['observed']) for h in ledger.get('inventory_headers', [])], [('global-api', 3, 3), ('events', 2, 2)])
        self.assertEqual(ledger.get('navigation'), dict(patch='1.13.5', prev='1.13.4', next='1.13.6'))
        self.assertEqual(ledger.get('source_toc'), 11305)
        self.assertEqual(ledger.get('captions'), [dict(line=4, literal='1.13.4 (34600) to 1.13.5 (34713)', before_patch='1.13.4', before_build=34600, after_patch='1.13.5', after_build=34713)])

    def test_prose_reference_templates_and_unexpanded_limits(self):
        ledger = audit.build()
        self.assertEqual([p['line'] for p in ledger.get('prose', [])], [10, 11])
        self.assertEqual(ledger.get('prose', [{}])[1].get('additional_slots') if len(ledger.get('prose', [])) > 1 else None, 4)
        self.assertEqual(len(ledger.get('links', [])), 7)
        self.assertEqual(len(ledger.get('templates', [])), 7)
        self.assertEqual(len(ledger.get('references', [])), 2)
        self.assertTrue(all((not x['expanded'] for field in ('links', 'templates', 'references') for x in ledger[field])))
        self.assertEqual(ledger.get('source_client_literal'), 'WoW Classic')
        self.assertEqual(ledger.get('client_literal_scope'), 'reference title, not expanded linked contents or configured native identity')
        self.assertEqual(len(ledger.get('contracts', [])), 14)
        self.assertTrue(all((c['status'] == 'UNPROVEN' for c in ledger['contracts'])))

    def test_current_model_and_successor_credit_boundaries(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual(len(ledger.get('model_review', [])), 4)
        self.assertTrue(all((x['status'] == 'UNPROVEN' for x in ledger['model_review'])))
        self.assertEqual([(p['feature'], p['configured_interface']) for p in ledger.get('configured_profiles', [])], [('client-era', 11507), ('client-anniversary', 11507)])
        queued = ledger.get('queued_successor', {})
        self.assertEqual((queued.get('patch'), queued.get('state'), queued.get('register')), ('1.13.6', 'concurrently-active-not-integrated', None))
        self.assertEqual(ledger.get('later_registers'), [])
        self.assertEqual([s['patch'] for s in ledger.get('successors', [])], ['1.13.7', '1.14.0', '1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)])
        self.assertTrue(all((not s['applied'] and (not s['native_proof']) for s in ledger['successors'])))

    def test_occurrence_omissions_and_invented_credit_rejected(self):
        original = audit.build()
        self.assertIn('contracts', original)
        for field in audit.OCCURRENCE_FIELDS:
            for index in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(index)
                with self.assertRaises(AssertionError, msg=f'{field}/{index}'):
                    audit.validate_ledger(changed)
        for field, value in [('measurements', dict(runtime=1, model=1, native=1)), ('source_toc', 11507), ('defaults', ['16']), ('later_registers', ['2.5.2']), ('source_client_literal', 'Retail'), ('queued_successor', dict(patch='1.13.6', applied=True))]:
            changed = copy.deepcopy(original)
            changed[field] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)
        for field, key, value in [('signatures', 'arguments', ['unit']), ('prose', 'final_capacity', 20), ('templates', 'expanded', True), ('inventory', 'symbol', 'UnitThreatLeadSituation')]:
            changed = copy.deepcopy(original)
            changed[field][0][key] = value
            with self.assertRaises(AssertionError):
                audit.validate_ledger(changed)

    def test_frozen_content_identity_tamper_rejected(self):
        self.assertIn('source', audit.build())
        response = json.loads((E / 'source-response.json').read_bytes())
        response['query']['pages']['398847']['revisions'][0]['revid'] += 1
        with self.assertRaises(AssertionError):
            audit.validate_source(json.dumps(response).encode(), (E / 'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError):
            audit.validate_source((E / 'source-response.json').read_bytes(), (E / 'source.wikitext').read_bytes() + b'\n')
if __name__ == '__main__':
    unittest.main(verbosity=2)
