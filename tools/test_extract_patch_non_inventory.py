"""Observable plaintext and source-ID contracts for retained patch pages."""
import unittest
from pathlib import Path

from extract_patch_non_inventory import extract_text, seed_rows

ROOT = Path(__file__).resolve().parent.parent


class ExtractTests(unittest.TestCase):
    def test_12_0_0_plaintext_stays_identical(self):
        sources = ROOT / 'data/patch-api/sources'
        self.assertEqual(
            extract_text((sources / '12.0.0-api-changes.wikitext').read_text()),
            (sources / '12.0.0-api-changes.txt').read_text())

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
