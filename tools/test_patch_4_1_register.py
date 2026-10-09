"""4.1.0 source-format fixtures; no network or runtime parity claim."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class CataclysmBulletsTests(unittest.TestCase):
    def test_opt_in_preserves_nested_additions_removals_and_changed_events(self):
        tool = Path(__file__).with_name('gen_patch_wikitext_register.py')
        raw = ('== Breaking changes ==\n'
               '* [[API COMBAT_LOG_EVENT|COMBAT_LOG_EVENT]] and COMBAT_LOG_EVENT_UNFILTERED have a new third parameter, hideCaster.\n'
               '* Register {{api|SendAddonMessage}} using {{api| RegisterAddonMessagePrefix}}.\n'
               '== API changes ==\n'
               '* REMOVED {{api|CanTransform}} / {{api|Transform}}; spell now.\n'
               '* NEW calendar extensions\n** {{api| GetEventTime}}\n'
               '* NEW {{api|CancelEmote}}\n'
               '== Event changes ==\n'
               '* REMOVED END_REFUND, no longer fires.\n'
               '* [[API COMBAT_LOG_EVENT|COMBAT_LOG_EVENT]] and COMBAT_LOG_EVENT_UNFILTERED shift.\n'
               '== Unrelated ==\n** {{api|IgnoreMe}}\n')
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'page.wikitext'
            output = Path(directory) / 'register.json'
            source.write_text(raw)
            argv = [sys.executable, '-B', str(tool), '4.1.0', str(source), '4094671', str(output)]
            subprocess.run(argv, check=True)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            result = subprocess.run(argv + ['--cataclysm-change-bullets'], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_text())['entries']
            self.assertEqual([(r['section'], r['symbol'], r['direction'], r['wikitext_line']) for r in rows], [
                ('events', 'COMBAT_LOG_EVENT', 'changed', 2),
                ('events', 'COMBAT_LOG_EVENT_UNFILTERED', 'changed', 2),
                ('global-api', 'SendAddonMessage', 'changed', 3),
                ('global-api', 'RegisterAddonMessagePrefix', 'changed', 3),
                ('global-api', 'CanTransform', 'removed', 5),
                ('global-api', 'Transform', 'removed', 5),
                ('global-api', 'GetEventTime', 'added', 7),
                ('global-api', 'CancelEmote', 'added', 8),
                ('events', 'END_REFUND', 'removed', 10),
                ('events', 'COMBAT_LOG_EVENT', 'changed', 11),
                ('events', 'COMBAT_LOG_EVENT_UNFILTERED', 'changed', 11),
            ])
            self.assertEqual(len(rows), len({r['id'] for r in rows}))
            self.assertTrue(all(r['annotation'] for r in rows))
            subprocess.run(argv, check=True)
            self.assertEqual(output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
