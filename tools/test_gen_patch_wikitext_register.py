"""Behavior fixtures for raw consolidated inventories."""
import unittest

from gen_patch_wikitext_register import parse_section, split_sections


class InventoryTests(unittest.TestCase):
    def test_type_change_events_do_not_replace_publication_inventory(self):
        raw = ('==Events==\n! Added <small>(1)</small>\n'
               '| valign="top" | <div>\n: {{api|t=e|CAN_PLAYER_SPEAK_LANGUAGE_CHANGED}}\n'
               '</div>\n|}\n==Structures==\n RafInfo\n'
               '==Type Changes==\n===Functions===\n {{api|C_AccountInfo.GetIDFromBattleNetAccountGUID}}\n'
               '   # arg 1: guid, Type: string -> WOWGUID\n'
               '===Events===\n {{api|t=e|ACHIEVEMENT_EARNED}}\n'
               '   # arg 1: achievementID, Type: number -> AchievementID\n')
        entries, counts = parse_section('events', split_sections(raw)['events'])
        self.assertEqual([(e['symbol'], e['direction']) for e in entries],
                         [('CAN_PLAYER_SPEAK_LANGUAGE_CHANGED', 'added')])
        self.assertEqual(counts[0]['parsed_count'], 1)

    def test_level_two_inventory_headings_preserve_source_lines_and_count_drift(self):
        raw = ('==Global API==\n'
               '! Added <small>(19)</small>\n! Removed <small>(0)</small>\n'
               '| valign="top" | <div>\n: {{api|C_Ping.SendMacroPing}}\n</div>\n|}\n'
               '==Widgets==\n| valign="top" | <div>\n'
               ': {{api|t=w|EditBox:ResetInputMode}}\n</div>\n|}\n'
               '==Enums==\n Enum.PingSubjectType\n')
        buckets = split_sections(raw)
        entries, counts = parse_section('global-api', buckets.get('global-api', []))
        self.assertEqual([(e['symbol'], e['wikitext_line']) for e in entries],
                         [('C_Ping.SendMacroPing', 5)])
        self.assertEqual(counts[0]['header_count'], 19)
        self.assertEqual(counts[0]['parsed_count'], 1)
        widgets, _ = parse_section('widgets', buckets.get('widgets', []))
        self.assertEqual([e['symbol'] for e in widgets], ['EditBox:ResetInputMode'])

    def test_unheaded_global_inventory_and_separate_commands(self):
        raw = ('==Consolidated changes==\n'
               '{| class="wikitable"\n'
               '| valign="top" | <div>\n'
               ': {{api|C_ActionBar.IsInterruptAction}}\n'
               '</div>\n|}\n'
               '===Commands===\n'
               '| valign="top" | <div>\n'
               ': {{apitooltip|type=command|name=NeighborhoodAddManager}}\n'
               '</div>\n|}\n===Structures===\n DifficultyInfo\n')
        buckets = split_sections(raw)
        entries, _ = parse_section('global-api', buckets.get('global-api', []))
        self.assertEqual([e['symbol'] for e in entries], ['C_ActionBar.IsInterruptAction'])
        entries, _ = parse_section('cvars', buckets.get('commands', []))
        self.assertEqual([(e['symbol'], e.get('kind')) for e in entries],
                         [('NeighborhoodAddManager', 'command')])

    def test_level_two_inventory_headings(self):
        raw = ('==Global API==\n| valign="top" | <div>\n'
               ': {{api|C_ActionBar.EnableActionRangeCheck}}\n</div>\n|}\n'
               ' {{api|C_PvP.GetArenaRewards}}\n   + ret 5: roleShortageBonus\n'
               '==Widgets==\n| valign="top" | <div>\n'
               ': {{api|Frame:AbortDrag}}\n</div>\n|}\n'
               '==Enums==\n Enum.X\n   + New = 1\n')
        buckets = split_sections(raw)
        entries, _ = parse_section('global-api', buckets.get('global-api', []))
        self.assertEqual([(e['symbol'], e['direction'], e['annotation']) for e in entries], [
            ('C_ActionBar.EnableActionRangeCheck', 'added', ''),
            ('C_PvP.GetArenaRewards', 'changed', '+ ret 5: roleShortageBonus')])
        entries, _ = parse_section('widgets', buckets.get('widgets', []))
        self.assertEqual([e['symbol'] for e in entries], ['Frame:AbortDrag'])

    def test_changed_entries_accept_multiple_indentation_spaces(self):
        entries, _ = parse_section('global-api', [
            (1, '|}'),
            (2, '  {{api|t=a|C_PlayerInfo.GetSex}}'),
            (3, '   # ret 1: sex, Type: number -> UnitSex'),
        ])
        self.assertEqual([(e['symbol'], e['annotation']) for e in entries], [
            ('C_PlayerInfo.GetSex', '# ret 1: sex, Type: number -> UnitSex')])

    def test_widget_script_label_is_not_an_inventory_member(self):
        entries, _ = parse_section('widgets', [
            (1, '| valign="top" | <div>'),
            (2, ': Widget Scripts'),
            (3, ': [[UIHANDLER OnMovieHideSubtitle|OnMovieHideSubtitle]]'),
        ])
        self.assertEqual([(e['symbol'], e.get('kind')) for e in entries],
                         [('OnMovieHideSubtitle', 'widget-script')])

    def test_plain_removed_cvar_preserves_identity_and_source_line(self):
        entries, counts = parse_section('cvars', [
            (1, '! Added <small>(0)</small>'),
            (2, '! Removed <small>(1)</small>'),
            (3, '| valign="top" | <div>'),
            (4, '</div>'),
            (5, '| valign="top" | <div>'),
            (6, ': professionGearSlotsExampleShown'),
        ])
        self.assertEqual([(e['symbol'], e['direction'], e['wikitext_line'])
                          for e in entries],
                         [('professionGearSlotsExampleShown', 'removed', 6)])
        self.assertEqual(counts[1]['parsed_count'], 1)

    def test_uncollapsed_added_scriptobjects(self):
        entries, counts = parse_section("scriptobjects", [
            (1, '{| class="wikitable"'),
            (2, '| <font color="lightgreen">Added</font>'),
            (3, ': [[ScriptObject CurveObject|CurveObject]]'),
            (4, ': [[ScriptObject DurationObject|DurationObject]]'),
            (5, '|}'),
        ])
        self.assertEqual([(e["symbol"], e["direction"]) for e in entries], [
            ("CurveObject", "added"), ("DurationObject", "added")])
        self.assertEqual(counts, [])


if __name__ == "__main__":
    unittest.main()
