"""Observable plaintext and source-ID contracts for retained patch pages."""
import unittest
from pathlib import Path

from extract_patch_non_inventory import extract_text, seed_rows

ROOT = Path(__file__).resolve().parent.parent


class ExtractTests(unittest.TestCase):
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
