"""Behavioral fixtures for compact Warlords prepatch summaries."""
import unittest
import gen_patch_wikitext_register as generator


class WarlordsRegisterTests(unittest.TestCase):
    def test_nested_references_and_canonical_widget_owners(self):
        raw = ('==New==\n*New Widget type: CinematicModel, derivative of PlayerModel\n'
               '*New atlas property: [[API Texture SetAtlas|Texture:SetAtlas]]\n'
               '*Timers {{api|C_Timer.After}}, {{api|C_Timer.NewTimer}}\n'
               '*Garrison stuff - C_Garrison.*\n'
               '==Changes==\n**{{api|CheckButton SetChecked|CheckButton:SetChecked}}()\n'
               '*New event {{api|t=e|LOOT_READY}}\n'
               '*{{api|GetTalentInfoByID}} returns {{api|GetTalentInfo}}\n')
        rows = generator.parse_warlords_prepatch(raw)
        self.assertEqual([(r['section'], r['direction'], r['symbol']) for r in rows], [
            ('widgets', 'added', 'CinematicModel'), ('widgets', 'added', 'Texture:SetAtlas'),
            ('global-api', 'added', 'C_Timer.After'), ('global-api', 'added', 'C_Timer.NewTimer'),
            ('global-api', 'added', 'C_Garrison'), ('widgets', 'changed', 'CheckButton:SetChecked'),
            ('events', 'changed', 'LOOT_READY'), ('global-api', 'changed', 'GetTalentInfoByID'),
            ('global-api', 'changed', 'GetTalentInfo')])
        self.assertEqual(len(rows), len({r['id'] for r in rows}))

    def test_diff_bare_removals_and_framexml_headers(self):
        raw = ('=== FrameXML ===\n{|\n|+ FrameXML (old &rarr; new)\n'
               '! | 1 new functions\n! | 1 removed functions\n'
               '| valign="top"\n: {{api|NewHelper}}\n</div>\n'
               '| valign="top"\n: OldHelper\n</div>\n|}\n'
               '=== Widget API ===\n{|\n|+ Widget API (old &rarr; new)\n'
               '! | 1 new methods\n! | 1 removed methods\n'
               '| valign="top"\n: {{api|t=w|Texture:SetAtlas}}\n</div>\n'
               '| valign="top"\n: Animation:GetProgressWithDelay\n</div>\n|}\n')
        rows, counts = generator.parse_warlords_diff(raw)
        self.assertEqual([(r['section'], r['direction'], r['symbol']) for r in rows], [
            ('framexml', 'added', 'NewHelper'), ('framexml', 'removed', 'OldHelper'),
            ('widgets', 'added', 'Texture:SetAtlas'),
            ('widgets', 'removed', 'Animation:GetProgressWithDelay')])
        self.assertTrue(all(c['header_count'] == c['parsed_count'] for c in counts))

    def test_unnamed_removals_and_transclusion_not_invented(self):
        raw = ('==Changes==\n*Many legacy functions removed in C_MountJournal.\n'
               '**Note: GetCompanionInfo remains broken.\n'
               '==Removals==\n*AddOns\n**Reforging\n'
               '==Automated diff==\n{{:Patch 6.0.2/API changes/diff}}\n')
        self.assertEqual(generator.parse_warlords_prepatch(raw), [])


if __name__ == '__main__':
    unittest.main()
