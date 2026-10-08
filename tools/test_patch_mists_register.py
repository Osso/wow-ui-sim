"""Behavioral fixtures for the 2013 Mists automated-diff syntax."""
import unittest

import gen_patch_wikitext_register as generator
import extract_patch_non_inventory as extractor


class MistsDiffTests(unittest.TestCase):
    def test_inventory_keeps_bare_removals_widget_owners_and_counts(self):
        raw = ('== Changes ==\n*{{api|random}}() changed.\n'
               '{| class="wikitable"\n|+ Global API (5.4.1.17538 &rarr; 5.4.2.17688)\n'
               '! style="width: 50%;"| 1 new functions\n! style="width: 50%;"| 1 removed functions\n'
               '| valign="top" | <div>\n: {{api|fastrandom}}\n</div>\n'
               '| valign="top" | <div>\n: securerandom\n</div>\n|}\n'
               '{| class="wikitable"\n|+ Widget API (old &rarr; new)\n'
               '! style="padding: 0;"| 1 new methods\n'
               '| valign="top" | <div>\n: {{api|t=w|Slider:SetObeyStepOnDrag}}\n</div>\n|}\n')
        rows, counts = generator.parse_mists_automated_diff(raw)
        self.assertEqual([(r['section'], r['symbol'], r['direction']) for r in rows], [
            ('global-api', 'fastrandom', 'added'), ('global-api', 'securerandom', 'removed'),
            ('widgets', 'Slider:SetObeyStepOnDrag', 'added')])
        self.assertEqual([(c['header_count'], c['parsed_count']) for c in counts],
                         [(1, 1), (1, 1), (1, 1)])
        self.assertEqual(len(rows), len({r['id'] for r in rows}))

    def test_extract_keeps_prose_enum_values_and_framexml_heading_not_api_rows(self):
        raw = ('{{apichanges|5.4.2|prev=5.4.1|next=5.4.7}}\n== Changes ==\n'
               '*{{api|GetGuildRosterInfo}}() returns name-realm.\n'
               '=== FrameXML ===\n{| class="wikitable"\n|+ FrameXML (old &rarr; new)\n'
               '| valign="top" | <div>\n: {{api|RGBToColorCode}}\n</div>\n|}\n'
               '=== Lua Enums ===\n{| class="wikitable"\n|+ Lua Enums (old &rarr; new)\n'
               '! style="padding: 0;"| New\n| valign="top" | <div>\n'
               ': [[LE_AUTOCOMPLETE_PRIORITY]] (new)\n:: _OTHER = 1\n:: _FRIEND = 5\n</div>\n|}')
        rendered = extractor.extract_text(raw, mists_automated_diff=True)
        self.assertIn('GetGuildRosterInfo() returns name-realm.', rendered)
        self.assertIn('=== FrameXML ===', rendered)
        self.assertNotIn('RGBToColorCode', rendered)
        self.assertIn(': LE_AUTOCOMPLETE_PRIORITY (new)\n:: _OTHER = 1\n:: _FRIEND = 5', rendered)


if __name__ == '__main__':
    unittest.main()
