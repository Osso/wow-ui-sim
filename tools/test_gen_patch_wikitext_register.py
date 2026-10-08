"""Behavior fixtures for raw consolidated inventories."""
import unittest

from gen_patch_wikitext_register import parse_section, split_sections


class InventoryTests(unittest.TestCase):
    def test_legacy_summary_tables_keep_names_not_prose_or_addons(self):
        import gen_patch_wikitext_register as generator
        raw = ('==New==\n* New AddOns: Blizzard_Console\n'
               '* New global table: SOUNDKIT - Keys hold soundkit IDs\n'
               '* New API tables: C_ArtifactRelicForgeUI, C_Console\n'
               '* New debug tool: Table Inspector - /tinspect\n'
               '==Changes==\n* New API tables: NotAnAddition\n')
        rows = generator.parse_legacy_summary_tables(raw)
        self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows],
                         [('SOUNDKIT', 'added', 3),
                          ('C_ArtifactRelicForgeUI', 'added', 4),
                          ('C_Console', 'added', 4)])
        self.assertEqual(len({r['id'] for r in rows}), 3)

    def test_top_level_bullets_keep_canonical_names_and_rename_directions(self):
        import gen_patch_wikitext_register as generator
        raw = ('== New ==\n* {{api|t=n|C_Bubbles}} added with '
               '{{api|C_Bubbles.GetAll|GetAll}}()\n'
               '== Changes ==\n* {{api|t=n|C_Trees}} now includes '
               '{{api|C_Trees.GetID|GetID}}() and {{api|C_Trees.GetIDs}}()\n'
               '* {{api|Old}}() renamed to {{api|C_New.Reload}}()\n'
               '== References ==\n* {{api|Ignore}}\n')
        rows = generator.parse_top_level_api_bullets(raw)
        self.assertEqual([(r['symbol'], r['direction']) for r in rows], [
            ('C_Bubbles', 'added'), ('C_Bubbles.GetAll', 'added'),
            ('C_Trees', 'changed'), ('C_Trees.GetID', 'added'),
            ('C_Trees.GetIDs', 'added'), ('Old', 'removed'), ('C_New.Reload', 'added')])
        self.assertEqual(len(rows), len({r['id'] for r in rows}))
        self.assertEqual(generator.split_sections(raw), {})

    def test_bfa_prepatch_nested_namespaces_events_and_removal_successors(self):
        import gen_patch_wikitext_register as generator
        raw = ('==New==\n* New {{api|C_Map}} table.\n**{{api|C_Map.GetMapInfo}}\n'
               '==Changes==\n* {{api|Old}} moved to {{api|C_New}}.\n'
               '==Removals==\n* {{api|OldMap}} removed. Use {{api|C_Map.NewMap}}.\n'
               '* FindSpellOverrideNameByName, FindBaseSpellNameByName, and SearchGuildRecipes have been removed.\n'
               '* [[GLYPH_ADDED]], [[GLYPH_REMOVED]] have been removed.\n'
               '==Events==\n====Added====\n* {{api|t=e|NEW_EVENT}}\n'
               '====Removed====\n* {{api|t=e|OLD_EVENT}}\n==See also==\n')
        entries = generator.parse_bfa_prepatch(raw)
        self.assertEqual([(e['section'], e['symbol'], e['direction']) for e in entries], [
            ('global-api', 'C_Map', 'added'), ('global-api', 'C_Map.GetMapInfo', 'added'),
            ('global-api', 'OldMap', 'removed'),
            ('global-api', 'FindSpellOverrideNameByName', 'removed'),
            ('global-api', 'FindBaseSpellNameByName', 'removed'),
            ('global-api', 'SearchGuildRecipes', 'removed'),
            ('events', 'GLYPH_ADDED', 'removed'), ('events', 'GLYPH_REMOVED', 'removed'),
            ('events', 'NEW_EVENT', 'added'), ('events', 'OLD_EVENT', 'removed')])
        self.assertEqual(len(entries), len({e['id'] for e in entries}))

    def test_prose_api_links_retain_each_changed_identity_and_literal(self):
        import gen_patch_wikitext_register as generator
        statement = '*[[API Logout|Logout]] and [[API Quit|Quit]] lua functions are now protected.'
        raw = ('==Changes==\n' + statement + '\n==References==\n'
               '* [[API Ignore|Ignore]]\n')
        rows = generator.parse_prose_api_links(raw)
        self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line'], r['annotation'])
                          for r in rows], [('Logout', 'changed', 2, statement),
                                          ('Quit', 'changed', 2, statement)])
        self.assertEqual(len({r['id'] for r in rows}), 2)
        self.assertEqual(generator.split_sections(raw), {})

    def test_legacy_caption_tables_keep_handlers_commands_and_directions(self):
        import gen_patch_wikitext_register as generator
        raw = ('==API==\n====Changes====\n* {{api|NotInventory}}\n'
               '{| class="wikitable"\n|+ Global API 8.1.5 to 8.2.0\n'
               '! style="width:50%"| 1 new functions\n'
               '! style="width:50%"| 1 removed functions\n'
               '| valign="top" | <div>\n: {{api|NewGlobal}}\n</div>\n'
               '| valign="top" | <div>\n: DEPRECATED {{api|OldGlobal}}\n</div>\n|}\n'
               '==Widgets==\n{| class="wikitable"\n|+ Widget Handlers 8.1.5 to 8.2.0\n'
               '! style="padding:0"| 1 new handlers\n'
               '| valign="top" | <div>\n: [[UIHANDLER  OnError|Checkout:OnError]]\n</div>\n|}\n'
               '==CVars==\n{| class="wikitable"\n|+ Console commands 8.1.5 to 8.2.0\n'
               '! style="width:50%"| 1 new commands\n'
               '| valign="top" | <div>\n: {{api|t=c|UpdateWindow}}\n</div>\n|}\n')
        entries, counts = generator.parse_legacy_caption_tables(raw)
        self.assertEqual([(e['symbol'], e['direction'], e.get('kind')) for e in entries],
                         [('NewGlobal', 'added', None), ('OldGlobal', 'removed', None),
                          ('Checkout:OnError', 'added', 'widget-script'),
                          ('UpdateWindow', 'added', 'command')])
        self.assertTrue(all(c['header_count'] == c['parsed_count'] for c in counts))
        self.assertEqual([c['section'] for c in counts],
                         ['global-api', 'global-api', 'widgets', 'commands'])

    def test_legacy_renames_retain_both_occurrences(self):
        import gen_patch_wikitext_register as generator
        raw = ('==API==\n====New====\n* {{api|Ignore}}\n'
               '====Renamed====\n* {{api|C_Calendar.EventGetClubID}} to '
               '{{api|C_Calendar.EventGetClubId}}\n====Removals====\n')
        rows = generator.parse_legacy_api_renames(raw)
        self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows],
                         [('C_Calendar.EventGetClubID', 'removed', 5),
                          ('C_Calendar.EventGetClubId', 'added', 5)])
        self.assertEqual(len({r['id'] for r in rows}), 2)

    def test_diff_additions_retain_each_late_build_publication(self):
        import gen_patch_wikitext_register as generator
        raw = ('==Changes==\n* New functions: {{api|Ignore}}\n'
               '==Diffs==\n===8.3.0 (34601)===\n'
               '* New functions: {{api|C_AzeriteEssence.GetNumUsableEssences}}, '
               '{{api|C_Item.IsItemCorruptable}}, {{api|TargetSpellHasApplyCorruption}}\n'
               '* New event: {{api|t=e|LFG_GROUP_DELISTED_LEADERSHIP_CHANGE}}\n'
               '==Global API==\n* New functions: {{api|IgnoreAgain}}\n')
        entries = generator.parse_diff_api_additions(raw)
        self.assertEqual([(e['section'], e['symbol'], e['direction'], e['wikitext_line'])
                          for e in entries], [
            ('global-api', 'C_AzeriteEssence.GetNumUsableEssences', 'added', 5),
            ('global-api', 'C_Item.IsItemCorruptable', 'added', 5),
            ('global-api', 'TargetSpellHasApplyCorruption', 'added', 5),
            ('events', 'LFG_GROUP_DELISTED_LEADERSHIP_CHANGE', 'added', 6),
        ])
        self.assertEqual(len({e['id'] for e in entries}), 4)

    def test_legacy_headers_retain_counts_and_removed_command_kind(self):
        lines = [(1, '! style="width:50%"| 1 new cvars'),
                 (2, '! style="width:50%"| 1 removed command'),
                 (3, '| valign="top" | <div>'),
                 (4, ': {{api|t=c|Collision}}'),
                 (5, '</div>'),
                 (6, '| valign="top" | <div>'),
                 (7, ': {{api|t=c|DumpSoundKits}}')]
        entries, counts = parse_section('cvars', lines, legacy_column_headers=True)
        self.assertEqual([(e['symbol'], e.get('kind')) for e in entries],
                         [('Collision', None), ('DumpSoundKits', 'command')])
        self.assertEqual([(c['direction'], c['header_count'], c['parsed_count']) for c in counts],
                         [('added', 1, 1), ('removed', 1, 1)])
        legacy, old_counts = parse_section('cvars', lines)
        self.assertNotIn('kind', legacy[1])
        self.assertEqual(old_counts, [])

    def test_legacy_bullets_preserve_new_removals_and_event_occurrences(self):
        import gen_patch_wikitext_register as generator
        raw = ('==API==\n====New====\nNew C_CVar table\n'
               '* {{api|C_CVar.GetCVar}}\n====Changes====\n?\n'
               '====Removals====\n* {{api|GetCVar}}\n'
               '==Widgets==\n?\n==Events==\n* {{api|t=e|NEW_TOY_ADDED}}\n'
               '==CVars==\n?\n')
        entries = generator.parse_legacy_api_bullets(raw)
        self.assertEqual([(e['section'], e['symbol'], e['direction'], e['wikitext_line'])
                          for e in entries], [
            ('global-api', 'C_CVar.GetCVar', 'added', 4),
            ('global-api', 'GetCVar', 'removed', 8),
            ('events', 'NEW_TOY_ADDED', 'added', 12),
        ])
        self.assertEqual(generator.parse_simple_api_list(raw), [])

    def test_simple_api_list_retains_function_and_cvar_source_lines(self):
        import gen_patch_wikitext_register as generator
        raw = ('== Changes ==\n* TOC: <code>80300</code>\n'
               '== API ==\n=== New ===\n* {{api|GetAreaText}}()\n'
               '* CVar {{api|t=c|MouseUseLazyRepositioning}}\n')
        entries = generator.parse_simple_api_list(raw)
        self.assertEqual([(e['section'], e['symbol'], e['direction'], e['wikitext_line'])
                          for e in entries], [
            ('global-api', 'GetAreaText', 'added', 5),
            ('cvars', 'MouseUseLazyRepositioning', 'added', 6),
        ])
        self.assertEqual(split_sections(raw), {})

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


    def test_legacy_mixed_cvars_retain_labels_and_count_each_kind(self):
        lines = [(1, '! style="padding: 0 1em;"| 4 new cvars, 1 new command'),
                 (2, '| valign="top" | <div>'), (3, ': CVar'),
                 (4, ': {{api|t=c|AIProcessDebugger}}'),
                 (5, ': {{api|t=c|closedInfoFramesAccountWide}}'),
                 (6, ': {{api|t=c|mountJournalShowPlayer}}'),
                 (7, ': {{api|t=c|PhaseHistory}}'), (8, ': Command'),
                 (9, ': {{api|t=c|DefragmentGPU}}')]
        entries, counts = parse_section('cvars', lines, legacy_inventory_labels=True)
        self.assertEqual([(e['symbol'], e.get('kind')) for e in entries], [
            ('AIProcessDebugger', None), ('closedInfoFramesAccountWide', None),
            ('mountJournalShowPlayer', None), ('PhaseHistory', None),
            ('DefragmentGPU', 'command')])
        self.assertEqual(counts, [
            {'section': 'cvars', 'direction': 'added', 'header_count': 4, 'parsed_count': 4},
            {'section': 'commands', 'direction': 'added', 'header_count': 1, 'parsed_count': 1}])

if __name__ == "__main__":
    unittest.main()
