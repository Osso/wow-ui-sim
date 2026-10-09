"""Own frozen 1.14.2 SOURCE fixtures, not runtime/native tests."""
import copy
import json
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent

class SourceAccounting(unittest.TestCase):
    def test_rows_lossless(self):
        expected = [(n, s) for n, s in enumerate((E/'source.wikitext').read_text().splitlines(), 1) if s.strip()]
        self.assertEqual([(r['line'], r['literal']) for r in audit.build().get('source_rows', [])], expected)

    def test_inventory_and_unspecified_signatures(self):
        ledger = audit.build()
        self.assertEqual([i['symbol'] for i in ledger.get('inventory', [])], ['C_GamePad.ClearLedColor', 'C_GamePad.GetLedColor', 'C_GamePad.SetLedColor', 'TradeSkillOnlyShowMakeable', 'GamePadFactionColor', 'GamePadVibrationStrength', 'telemetryTargetPackage'])
        self.assertEqual(len(ledger.get('signatures', [])), 4)
        self.assertTrue(all(s['arguments'] is None and s['returns'] is None for s in ledger['signatures']))

    def test_hidden_defaults_and_prose(self):
        self.assertEqual([(c['symbol'], c['default'], c['description']) for c in audit.build().get('cvars', [])], [('GamePadFactionColor','1',"Enable setting GamePad's led color to match current faction"),('GamePadVibrationStrength','1','GamePad vibration effect strength'),('telemetryTargetPackage','Blizzard.Telemetry.Wow_Mainline','The Package we want to send telemetry to e.g. Wow_Mainline or Wow_Classic')])

    def test_headers_caption_navigation(self):
        ledger = audit.build()
        self.assertEqual(ledger.get('caption'), '1.14.1 (41030) &rarr; 1.14.2 (42214) Feb 3 2022')
        self.assertEqual([(h['section'],h['direction'],h['count']) for h in ledger.get('inventory_headers', [])], [('Global API','added',4),('Global API','removed',0),('CVars','added',3),('CVars','removed',0)])
        self.assertEqual([h['label'] for h in ledger.get('headers',[])], ['Resources','Global API','CVars'])
        self.assertEqual(ledger.get('source_toc'),11402)
        self.assertEqual(ledger.get('navigation'),dict(patch='1.14.2',prev='1.14.1',next='1.14.3'))

    def test_links_examples_and_credit(self):
        ledger = audit.build()
        self.assertEqual(len(ledger.get('links',[])),7)
        self.assertEqual(ledger.get('examples'),[])
        self.assertEqual(ledger.get('measurements'),dict(runtime=0,model=0,native=0))
        self.assertTrue(all(c['status']=='UNPROVEN' for c in ledger['contracts']))

    def test_profiles_successors(self):
        ledger = audit.build()
        self.assertEqual([(p['feature'],p['configured_interface']) for p in ledger.get('configured_profiles',[])],[('client-era',11507),('client-anniversary',11507)])
        self.assertEqual([(p['patch'],p['state']) for p in ledger.get('successors',[])],[('1.14.3','in-flight'),('1.14.4','queued')]+[(f'1.15.{n}','integrated-not-applied') for n in range(10)])
        self.assertEqual(ledger.get('later_registers'),[])

    def test_every_omission_and_fabrication_rejected(self):
        original=audit.build()
        self.assertIn('contracts',original)
        for field in ['source_rows','inventory','signatures','cvars','headers','inventory_headers','links','contracts','configured_profiles','successors']:
            for index in range(len(original[field])):
                changed=copy.deepcopy(original);changed[field].pop(index)
                with self.assertRaises(AssertionError): audit.validate_ledger(changed)
        for field,value in [('measurements',dict(runtime=1,model=1,native=1)),('caption','Feb 3 2021'),('source_toc',11507),('examples',['invented']),('later_registers',['1.14.3'])]:
            changed=copy.deepcopy(original);changed[field]=value
            with self.assertRaises(AssertionError): audit.validate_ledger(changed)

    def test_response_identity_and_content_tamper(self):
        self.assertIn('source',audit.build())
        response=json.loads((E/'source-response.json').read_bytes())
        response['query']['pages']['236101']['revisions'][0]['revid']+=1
        with self.assertRaises(AssertionError): audit.validate_source(json.dumps(response).encode(),(E/'source.wikitext').read_bytes())
        with self.assertRaises(AssertionError): audit.validate_source((E/'source-response.json').read_bytes(),(E/'source.wikitext').read_bytes()+b'\n')

if __name__=='__main__': unittest.main(verbosity=2)
