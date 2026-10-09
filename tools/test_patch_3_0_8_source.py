"""Literal historical 3.0.8 inventory; aliases and secure handles remain distinct."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'data/patch-api/sources/3.0.8-api-changes.wikitext'


class SourceTests(unittest.TestCase):
    def test_opt_in_literal_rows_and_default_compatibility(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            argv = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                    '3.0.8', str(SOURCE), '947144', str(output)]
            default = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(default.returncode, 0, default.stderr)
            original = output.read_bytes()
            result = subprocess.run(argv + ['--legacy-labeled-summaries', '--client-line', 'retail'],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            register = json.loads(output.read_text())
            rows = register['entries']
            self.assertEqual(len(rows), 56)
            self.assertEqual(len({r['id'] for r in rows}), 56)
            self.assertEqual(register['header_counts'], [])
            self.assertEqual([r['symbol'] for r in rows[:2]],
                             ['ScriptObject:HookScript', 'ScriptObject:SetScript'])
            self.assertEqual(sum(r['direction'] == 'changed' for r in rows), 20)
            self.assertEqual(sum(r['direction'] == 'removed' for r in rows), 12)
            by_name = {r['symbol']: r for r in rows}
            self.assertIn('CalenderEventInvite', by_name)  # Source spelling, not CalendarEventInvite.
            self.assertIn('Control:GetTIme', by_name)
            self.assertEqual(by_name['ScrollingMessageFrame:GetHyperlinksEnabled']['signature'],
                             'Frame:GetHyperlinksEnabled()')
            self.assertEqual(by_name['SimpleHTML:SetHyperlinksEnabled']['signature'],
                             'Frame:SetHyperlinksEnabled(enableFlag)')
            self.assertEqual(by_name['CalendarContextGetEventIndex']['returns'],
                             'monthOfs, eventDay, eventIndex')
            self.assertEqual(by_name['GetArenaTeamGdfInfo'].get('signature'), None)
            self.assertEqual(by_name['FrameHandle:GetRect']['kind'], 'secure-handle')
            self.assertEqual(by_name['SetUpAnimation']['kind'], 'secure-control')
            restored = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(restored.returncode, 0, restored.stderr)
            self.assertEqual(output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
