"""Frozen Retail 3.3.0 publication and lossless-summary contracts."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/3.3.0-wikitext.txt'


class SourceTests(unittest.TestCase):
    def test_literal_inventory_is_opt_in(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            command = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                       '3.3.0', str(RAW), '6055853', str(output)]
            default = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(default.returncode, 0, default.stderr)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            result = subprocess.run(command + ['--wrath-retail-summary', '--client-line', 'retail'],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_text())['entries']
            self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows], [
                ('QueryQuestsCompleted', 'added', 8), ('QUEST_QUERY_COMPLETE', 'added', 8),
                ('GetQuestsCompleted', 'added', 9), ('Frame:GetFrameType', 'removed', 15),
                ('Frame:GetObjectType', 'changed', 15), ('Region:IsMouseOver', 'added', 18),
                ('MouseIsOver', 'changed', 18), ('Button:SetMotionScriptsWhileDisabled', 'added', 21),
                ('Button:GetMotionScriptsWhileDisabled', 'added', 22),
                ('Texture:GetFileWidth', 'added', 27), ('Texture:GetFileHeight', 'added', 28)])
            self.assertEqual(len({r['id'] for r in rows}), len(rows))
            self.assertTrue(all(r['annotation'] for r in rows))
            subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
            self.assertEqual(output.read_bytes(), original)

    def test_summary_extraction_keeps_inline_xml_and_attribution(self):
        from extract_patch_non_inventory import extract_text
        raw = RAW.read_text()
        original = extract_text(raw)
        text = extract_text(raw, wrath_summary_markup=True)
        self.assertIn('Source attribution: Based on Iriel\'s post on the official forums.', text)
        self.assertIn('`<Button motionScriptsWhileDisabled="true">`', text)
        self.assertIn('registerForClicks', text)
        self.assertIn('0x1C0', text)
        self.assertIn('vehicleui', text)
        self.assertIn('unithasvehicleui', text)
        self.assertIn('Global API Changes: 3.2.2 -> 3.3.0', text)
        self.assertEqual(extract_text(raw), original)
        self.assertNotIn('<Button motionScriptsWhileDisabled', original)


if __name__ == '__main__':
    unittest.main()
