"""Observable plaintext and source-ID contracts for retained patch pages."""
import subprocess
import sys
import unittest
from pathlib import Path

from extract_patch_non_inventory import extract_text, seed_rows

ROOT = Path(__file__).resolve().parent.parent


class ExtractTests(unittest.TestCase):
    def test_legacy_cvar_table_keeps_caption_not_inventory_markup(self):
        raw = ('==API==\n====New====\n* {{api|C_Test.Call}}\n==CVars==\n'
               '{| class="wikitable"\n|+ 8.0.1 to 8.1.0\n|-\n'
               '! Added (1)\n| valign="top" | <div>\n: {{api|t=c|Foo}}\n'
               '</div>\n|}\n==References==\n{{Reflist}}\n')
        self.assertEqual(extract_text(raw, legacy_api_bullets=True, legacy_cvar_tables=True),
                         '== API ==\n==== New ====\n== CVars ==\n'
                         '|+ 8.0.1 to 8.1.0\n== References ==\n'
                         '[References list; not expanded]\n')

    def test_legacy_bullets_strip_only_inventory_and_retain_unknown_sections(self):
        raw = ('Diff: 8.1.0 to 8.1.5\n==API==\n====New====\n'
               'New C_CVar table\n* {{api|C_CVar.GetCVar}}\n'
               '====Changes====\n?\n====Removals====\n* {{api|GetCVar}}\n'
               '==Widgets==\n?\n==Events==\n* {{api|t=e|NEW_TOY_ADDED}}\n'
               '==CVars==\n?\n')
        self.assertEqual(extract_text(raw, legacy_api_bullets=True),
                         'Diff: 8.1.0 to 8.1.5\n== API ==\n==== New ====\n'
                         'New C_CVar table\n==== Changes ====\n?\n'
                         '==== Removals ====\n== Widgets ==\n?\n== Events ==\n'
                         '== CVars ==\n?\n')
        self.assertNotEqual(extract_text(raw), extract_text(raw, legacy_api_bullets=True))
    def test_bfa_prepatch_retains_migrations_and_references_not_inventories(self):
        raw = ('{{apichanges|8.0.1|prev=7.3.2|next=8.1.0}}\n'
               '==New==\n* New {{api|C_Map}} table.\n**{{api|C_Map.GetMapInfo}}\n'
               '==Changes==\n* {{api|Old}} moved to {{api|C_New}}.\n'
               '==Removals==\n* {{api|OldMap}} removed. Use {{api|C_Map.NewMap}}.\n'
               '==Events==\n====Added====\nThese events were added or initially documented.\n'
               '* {{api|t=e|NEW_EVENT}}\n====Removed====\n* {{api|t=e|OLD_EVENT}}\n'
               '==See also==\n*{{ref web|url=https://example.test|title=Bfa}}\n')
        self.assertEqual(extract_text(raw, bfa_prepatch=True),
                         'Patch 8.0.1 API changes\n== Changes ==\n'
                         '* Old moved to C_New.\n== Removals ==\n'
                         '* OldMap removed. Use C_Map.NewMap.\n== Events ==\n'
                         '==== Added ====\nThese events were added or initially documented.\n'
                         '==== Removed ====\n== See also ==\n'
                         '*[Reference: url=https://example.test|title=Bfa]\n')

    def test_cli_self_test_runs_without_argument_conflict(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / 'tools/extract_patch_non_inventory.py'), '--self-test'],
            cwd=ROOT, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, 'PASS: extraction and row-ID behavioral fixtures\n')

    def test_legacy_api_caption_inventory_retains_prose_and_reference(self):
        raw = ('{{apichanges|8.2.0|prev=8.1.5|next=8.2.5}}\n'
               '* {{ref web|url=https://example.test|author=Kaivax|title=UI Changes}}\n'
               '==API==\n====Changes====\n* {{api|GetFileIDFromPath}} changed.\n'
               '{| class="wikitable"\n|+ Global API 8.1.5 to 8.2.0\n'
               '| valign="top" | <div>\n: {{api|IgnoreMe}}\n</div>\n|}\n'
               '==Widgets==\n{| class="wikitable"\n|+ Widget Handlers 8.1.5 to 8.2.0\n'
               ': [[UIHANDLER OnError|Checkout:OnError]]\n|}\n==References==\n')
        self.assertEqual(extract_text(raw, legacy_api_tables=True),
                         'Patch 8.2.0 API changes\n'
                         '* [Reference: url=https://example.test|author=Kaivax|title=UI Changes]\n'
                         '== API ==\n==== Changes ====\n* GetFileIDFromPath changed.\n'
                         '== Widgets ==\n== References ==\n')

    def test_reference_note_preserves_publication_prose_and_citation(self):
        raw = ('* The [[Auction House]] was revamped <ref>{{ref web|'
               'url=https://example.test/source|author=[[Kaivax]]|date=2019-10-07|'
               'title=Feedback – Auction House Revamp}}</ref>\n')
        with self.assertRaises(ValueError):
            extract_text(raw)
        self.assertEqual(extract_text(raw, retain_reference_notes=True),
                         '* The Auction House was revamped [Reference: '
                         'url=https://example.test/source|author=Kaivax|date=2019-10-07|'
                         'title=Feedback – Auction House Revamp]\n')

    def test_spaced_inventory_headings_retain_deprecated_table(self):
        raw = ('==Deprecated API==\n{| class="wikitable"\n'
               ': {{api|Old}} → {{api|C_New.Call}}\n|}\n'
               '== Global API ==\n{| class="wikitable"\n: {{api|IgnoreMe}}\n|}\n'
               '== Widgets ==\n: {{api|Frame:IgnoreMe}}\n'
               '== Events ==\n: {{api|t=e|IGNORE_ME}}\n'
               '== CVars ==\n: {{api|t=c|ignoreMe}}\n'
               '==References==\n{{Reflist}}\n')
        self.assertEqual(extract_text(raw), '== Deprecated API ==\n')
        self.assertEqual(extract_text(raw, normalize_inventory_headings=True),
                         '== Deprecated API ==\n{| class="wikitable"\n'
                         ': Old → C_New.Call\n== References ==\n'
                         '[References list; not expanded]\n')

    def test_multiline_ambox_preserves_security_warning(self):
        raw = ('==UnitPopup Changes==\n{{Ambox\n'
               '| image = [[Image:Icon.png]]\n| border = red\n'
               '| type = Secure Execution and Tainting\n| info = \n'
               '* Modifying menus will [[Secure Execution and Tainting|taint]].\n}}\n')
        self.assertEqual(extract_text(raw), '== UnitPopup Changes ==\n'
                         '[Warning: Secure Execution and Tainting]\n'
                         '* Modifying menus will taint.\n')

    def test_inline_structures_are_retained_without_widget_inventory(self):
        raw = ('==Global API==\n: {{api|IgnoreMe}}\n|}\n'
               'Structures\n ClubInfo\n   + crossFaction\n'
               '==Widgets==\n: {{api|Region:GetSourceLocation}}\n'
               '==CVars==\n: cameraFov\n==Enums==\n + Enum.ReportType\n')
        self.assertEqual(extract_text(raw), '== Structures ==\n ClubInfo\n'
                         '   + crossFaction\n== Enumerations ==\n + Enum.ReportType\n')

    def test_level_two_structures_end_inventory_before_type_changes(self):
        raw = ('==Global API==\n{| class="wikitable"\n: {{api|IgnoreMe}}\n|}\n'
               '==Structures==\n Enum.ItemGemColor\n   + Primordial\n'
               '==Type Changes==\n===Functions===\n {{api|C_AccountInfo.GetIDFromBattleNetAccountGUID}}\n'
               '   # arg 1: guid, Type: string -> WOWGUID\n'
               '===Events===\n {{api|t=e|ACHIEVEMENT_EARNED}}\n'
               '   # arg 1: achievementID, Type: number -> AchievementID\n')
        self.assertEqual(extract_text(raw), '== Structures ==\n Enum.ItemGemColor\n   + Primordial\n'
                         '== Type Changes ==\n=== Functions ===\n C_AccountInfo.GetIDFromBattleNetAccountGUID\n'
                         '   # arg 1: guid, Type: string -> WOWGUID\n'
                         '=== Events ===\n ACHIEVEMENT_EARNED\n'
                         '   # arg 1: achievementID, Type: number -> AchievementID\n')

    def test_layout_clear_template_is_editorial_and_keeps_examples(self):
        raw = ('==Settings API==\n* Canvas and vertical layouts.\n{{clrr}}\n'
               '<syntaxhighlight lang="lua">\nSettings.RegisterAddOnCategory(category)\n'
               '</syntaxhighlight>\n==Global API==\n: {{api|IgnoreMe}}\n'
               '==Enums==\n Enum.X\n')
        self.assertEqual(extract_text(raw, preserve_examples=True),
                         '== Settings API ==\n* Canvas and vertical layouts.\n\n'
                         '```lua\nSettings.RegisterAddOnCategory(category)\n```\n'
                         '== Enumerations ==\n Enum.X\n')

    def test_code_examples_preserve_xml_and_lua_literals_when_requested(self):
        raw = ('==Example==\n<syntaxhighlight lang="lua">\n'
               'Texture:SetTexture([[Interface\\Buttons\\White8x8]])\n'
               '</syntaxhighlight>\n<syntaxhighlight lang="xml">\n'
               '<VertexColor duration="2">\n'
               '  <StartColor r="1" g="0" b="0"/>\n'
               '</VertexColor>\n</syntaxhighlight>\n')
        self.assertEqual(extract_text(raw, preserve_examples=True),
                         '== Example ==\n```lua\n'
                         'Texture:SetTexture([[Interface\\Buttons\\White8x8]])\n'
                         '```\n```xml\n<VertexColor duration="2">\n'
                         '  <StartColor r="1" g="0" b="0"/>\n'
                         '</VertexColor>\n```\n')

    def test_reference_list_is_context_and_xml_example_is_preserved(self):
        raw = ('<syntaxhighlight lang="xml">\n'
               '<Texture file="example">\n'
               '    <TextureSliceMode mode="Tiled"/>\n'
               '</Texture>\n</syntaxhighlight>\n{{Reflist}}\n')
        self.assertEqual(extract_text(raw), '<Texture file="example">\n'
                         '    <TextureSliceMode mode="Tiled"/>\n'
                         '</Texture>\n[References list; not expanded]\n')

    def test_level_two_inventory_and_ping_key_preserve_prose_and_enums(self):
        raw = ('==Ping system==\nPress {{keypress|G}}.\n'
               '==Global API==\n'
               '{{api ambox|border=red|image=x|size=48|The listed changes are out of date.}}\n'
               '{| class="wikitable"\n: {{api|C_Ping.SendMacroPing}}\n|}\n'
               '==Widgets==\n{| class="wikitable"\n: {{api|EditBox:ResetInputMode}}\n|}\n'
               '==Enums==\n Enum.PingSubjectType\n   + OnMyWay = 3\n'
               '==References==\n{{Reflist}}\n')
        self.assertEqual(extract_text(raw, preserve_examples=True),
                         '== Ping system ==\nPress G.\n'
                         '== Enumerations ==\n Enum.PingSubjectType\n   + OnMyWay = 3\n'
                         '== References ==\n[References list; not expanded]\n')

    def test_11_0_2_settings_transclusion_and_transition_are_editorial(self):
        text = extract_text('{{:Settings_API}}\n'
                            ': 11.0.0 (55818) → 11.0.2 (56819) Sep 27 2024\n'
                            '* {{api|Settings.RegisterAddOnSetting}} reads variableTbl.\n')
        self.assertEqual(text.splitlines()[0],
                         '[Transcluded source: Settings_API; not expanded]')
        rows = seed_rows('Patch 11.0.2 API changes\n' + text, patch='11.0.2')
        self.assertEqual([row['status'] for row in rows],
                         ['metadata-only', 'metadata-only', 'metadata-only', 'audit-pending'])
        self.assertEqual(rows[-1]['source_id'], 'prose-undated-004')

    def test_11_0_5_build_transition_is_editorial_not_a_contract(self):
        rows = seed_rows('Patch 11.0.5 API changes\n'
                         ': 11.0.2 (56819) → 11.0.5 (57212) Oct 22 2024\n'
                         '=== Enumerations ===\n Enum.PowerType\n   + Happiness\n',
                         patch='11.0.5')
        self.assertEqual(rows[1]['source_id'], 'source-context-002')
        self.assertEqual(rows[1]['status'], 'metadata-only')
        self.assertEqual(rows[3]['status'], 'audit-pending')
        self.assertEqual(rows[4]['source_id'], 'enumerations-Enum-PowerType-005')

    def test_11_0_7_profiler_transclusion_is_retained_without_expansion(self):
        text = extract_text('==Addon Profiling API==\n'
                            '{{:Enum.AddOnProfilerMetric}}\n'
                            ': 11.0.5 (57212) → 11.0.7 (58238) Dec 19 2024\n')
        self.assertEqual(text, '== Addon Profiling API ==\n'
                         '[Transcluded source: Enum.AddOnProfilerMetric; not expanded]\n'
                         ': 11.0.5 (57212) → 11.0.7 (58238) Dec 19 2024\n')
        rows = seed_rows(text, patch='11.0.7')
        self.assertEqual(rows[1]['status'], 'audit-pending')
        self.assertEqual(rows[1]['source_id'], 'prose-undated-002')
        self.assertEqual(rows[2]['status'], 'metadata-only')

    def test_11_1_0_build_context_preserves_category_contract(self):
        rows = seed_rows('Patch 11.1.0 API changes\n'
                         '## Category-deDE: Dies ist ein Test\n'
                         ': 11.0.7 (58238) → 11.1.0 (59466) Feb 27 2025\n',
                         patch='11.1.0')
        self.assertEqual(rows[1]['status'], 'audit-pending')
        self.assertEqual(rows[2]['status'], 'metadata-only')

    def test_11_1_5_toc_example_preserves_symbolic_interface_and_directives(self):
        raw = ('==TOC format changes==\n{{#tag:syntaxhighlight|\n'
               '## Interface: {{API LatestInterface}}\n'
               '[Family]\\File.lua\n'
               'MainlineOnly.lua [AllowLoadGameType mainline]\n'
               '|lang="wowtoc"}}\n')
        self.assertEqual(extract_text(raw), '== TOC format changes ==\n'
                         '## Interface: [API LatestInterface]\n'
                         '[Family]\\File.lua\n'
                         'MainlineOnly.lua [AllowLoadGameType mainline]\n')

    def test_11_1_5_build_transition_is_context_but_toc_directive_is_pending(self):
        rows = seed_rows('Patch 11.1.5 API changes\n'
                         'MainlineOnly.lua [AllowLoadGameType mainline]\n'
                         ': 11.1.0 (59466) → 11.1.5 (61265) Jun 3 2025\n',
                         patch='11.1.5')
        self.assertEqual(rows[1]['status'], 'audit-pending')
        self.assertEqual(rows[2]['status'], 'metadata-only')
        self.assertEqual(rows[2]['source_id'], 'source-context-003')

    def test_level_two_inventory_preserves_enum_constant_and_structure_rows(self):
        raw = ('==Summary==\n* Range checks changed.\n'
               '==Global API==\n{| class="wikitable"\n: {{api|IgnoreMe}}\n|}\n'
               '==Widgets==\n: {{api|Frame:AbortDrag}}\n'
               '==Enums==\n Enum.PvPMatchState\n   + Waiting = 1\n'
               '==Constants==\n ContentTrackingConsts\n   + MaxTrackedAchievements = 10\n'
               '==Structures==\n BattlefieldRewards\n   + roleShortageBonus\n')
        self.assertEqual(extract_text(raw), '== Summary ==\n* Range checks changed.\n'
                         '== Enumerations ==\n Enum.PvPMatchState\n   + Waiting = 1\n'
                         '== Constants ==\n ContentTrackingConsts\n   + MaxTrackedAchievements = 10\n'
                         '== Structures ==\n BattlefieldRewards\n   + roleShortageBonus\n')

    def test_event_only_inventory_preserves_enum_and_structure_contracts(self):
        raw = ('==Summary==\n* Documentation moved.\n'
               '==Events==\n{| class="wikitable"\n: {{api|t=e|NEW_EVENT}}\n|}\n'
               ' {{api|t=e|CHANGED_EVENT}}\n   + auctionID\n'
               '==Enums==\n + Enum.GamePadPowerLevel\n'
               ' # Enum.ReportType\n   + PvPScoreboard\n'
               '==Structures==\n # ItemKeyInfo ({{api|C_AuctionHouse.GetItemKeyInfo}})\n'
               '   + itemID\n   + battlePetSpeciesID\n')
        self.assertEqual(extract_text(raw), '== Summary ==\n* Documentation moved.\n'
                         '== Enumerations ==\n + Enum.GamePadPowerLevel\n'
                         ' # Enum.ReportType\n   + PvPScoreboard\n'
                         '== Structures ==\n # ItemKeyInfo (C_AuctionHouse.GetItemKeyInfo)\n'
                         '   + itemID\n   + battlePetSpeciesID\n')
        rows = seed_rows(extract_text(raw), patch='9.2.7')
        self.assertEqual(len(rows), 10)
        self.assertEqual(sum(row['status'] == 'audit-pending' for row in rows), 7)

    def test_unheaded_inventory_stops_at_structures(self):
        raw = ('==Consolidated changes==\n: 11.2.5 → 11.2.7\n'
               '{| class="wikitable"\n: {{api|IgnoreMe}}\n|}\n'
               '===CVars===\n: {{apitooltip|name=IgnoreCVar}}\n'
               '===Commands===\n: {{apitooltip|name=IgnoreCommand}}\n'
               '===Structures===\n DifficultyInfo\n   + isUserSelectable\n')
        self.assertEqual(extract_text(raw), '== Consolidated changes ==\n'
                         ': 11.2.5 → 11.2.7\n=== Structures ===\n'
                         ' DifficultyInfo\n   + isUserSelectable\n')

    def test_headed_inventory_preserves_existing_structure_exclusion(self):
        raw = ('===Global API===\n: {{api|IgnoreMe}}\n'
               '===Structures===\n DifficultyInfo\n   + isUserSelectable\n')
        self.assertEqual(extract_text(raw), '\n')

    def test_level_two_structure_section_ends_inventory(self):
        raw = ('==Global API==\n: {{api|IgnoreMe}}\n'
               '==Structures==\n Enum.PowerType\n'
               '   + <font color="green">AlternateQuest</font> = 23\n'
               ' AreaPOIInfo\n   + 13: addPaddingAboveWidgets\n')
        self.assertEqual(extract_text(raw),
                         '== Structures ==\n Enum.PowerType\n'
                         '   + AlternateQuest = 23\n'
                         ' AreaPOIInfo\n   + 13: addPaddingAboveWidgets\n')

    def test_12_0_0_plaintext_stays_identical(self):
        sources = ROOT / 'data/patch-api/sources'
        self.assertEqual(
            extract_text((sources / '12.0.0-api-changes.wikitext').read_text()),
            (sources / '12.0.0-api-changes.txt').read_text())

    def test_nested_blue_post_contract_is_not_editorial(self):
        rows = seed_rows('== Blue posts ==\n=== 2026-03-21 ===\n'
                         '** For charge cooldowns, currentCharges < maxCharges.\n',
                         patch='12.0.1')
        self.assertEqual(rows[-1]['status'], 'audit-pending')
        self.assertEqual(rows[-1]['source_id'], 'prose-2026-03-21-003')

    def test_11_2_5_resource_links_and_build_transition_are_editorial(self):
        rows = seed_rows('Patch 11.2.5 API changes\n== Summary ==\n'
                         '* Item socketing APIs moved to C_ItemSocketInfo.\n'
                         '== Resources ==\n* Deprecated APIs:\n'
                         '** Deprecated_ItemSocketInfo.lua\n'
                         '== Consolidated changes ==\n'
                         ': 11.2.0 (62438) → 11.2.5 (63796) Oct 10 2025\n',
                         patch='11.2.5')
        self.assertEqual(rows[2]['status'], 'audit-pending')
        self.assertEqual(rows[5]['source_id'], 'source-context-006')
        self.assertEqual(rows[5]['status'], 'metadata-only')
        self.assertEqual(rows[7]['source_id'], 'source-context-008')
        self.assertEqual(rows[7]['status'], 'metadata-only')

    def test_11_2_0_resources_are_editorial_but_nested_contracts_are_pending(self):
        rows = seed_rows('Patch 11.2.0 API changes\n== Breaking changes ==\n'
                         '** Addons should use StaticPopup_ForEachShownDialog.\n'
                         '== Resources ==\n** Deprecated_ChatInfo.lua\n'
                         '** Blizzard_DeprecatedSpecialization\n'
                         '== Consolidated changes ==\n'
                         ': 11.1.7 (61559) → 11.2.0 (62438) Aug 5 2025\n',
                         patch='11.2.0')
        self.assertEqual(rows[2]['status'], 'audit-pending')
        for index in (4, 5, 7):
            self.assertEqual(rows[index]['status'], 'metadata-only')
            self.assertEqual(rows[index]['source_id'], f'source-context-{index + 1:03}')

    def test_11_1_7_summary_contracts_are_pending_but_build_transition_is_editorial(self):
        rows = seed_rows('Patch 11.1.7 API changes\n== Summary ==\n'
                         '* Added table.create(arraySizeHint[, nodeSizeHint]).\n'
                         '== Consolidated changes ==\n'
                         ': 11.1.5 (61265) → 11.1.7 (61559) Jun 17 2025\n',
                         patch='11.1.7')
        self.assertEqual(rows[2]['status'], 'audit-pending')
        self.assertEqual(rows[2]['source_id'], 'prose-undated-003')
        self.assertEqual(rows[4]['status'], 'metadata-only')
        self.assertEqual(rows[4]['source_id'], 'source-context-005')

    def test_comparison_operators_are_not_html_tags(self):
        self.assertEqual(
            extract_text('* <code>currentCharges < maxCharges and startTime > 0</code>\n'),
            '* currentCharges < maxCharges and startTime > 0\n')

    def test_blue_post_templates_preserve_api_contract_and_spell_ids(self):
        raw = ('==Blue posts==\n===2026-02-24===\n'
               '{{apisummary.header|Healer Buffs and HoTs}}\n'
               '{{text|blizz|Holy Paladin}}\n*53563 - Beacon of Light\n'
               '* {{api|t=w|Cooldown:SetCooldown|SetCooldown}} rejects secrets.\n'
               '* {{api.inline|format("%.1s", secretwrap("Jar Jar Binks"))}}\n'
               '* {{api.system|UnitAuras|C_UnitAuras}} is restricted.\n'
               '* {{tlygo|ActionButton_ApplyCooldown}} changed.\n'
               '===Global API===\n: {{api|IgnoreMe}}\n===Enums===\n Enum.X\n')
        text = extract_text(raw)
        self.assertEqual(text, '== Blue posts ==\n=== 2026-02-24 ===\n'
                         '=== Healer Buffs and HoTs ===\n=== Holy Paladin ===\n'
                         '*53563 - Beacon of Light\n'
                         '* Cooldown:SetCooldown rejects secrets.\n'
                         '* format("%.1s", secretwrap("Jar Jar Binks"))\n'
                         '* C_UnitAuras is restricted.\n'
                         '* ActionButton_ApplyCooldown changed.\n'
                         '=== Enumerations ===\n Enum.X\n')
        rows = seed_rows(text, patch='12.0.1')
        spell = next(row for row in rows if row['source_id'] == 'prose-2026-02-24-005')
        self.assertEqual(spell['status'], 'audit-pending')
        self.assertIn('p1201-extract-scout.md', spell['note'])


    def test_legacy_inline_reference_retains_hardware_contract(self):
        raw = '* {{api|SendChatMessage}} restricted.<ref>{{ref web|url=https://example.org|quote=CHANNEL protected}}</ref>\n'
        self.assertEqual(extract_text(raw, retain_reference_notes=True),
                         '* SendChatMessage restricted.[Reference: url=https://example.org|quote=CHANNEL protected]\n')
        with self.assertRaisesRegex(ValueError, 'unhandled template'):
            extract_text(raw)

if __name__ == '__main__':
    unittest.main()
