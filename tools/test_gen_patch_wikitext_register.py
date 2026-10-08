"""Behavior fixtures for raw consolidated inventories."""
import unittest

from gen_patch_wikitext_register import parse_section, split_sections


class InventoryTests(unittest.TestCase):
    def test_plain_command_label_marks_commands_and_resets_at_removed_column(self):
        lines = [(1, '| valign="top" | <div>'),
                 (2, ': [[CVar GameplayContext|GameplayContext]]'),
                 (3, ': Commands'),
                 (4, ': [[CVar GxFrameStats|GxFrameStats]]'),
                 (5, '</div>'),
                 (6, '| valign="top" | <div>'),
                 (7, ': [[CVar GamePadForceXInput|GamePadForceXInput]]')]
        entries, _ = parse_section('cvars', lines)
        self.assertEqual([(e['symbol'], e['direction'], e.get('kind')) for e in entries],
                         [('GameplayContext', 'added', None),
                          ('GxFrameStats', 'added', 'command'),
                          ('GamePadForceXInput', 'removed', None)])

    def test_span_cvar_defaults_are_opt_in_and_keep_literal_values(self):
        lines = [(1, '| valign="top" | <div>'),
                 (2, ': <span>[[CVar cameraFov|cameraFov]]</span>'
                     '<span style="display:none">Default: <code><span>90</span></code>, Scope: Account</span>')]
        entries, _ = parse_section('cvars', lines, capture_span_defaults=True)
        self.assertEqual(entries[0]['page_default'], '90')
        legacy, _ = parse_section('cvars', lines)
        self.assertNotIn('page_default', legacy[0])

    def test_inline_structures_do_not_attach_fields_to_last_changed_api(self):
        raw = ('==Global API==\n|}\n {{api|C_TransmogCollection.GetCategoryAppearances}}\n'
               '   + arg transmogLocation\nStructures\n ClubInfo\n   + crossFaction\n'
               '==Widgets==\n| valign="top" | <div>\n: {{api|Region:GetSourceLocation}}\n')
        buckets = split_sections(raw, separate_inline_structures=True)
        entries, _ = parse_section('global-api', buckets['global-api'])
        self.assertEqual([(e['symbol'], e['annotation']) for e in entries],
                         [('C_TransmogCollection.GetCategoryAppearances', '+ arg transmogLocation')])
        widgets, _ = parse_section('widgets', buckets['widgets'])
        self.assertEqual([e['symbol'] for e in widgets], ['Region:GetSourceLocation'])

    def test_html_cvar_defaults_and_unbolded_command_label(self):
        entries, counts = parse_section('cvars', [
            (1, '! Added <small>(2)</small>'),
            (2, '! Removed <small>(0)</small>'),
            (3, '| valign="top" | <div>'),
            (4, ': <span>[[CVar NotchedDisplayMode|NotchedDisplayMode]]</span>'
                '<span style="display:none">Default: <code><span class="apitype">1</span></code></span>'),
            (5, ': Commands'),
            (6, ': <span>[[CVar spectate|spectate]]</span><span style="display:none"><small>Spectate <CharName-Realm></small></span>'),
            (7, '</div>'),
            (8, '| valign="top" | <div>'),
            (9, ': RemovedCvar'),
        ])
        self.assertEqual([(e['symbol'], e.get('page_default'), e.get('kind'), e['direction'])
                          for e in entries], [
            ('NotchedDisplayMode', '1', None, 'added'),
            ('spectate', None, 'command', 'added'),
            ('RemovedCvar', None, None, 'removed'),
        ])
        self.assertEqual(counts[0]['parsed_count'], 2)

    def test_inline_plain_commands_do_not_turn_removed_cvars_into_commands(self):
        entries, counts = parse_section('cvars', [
            (1, '! Added <small>(3)</small>'),
            (2, '! Removed <small>(1)</small>'),
            (3, '| valign="top" | <div>'),
            (4, ': {{apitooltip|type=cvar|name=validateFrameXML|default=1}}'),
            (5, ": '''Commands'''"),
            (6, ': LogFps'),
            (7, ': WriteCustomizationOptions'),
            (8, '</div>'),
            (9, '| valign="top" | <div>'),
            (10, ': bspcache'),
        ])
        self.assertEqual([(e['symbol'], e['direction'], e.get('kind'), e['wikitext_line'])
                          for e in entries], [
            ('validateFrameXML', 'added', None, 4),
            ('LogFps', 'added', 'command', 6),
            ('WriteCustomizationOptions', 'added', 'command', 7),
            ('bspcache', 'removed', None, 10),
        ])
        self.assertEqual([c['parsed_count'] for c in counts], [3, 1])

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

    def test_changed_line_accounts_for_every_api_and_shared_annotation(self):
        entries, _ = parse_section('global-api', [
            (1, '|}'),
            (2, ' {{api|C_GossipInfo.SelectActiveQuest}}, {{api|C_GossipInfo.SelectAvailableQuest}}, {{api|C_GossipInfo.SelectOption}}'),
            (3, '   + arg 1: optionID'),
            (4, '   - arg 1: index'),
            (5, ' {{api|C_Item.GetItemGUID}}'),
            (6, '   + ret 1: itemGUID'),
        ], expand_shared_changes=True)
        self.assertEqual([(e['symbol'], e['wikitext_line'], e['annotation']) for e in entries], [
            ('C_GossipInfo.SelectActiveQuest', 2, '+ arg 1: optionID\n- arg 1: index'),
            ('C_GossipInfo.SelectAvailableQuest', 2, '+ arg 1: optionID\n- arg 1: index'),
            ('C_GossipInfo.SelectOption', 2, '+ arg 1: optionID\n- arg 1: index'),
            ('C_Item.GetItemGUID', 5, '+ ret 1: itemGUID'),
        ])

    def test_underscore_handler_link_keeps_owner_label(self):
        entries, _ = parse_section('widgets', [
            (1, '| valign="top" | <div>'),
            (2, ': [[UIHANDLER_OnModelCleared|ModelSceneActor OnModelCleared]]'),
        ])
        self.assertEqual([(e['symbol'], e.get('kind')) for e in entries],
                         [('ModelSceneActor OnModelCleared', 'widget-script')])

    def test_plain_scripts_label_is_opt_in_and_preserves_modelscene_handler(self):
        lines = [
            (1, '! Added <small>(1)</small>'),
            (2, '! Removed <small>(0)</small>'),
            (3, '| valign="top" | <div>'),
            (4, ': Scripts'),
            (5, ': [[UIHANDLER OnDressModel|ModelScene OnDressModel]]'),
        ]
        with self.assertRaisesRegex(ValueError, 'no symbol reference'):
            parse_section('widgets', lines)
        entries, counts = parse_section('widgets', lines, skip_plain_scripts_label=True)
        self.assertEqual([(e['symbol'], e.get('kind'), e['wikitext_line']) for e in entries],
                         [('ModelScene OnDressModel', 'widget-script', 5)])
        self.assertEqual([c['parsed_count'] for c in counts], [1, 0])

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
