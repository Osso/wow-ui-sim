"""Observable plaintext and source-ID contracts for retained patch pages."""
import unittest
from pathlib import Path

from extract_patch_non_inventory import extract_text, seed_rows

ROOT = Path(__file__).resolve().parent.parent


class ExtractTests(unittest.TestCase):
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


if __name__ == '__main__':
    unittest.main()
