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

    def test_unnamed_removals_and_transclusion_not_invented(self):
        raw = ('==Changes==\n*Many legacy functions removed in C_MountJournal.\n'
               '**Note: GetCompanionInfo remains broken.\n'
               '==Removals==\n*AddOns\n**Reforging\n'
               '==Automated diff==\n{{:Patch 6.0.2/API changes/diff}}\n')
        self.assertEqual(generator.parse_warlords_prepatch(raw), [])


if __name__ == '__main__':
    unittest.main()
