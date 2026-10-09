"""Own frozen Era 1.13.6 SOURCE fixtures; no runtime/native credit."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):
    def test_lossless_rows(self):
        expected = [(n, s) for n, s in enumerate((E/'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in audit.build().get('source_rows', [])], expected)

    def test_three_cvars_no_invented_defaults(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], ['nameplateCommentatorMaxDistance','specular','textureErrorColors'])
        self.assertTrue(all(i['direction']=='added' and i['section']=='cvars' and i['page_default'] is None for i in ledger['inventory']))
        self.assertEqual(ledger.get('signatures'), [])
        self.assertEqual(ledger.get('examples'), [])
        self.assertEqual(ledger.get('prose_defaults'), [])

    def test_headers_and_navigation(self):
        ledger = audit.build()
        self.assertEqual([h['label'] for h in ledger.get('headers', [])], ['Summary','CVars'])
        self.assertEqual([(h['count'],h['observed'],h['matches']) for h in ledger.get('inventory_headers', [])], [(3,3,True)])
        self.assertEqual(ledger.get('source_toc'),11306)
        self.assertEqual(ledger.get('navigation'),dict(patch='1.13.6',prev='1.13.5',next='1.13.7'))

    def test_prose_links_and_templates(self):
        ledger = audit.build()
        self.assertEqual(len(ledger.get('links', [])),3)
        self.assertTrue(all(not i['expanded'] for i in ledger['links']))
        self.assertEqual(len(ledger.get('templates', [])),4)
        self.assertEqual(ledger.get('summary'),'Patch 1.13.6 [[Naxxramas (Classic)|Naxxramas]] content patch.')
        self.assertEqual(len(ledger.get('contracts', [])),7)
        self.assertTrue(all(i['status']=='UNPROVEN' for i in ledger['contracts']))
        self.assertEqual(ledger.get('measurements'),dict(runtime=0,model=0,native=0))

    def test_profiles_and_separate_successors(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'],p['configured_interface']) for p in ledger.get('configured_profiles',[])],[('client-era',11507),('client-anniversary',11507)])
        expected=[('1.13.7','in-flight'),('1.14.0','queued'),('1.14.1','queued')]+[(f'1.14.{n}','integrated-not-applied') for n in range(2,5)]+[(f'1.15.{n}','integrated-not-applied') for n in range(10)]
        self.assertEqual([(p['patch'],p['state']) for p in ledger.get('successors',[])],expected)
        self.assertTrue(all(not p['applied'] and not p['native_proof'] for p in ledger['successors']))
        self.assertEqual(ledger.get('later_registers'),[])

    def test_every_omission_and_fabrication_rejected(self):
        original=audit.build()
        self.assertIn('contracts',original)
        controls=0
        for field in ['source_rows','inventory','headers','inventory_headers','links','templates','prose','contracts','configured_profiles','successors']:
            for index in range(len(original[field])):
                changed=copy.deepcopy(original);changed[field].pop(index)
                with self.assertRaises(AssertionError): audit.validate_ledger(changed)
                controls+=1
        print('omission controls:',controls)
        for field,value in [('measurements',dict(runtime=1,model=1,native=1)),('source_toc',11507),('examples',['invented']),('signatures',['invented']),('prose_defaults',['1']),('later_registers',['retail'])]:
            changed=copy.deepcopy(original);changed[field]=value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        changed=copy.deepcopy(original);changed['inventory'][0]['page_default']='60'
        with self.assertRaises(AssertionError): audit.validate_ledger(changed)

    def test_exact_response_identity_and_content(self):
        self.assertIn('source',audit.build())
        response=json.loads((E/'source-response.json').read_bytes())
        response['query']['pages']['461367']['revisions'][0]['revid']+=1
        with self.assertRaises(AssertionError): audit.validate_source(json.dumps(response).encode(),(E/'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError): audit.validate_source((E/'source-response.json').read_bytes(),(E/'source.wikitext').read_bytes()+b'\n')

if __name__=='__main__': unittest.main(verbosity=2)
