"""Pinned 2010 retail section lists, signatures and opt-in byte compatibility."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class LegacySectionListsTests(unittest.TestCase):
    def test_section_lists_keep_signatures_and_changed_prose_without_expanding_links(self):
        raw = ('== New API functions ==\n'
               ': accountId, name = [[API BNGetInfo|BNGetInfo]]()\n'
               ': [[API BNIsBlocked|BNIsBlocked]]\n'
               '== New FrameXML API ==\n'
               ': [[API BetterDate|BetterDate]](format, time)\n'
               '== New Events ==\n'
               ': {{api|BN_NEW_PRESENCE|t=e}}(presenceId, name)\n'
               '== API Changes ==\n'
               '* No inventory guaranteed for {{api|NotifyInspect}} requests.\n'
               '== Removed FrameXML API==\n'
               ': [[API ToggleCombatLog|ToggleCombatLog]]\n'
               '== Unrelated ==\n: [[API Ignore|Ignore]]\n')
        tool = ROOT / 'tools/gen_patch_wikitext_register.py'
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            source, output = Path(directory) / 'page.wikitext', Path(directory) / 'register.json'
            source.write_text(raw)
            argv = [sys.executable, '-B', str(tool), '3.3.5', str(source), '247986', str(output)]
            subprocess.run(argv, check=True, cwd=ROOT)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            result = subprocess.run(argv + ['--legacy-section-lists', '--client-line', 'retail'],
                                    capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stderr)
            register = json.loads(output.read_text())
            rows = register['entries']
            self.assertEqual([(r['section'], r['symbol'], r['direction'], r['wikitext_line']) for r in rows], [
                ('global-api', 'BNGetInfo', 'added', 2),
                ('global-api', 'BNIsBlocked', 'added', 3),
                ('framexml', 'BetterDate', 'added', 5),
                ('events', 'BN_NEW_PRESENCE', 'added', 7),
                ('global-api', 'NotifyInspect', 'changed', 9),
                ('framexml', 'ToggleCombatLog', 'removed', 11),
            ])
            self.assertEqual(rows[0]['signature'], '()')
            self.assertEqual(rows[0]['returns'], 'accountId, name')
            self.assertNotIn('signature', rows[1])
            self.assertEqual(rows[2]['signature'], '(format, time)')
            self.assertEqual(rows[3]['signature'], '(presenceId, name)')
            self.assertEqual(register['header_counts'], [])
            self.assertEqual(len(rows), len({r['id'] for r in rows}))
            subprocess.run(argv, check=True, cwd=ROOT)
            self.assertEqual(output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
