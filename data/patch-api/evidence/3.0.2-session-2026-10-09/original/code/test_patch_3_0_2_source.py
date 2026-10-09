"""Literal frozen 3.0.2 source contracts; no native/runtime credit."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/3.0.2-wikitext.txt'


class SourceTests(unittest.TestCase):
    def parser(self, text):
        import gen_patch_wikitext_register as generator
        parser = getattr(generator, 'parse_wrath_launch_inventory', None)
        self.assertIsNotNone(parser, 'literal launch parser missing')
        return parser(text)

    def test_labeled_signatures_handlers_and_bare_unknowns(self):
        rows = self.parser('''== Actual API changes ==
* NEW - ?? = GetUnitPitch("unit")
 NEW - month, year = CalendarGetAbsMonth() ??
* MODIFIED - name, rank = UnitBuff("unit", [index])
* REMOVED - Button:GetTextFontObject() (replaced by GetNormalFontObject)
* NEW HANDLER - OnTooltipSetAchievement
* NEW - count = {{api|GetNumCompletedAchievements}}()
* NEW - UnitSelectionColor ??
''')
        self.assertEqual([(r['symbol'], r['section'], r['direction']) for r in rows], [
            ('GetUnitPitch', 'global-api', 'added'),
            ('CalendarGetAbsMonth', 'global-api', 'added'),
            ('UnitBuff', 'global-api', 'changed'),
            ('Button:GetTextFontObject', 'widgets', 'removed'),
            ('OnTooltipSetAchievement', 'widgets', 'added'),
            ('GetNumCompletedAchievements', 'global-api', 'added'),
            ('UnitSelectionColor', 'global-api', 'added')])
        self.assertEqual(rows[4]['kind'], 'widget-script')
        self.assertTrue(all(r['annotation'] for r in rows))

    def test_summary_occurrences_do_not_expand_domains(self):
        rows = self.parser('''== Settings and Preferences ==
* The following functions have been replaced with server-stored cvars: SetAutoLootDefault(), GetAutoLootDefault()
* Saving settings uses the "synchronizeSettings" cvar; UploadSettings() and DownloadSettings().
* There is a new FOCUSCAST click modifier (with no default value).
Finally, there's a new event COMPANION_UPDATE with argument type.
There are also two new events, UNIT_THREAT_LIST_UPDATE and UNIT_THREAT_SITUATION_UPDATE.
* InterfaceOptionsFrame_OpenToPage has been renamed InterfaceOptionsFrame_OpenToCategory !!!
* The various GetPlayerBuff functions have been removed.
''')
        self.assertEqual([r['symbol'] for r in rows], [
            'SetAutoLootDefault', 'GetAutoLootDefault', 'UploadSettings', 'DownloadSettings',
            'synchronizeSettings', 'FOCUSCAST', 'COMPANION_UPDATE',
            'UNIT_THREAT_LIST_UPDATE', 'UNIT_THREAT_SITUATION_UPDATE',
            'InterfaceOptionsFrame_OpenToPage', 'InterfaceOptionsFrame_OpenToCategory'])
        self.assertEqual(rows[0]['direction'], 'removed')
        self.assertEqual(rows[5]['kind'], 'click-modifier')

    def test_frozen_inventory_and_default_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            command = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                       '3.0.2', str(RAW), '4638841', str(output)]
            old = subprocess.run(command, cwd=ROOT, capture_output=True)
            self.assertEqual(old.returncode, 0, old.stderr)
            original = output.read_bytes()
            result = subprocess.run(command + ['--wrath-launch-inventory', '--client-line', 'retail'],
                                    cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_bytes())['entries']
            self.assertEqual(len(rows), 373)
            self.assertEqual(len({r['id'] for r in rows}), len(rows))
            import re
            labeled = [number for number, line in enumerate(RAW.read_text().splitlines(), 1)
                       if re.match(r'\s*\*?\s*(NEW|UPDATED|MODIFIED|REMOVED)\b', line)]
            self.assertEqual(len(labeled), 346)
            for number in labeled:
                self.assertEqual(sum(r['wikitext_line'] == number for r in rows), 1, number)
            self.assertTrue(any(r['symbol'] == 'CalendarGetAbsMonth' for r in rows))
            self.assertEqual(sum(r['symbol'] == 'Slider:Disable' for r in rows), 2)
            self.assertEqual(sum(r['symbol'] == 'GetCVarBool' for r in rows), 2)
            self.assertEqual(sum(r['symbol'] == 'COMPANION_UPDATE' for r in rows), 1)
            self.assertFalse(any(r['symbol'] == 'GetPlayerBuffFunctions' for r in rows))
            subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
            self.assertEqual(output.read_bytes(), original)

    def test_unrecognized_labeled_row_fails_explicitly(self):
        with self.assertRaisesRegex(ValueError, 'launch inventory'):
            self.parser('* NEW - ??')


if __name__ == '__main__':
    unittest.main(verbosity=2)
