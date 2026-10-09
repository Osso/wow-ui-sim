"""Pinned 2010 retail headings and signatures, not Wrath Classic behavior."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'data/patch-api/sources/3.3.3-api-changes.wikitext'


class HistoricalHeadingsTests(unittest.TestCase):
    def test_opt_in_classifies_full_page_and_preserves_default_bytes(self):
        tool = ROOT / 'tools/gen_patch_wikitext_register.py'
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            output = Path(directory) / 'register.json'
            argv = [sys.executable, '-B', str(tool), '3.3.3', str(SOURCE),
                    '2531935', str(output)]
            subprocess.run(argv, check=True, cwd=ROOT)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            result = subprocess.run(argv + ['--historical-api-headings', '--client-line', 'retail'],
                                    capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stderr)
            register = json.loads(output.read_text())
            rows = register['entries']
            self.assertEqual(register['client_line'], 'retail')
            self.assertEqual(register['header_counts'], [])
            self.assertEqual(len(rows), 36)
            self.assertEqual(len({row['id'] for row in rows}), 36)
            self.assertEqual([(row['section'], row['direction']) for row in rows],
                             [('global-api', 'added')] * 21 + [('events', 'added')] * 9 +
                             [('global-api', 'changed')] * 3 + [('global-api', 'removed')] +
                             [('widgets', 'removed')] * 2)
            self.assertEqual([(row['symbol'], row['wikitext_line']) for row in rows[-6:]], [
                ('AcceptProposal', 38), ('GetLFGBootProposal', 39), ('UninviteUnit', 40),
                ('ShowBattlefieldList', 43), ('Texture:GetTexCoordModifiesRect', 44),
                ('Texture:SetTexCoordModifiesRect', 45)])
            self.assertEqual(rows[3]['annotation'],
                             '* isTrivial, isDaily, isRepeatable = {{api|GetAvailableQuestInfo}}(id)')
            subprocess.run(argv, check=True, cwd=ROOT)
            self.assertEqual(output.read_bytes(), original)

    def test_unrelated_sections_are_not_inventory(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            source = Path(directory) / 'source.wikitext'
            source.write_text('== New API functions ==\n* value = {{api|Read}}(id)\n'
                              '== Removed API ==\n* {{api|Texture:Old|t=w}}()\n'
                              '== References ==\n* {{api|NotInventory}}()\n')
            output = Path(directory) / 'register.json'
            result = subprocess.run([sys.executable, '-B',
                                     str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                                     '3.3.3', str(source), '2531935', str(output),
                                     '--historical-api-headings'], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([(row['symbol'], row['direction'])
                              for row in json.loads(output.read_text())['entries']],
                             [('Read', 'added'), ('Texture:Old', 'removed')])


if __name__ == '__main__':
    unittest.main()
