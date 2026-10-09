"""Frozen 1.13.7 SOURCE behavior, not native/model/runtime acceptance."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent
SYMBOLS = ['GetDefaultScale', 'heardChoiceSFX', 'lastCharacterGuid', 'seenTBCInfoPane', 'showUnactivatedCharacters']

class SourceAccounting(unittest.TestCase):
    def test_exact_source_identity(self):
        source = audit.build().get('source', {})
        self.assertEqual((source.get('pageid'), source.get('revid'), source.get('timestamp')), (46829, 458410, '2021-09-04T08:55:38Z'))
        self.assertEqual(source.get('wikitext_bytes'), 1179)

    def test_lossless_raw_and_default_extract(self):
        ledger = audit.build()
        expected = [(n, s) for n, s in enumerate((E/'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('source_rows', [])], expected)
        extracted = [(n, s) for n, s in enumerate((E/'default-extract.txt').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in ledger.get('extracted_rows', [])], extracted)

    def test_inventory_signatures_and_no_defaults(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], SYMBOLS)
        self.assertEqual([(i['section'], i['direction']) for i in ledger['inventory']], [('global-api', 'added')]+[('cvars', 'added')]*4)
        self.assertEqual(ledger.get('signatures'), [dict(line=13, symbol='GetDefaultScale', arguments=None, returns=None, declaration=None, status='UNPROVEN')])
        self.assertEqual([(i['symbol'], i['default'], i['description']) for i in ledger.get('cvars', [])], [(s, None, None) for s in SYMBOLS[1:]])

    def test_headers_navigation_and_unexpanded_links(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger.get('headers', [])], ['Summary', 'Global API', 'CVars'])
        self.assertEqual([(h['section'], h['direction'], h['count']) for h in ledger.get('inventory_headers', [])], [('global-api','added',None),('cvars','added',None)])
        self.assertEqual(ledger.get('navigation'), dict(patch='1.13.7', prev='1.13.6', next='1.14.0'))
        self.assertEqual(ledger.get('source_toc'), 11307)
        self.assertEqual([l['label'] for l in ledger.get('links', [])], ['wow-ui-source', 'BlizzardInterfaceResources'])
        self.assertEqual(len(ledger.get('templates', [])), 6)
        self.assertTrue(all(not t['expanded'] for t in ledger['templates']))

    def test_literal_limits_and_zero_credit(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('measurements'), dict(runtime=0, model=0, native=0))
        self.assertEqual(len(ledger.get('contracts', [])), 7)
        self.assertTrue(all(c['status']=='UNPROVEN' for c in ledger['contracts']))
        for field in ('examples', 'prose', 'later_registers'):
            self.assertEqual(ledger.get(field), [])
        self.assertIsNone(ledger.get('source_client_literal'))

    def test_configured_profiles_and_same_history_boundaries(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'],p['configured_interface']) for p in ledger.get('configured_profiles',[])], [('client-era',11507),('client-anniversary',11507)])
        expected=[('1.14.0','completed-queued'),('1.14.1','completed-queued')]+[(v,'integrated-not-applied') for v in ['1.14.2','1.14.3','1.14.4']+[f'1.15.{n}' for n in range(10)]]
        self.assertEqual([(p['patch'],p['state']) for p in ledger.get('successors',[])], expected)
        self.assertTrue(all(not p['applied'] and not p['native_proof'] for p in ledger['successors']))

    def test_every_omission_and_fabrication_rejected(self):
        original=audit.build()
        self.assertIn('contracts',original)
        for field in ('source_rows','extracted_rows','inventory','signatures','cvars','headers','inventory_headers','links','templates','contracts','configured_profiles','successors'):
            for index in range(len(original[field])):
                changed=copy.deepcopy(original); changed[field].pop(index)
                with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for field,value in [('measurements',dict(runtime=1,model=1,native=1)),('source_toc',11507),('examples',['invented']),('later_registers',['2.5.4']),('source_client_literal','TBC')]:
            changed=copy.deepcopy(original); changed[field]=value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        changed=copy.deepcopy(original); changed['signatures'][0]['arguments']=[]
        with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        changed=copy.deepcopy(original); changed['cvars'][0]['default']='0'
        with self.assertRaises(AssertionError): audit.validate_ledger(changed)

    def test_response_and_body_tamper(self):
        self.assertIn('source',audit.build())
        response=json.loads((E/'source-response.json').read_bytes())
        response['query']['pages']['46829']['revisions'][0]['revid']+=1
        with self.assertRaises(AssertionError): audit.validate_source(json.dumps(response).encode(),(E/'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError): audit.validate_source((E/'source-response.json').read_bytes(),(E/'source.wikitext').read_bytes()+b'\n')

if __name__=='__main__': unittest.main(verbosity=2)
