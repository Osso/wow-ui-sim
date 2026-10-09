"""Bounded frozen historical Retail 2.4.0 source behavior, not native parity."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/2.4.0-wikitext.txt'


class SourceTests(unittest.TestCase):
    def test_literal_occurrences_are_opt_in(self):
        import gen_patch_wikitext_register as generator
        parse = getattr(generator, 'parse_retail_240_summary', None)
        self.assertIsNotNone(parse, 'missing opt-in historical Retail 2.4.0 parser')
        entries = parse(RAW.read_text())
        self.assertEqual(len(entries), 49)
        self.assertEqual(len({row['id'] for row in entries}), 49)
        self.assertEqual([(r['symbol'], r['direction']) for r in entries[:5]], [
            ('PARTY_MEMBER_ENABLE', 'changed'), ('PARTY_MEMBER_DISABLE', 'changed'),
            ('debugstack', 'changed'), ('unitHighlights', 'added'),
            ('/console', 'changed')])
        self.assertEqual([r['symbol'] for r in entries if r['section'] == 'widgets'],
                         ['GameTooltip:SetTotem'])
        family = [r for r in entries if r['symbol'] == 'GetItemFamily']
        self.assertEqual([r['direction'] for r in family], ['added', 'changed'])
        info = next(r for r in entries if r['symbol'] == 'GetItemInfo')
        self.assertEqual(info['direction'], 'changed')
        self.assertIn('returns nil', info['annotation'])
        self.assertEqual(next(r for r in entries if r['symbol'] == 'SetGuildBankText')
                         ['wikitext_line'], 53)
        self.assertTrue(all(r['annotation'] for r in entries))
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            args = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                    '2.4.0', str(RAW), '6471380', str(output)]
            subprocess.run(args, cwd=ROOT, check=True, capture_output=True)
            original = output.read_bytes()
            self.assertEqual(json.loads(original)['entries'], [])
            subprocess.run(args + ['--retail-240-summary', '--client-line', 'retail'],
                           cwd=ROOT, check=True, capture_output=True)
            self.assertEqual(json.loads(output.read_text())['entries'], entries)
            subprocess.run(args, cwd=ROOT, check=True, capture_output=True)
            self.assertEqual(output.read_bytes(), original)

    def test_complete_extract_preserves_literal_texture_and_events(self):
        import extract_patch_non_inventory as extractor
        raw = RAW.read_text()
        default = extractor.extract_text(raw)
        try:
            text = extractor.extract_text(raw, retail_240_summary=True)
        except TypeError:
            self.fail('missing opt-in full historical extract')
        self.assertEqual(sum(bool(l.strip()) for l in text.splitlines()), 99)
        for literal in ('PARTY_MEMBER_ENABLE', 'PARTY_MEMBER_DISABLE',
                        '|T<path>:<width>[:<height>:<xOffset>:<yOffset>]|t',
                        '2048 = Unknown', '4096 = Vanity Pets', 'canEdit =',
                        'topicId=2968233433', 'UIOptionsPanels.lua', 'SetGuildBankText'):
            self.assertIn(literal, text)
        self.assertEqual(extractor.extract_text(raw), default)


if __name__ == '__main__':
    unittest.main()
