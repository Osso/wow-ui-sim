"""Literal frozen 2008 retail source, not TBC Classic."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/2.4.2-wikitext.txt'


class SourceTests(unittest.TestCase):
    def test_literal_inventory_is_opt_in(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            command = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                       '2.4.2', str(RAW), '803933', str(output)]
            original = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(original.returncode, 0, original.stderr)
            original_bytes = output.read_bytes()
            self.assertEqual(json.loads(original_bytes)['entries'], [])
            result = subprocess.run(command + ['--retail-tbc-summary', '--client-line', 'retail'],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_text())['entries']
            self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows], [
                ('AcceptLevelGrant', 'added', 5), ('CombatLog_Object_IsA', 'added', 6),
                ('DeclineLevelGrant', 'added', 7), ('GetCoinText', 'added', 8),
                ('GetQuestLogSpellLink', 'added', 9), ('GetQuestSpellLink', 'added', 10),
                ('strreplace', 'added', 11), ('GuildRosterSetOfficerNote', 'changed', 19),
                ('Minimap:PingLocation', 'changed', 20), ('CombatLogSetCurrentEntry', 'changed', 21),
                ('ScrollingMessageFrame:AddMessage', 'changed', 22)])
            self.assertEqual(len({r['id'] for r in rows}), len(rows))
            self.assertEqual([r['section'] for r in rows[-4:]],
                             ['global-api', 'widgets', 'global-api', 'widgets'])
            self.assertTrue(all(r['annotation'] for r in rows))
            subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
            self.assertEqual(output.read_bytes(), original_bytes)

    def test_extract_keeps_plural_placeholder_reference_and_limits(self):
        from extract_patch_non_inventory import extract_text
        text = extract_text(RAW.read_text(), retail_tbc_markup=True, lowercase_reflist=True)
        self.assertIn('<CONSTANT_NAME>_P1', text)
        self.assertIn('|4apple:apples;', text)
        self.assertIn('128 queries every 30 seconds (queued)', text)
        self.assertIn('15000+', text)
        self.assertIn('2008-05-14 06:08 PST', text)
        self.assertIn('Re: Minimap Ping Macros Disabled?', text)
        self.assertIn('ScrollingMessageFrame:AddMessage', text)
        self.assertIn('GetCoinText(amount, "separator")', text)
        self.assertNotIn('{{', text)


if __name__ == '__main__':
    unittest.main()
