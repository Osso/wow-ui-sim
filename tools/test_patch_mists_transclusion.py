"""Behavioral fixtures for the pinned 2013 caption tables and Browser handlers."""
import unittest
import gen_patch_wikitext_register as generator


class MistsTransclusionTests(unittest.TestCase):
    def test_caption_tables_and_bare_removal(self):
        raw = '''{| class="wikitable"
|+ Global API (5.2.0.16650 &rarr; 5.3.0.16965)
! style="width: 50%"| 1 new functions
! style="width: 50%"| 1 removed functions
| valign="top" | <div>
: {{api|GetLootSpecialization}}
</div>
| valign="top" | <div>
: PrepVoidStorageForTransmogrify
</div>
|}
'''
        rows, counts = generator.parse_mists_automated_diff(raw)
        self.assertEqual([(r['symbol'], r['direction']) for r in rows], [
            ('GetLootSpecialization', 'added'), ('PrepVoidStorageForTransmogrify', 'removed')])
        self.assertEqual([(r['header_count'], r['parsed_count']) for r in counts], [(1, 1), (1, 1)])
        self.assertEqual([r['wikitext_line'] for r in rows], [6, 9])

    def test_widget_handler_ownership_and_kind(self):
        raw = '''{| class="wikitable"
|+ Widget Handlers (5.2.0.16650 &rarr; 5.3.0.16965)
! style="width: 50%"| 2 new handlers
! style="width: 50%"| 0 removed handlers
| valign="top" | <div>
: {{api|t=wh|Browser OnEditFocusGained}}
: {{api|t=wh|Browser OnEscapePressed}}
</div>
| valign="top" | <div>
</div>
|}
'''
        rows, counts = generator.parse_mists_widget_handlers(raw)
        self.assertEqual([r['symbol'] for r in rows], ['Browser OnEditFocusGained', 'Browser OnEscapePressed'])
        self.assertTrue(all(r['kind'] == 'widget-script' and r['section'] == 'widgets' for r in rows))
        self.assertEqual([r['wikitext_line'] for r in rows], [6, 7])
        self.assertEqual([(r['header_count'], r['parsed_count']) for r in counts], [(2, 2), (0, 0)])
        self.assertEqual(generator.parse_mists_widget_handlers('|+ Widget API (x)\n'), ([], []))


if __name__ == '__main__':
    unittest.main()
