"""Literal historical 3.2.0 bullets; opt-in never corrects source uncertainty."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class WrathRetailBulletsTests(unittest.TestCase):
    def test_opt_in_retains_signatures_arrays_and_uncertain_removal(self):
        raw = (
            "== Saved Instances ==\n"
            "* ''updated'' {{api|GetSavedInstanceInfo(index)}} -- two bools\n"
            "== Quest Difficulty ==\n"
            "* ''removed'' {{api|GetDifficutlyColor(level)}} -- GetQuestDifficultyColor(level)\n"
            '* \'\'new\'\' "QuestDifficultyColors"array -- replacement\n'
            "== Unit Functions ==\n"
            "* ''updated'' or ''removed'' {{api|UnitIsPlusMob(unitID)}} -- uncertain\n"
            "== Quest API ==\n"
            "* ''undocumented'' {{api|GetAbandonedQuestName()}}\n"
            "== Casting Events ==\n"
            "* ''new'' {{api|t=e|UNIT_SPELLCAST_INTERRUPTIBLE}}<ref name=\"Iriel\" />\n"
            "== Notes ==\n"
            "* Unlabelled {{api|IgnoreMe}}\n"
        )
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            source = Path(directory) / 'page.wikitext'
            output = Path(directory) / 'register.json'
            source.write_text(raw)
            argv = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                    '3.2.0', str(source), '4812266', str(output)]
            subprocess.run(argv, cwd=ROOT, check=True)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            result = subprocess.run(argv + ['--wrath-retail-change-bullets', '--client-line', 'retail'],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_text())['entries']
            self.assertEqual([(r['symbol'], r['direction'], r['section'], r['wikitext_line']) for r in rows], [
                ('GetSavedInstanceInfo', 'changed', 'global-api', 2),
                ('GetDifficutlyColor', 'removed', 'global-api', 4),
                ('QuestDifficultyColors', 'added', 'global-api', 5),
                ('UnitIsPlusMob', 'changed', 'global-api', 7),
                ('GetAbandonedQuestName', 'changed', 'global-api', 9),
                ('UNIT_SPELLCAST_INTERRUPTIBLE', 'added', 'events', 11),
            ])
            self.assertEqual(rows[0]['source_signature'], 'GetSavedInstanceInfo(index)')
            self.assertEqual(rows[2]['kind'], 'table')
            self.assertEqual(rows[3]['source_change'], 'updated or removed')
            self.assertEqual(rows[4]['source_change'], 'undocumented')
            self.assertTrue(all(row['annotation'] for row in rows))
            subprocess.run(argv, cwd=ROOT, check=True)
            self.assertEqual(output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
