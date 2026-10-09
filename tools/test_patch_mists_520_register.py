"""2013 retail transclusion and bare widget-handler inventory behavior."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('generator', ROOT / 'tools/gen_patch_wikitext_register.py')
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)


class Mists520RegisterTest(unittest.TestCase):
    def test_bare_handler_owner_and_direction(self):
        raw = '''{| class="wikitable"
|+ Widget Handlers (5.1.0.16309 &rarr; 5.2.0.16650)
! style="width: 50%"| 1 new handlers
! style="width: 50%"| 1 removed handlers
| valign="top" |
: {{api|t=wh|PlayerModel OnAnimFinished}}
</div>
| valign="top" |
: Model OnAnimFinished
</div>
|}
'''
        rows, counts = GEN.parse_mists_bare_widget_handlers(raw)
        self.assertEqual([(r['symbol'], r['direction'], r['kind']) for r in rows], [
            ('PlayerModel OnAnimFinished', 'added', 'widget-script'),
            ('Model OnAnimFinished', 'removed', 'widget-script')])
        self.assertEqual([c['parsed_count'] for c in counts], [1, 1])
        self.assertEqual([r['wikitext_line'] for r in rows], [6, 9])

    def test_pinned_transclusion_counts(self):
        raw = (ROOT / 'data/patch-api/sources/5.2.0-api-changes-diff.wikitext').read_text()
        rows, counts = GEN.parse_mists_automated_diff(raw)
        handlers, handler_counts = GEN.parse_mists_bare_widget_handlers(raw)
        self.assertEqual(len(rows), 159)
        self.assertEqual(len(handlers), 4)
        self.assertTrue(all(c['header_count'] == c['parsed_count'] for c in counts + handler_counts))
        self.assertEqual(rows[0]['symbol'], 'C_MapBar.BarIsShown')
        self.assertIn(('GetRaidDifficulty', 'removed'), [(r['symbol'], r['direction']) for r in rows])


if __name__ == '__main__':
    unittest.main()
