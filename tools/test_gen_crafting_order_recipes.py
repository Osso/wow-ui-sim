"""Behavioral fixtures for the pinned crafting-order projection generator."""

import hashlib
from pathlib import Path
import tempfile
import unittest

from gen_crafting_order_recipes import order_recipe_ids, read_pinned_rows


class CraftingOrderProjectionTests(unittest.TestCase):
    def test_order_existence_includes_zero_field_and_deduplicates_spells(self):
        orders = [
            {"ID": "36", "SkillLineAbilityID": "46625", "Field_10_0_2_46479_002": "1"},
            {"ID": "854", "SkillLineAbilityID": "49927", "Field_10_0_2_46479_002": "0"},
            {"ID": "855", "SkillLineAbilityID": "49927", "Field_10_0_2_46479_002": "1"},
        ]
        abilities = [
            {"ID": "49927", "Spell": "438914"},
            {"ID": "46625", "Spell": "365729"},
            {"ID": "1", "Spell": "2662"},
        ]
        self.assertEqual(order_recipe_ids(orders, abilities), [365729, 438914])

    def test_missing_ability_relation_is_an_error(self):
        with self.assertRaises(KeyError):
            order_recipe_ids([{"SkillLineAbilityID": "46625"}], [])

    def test_pinned_input_accepts_exact_bytes_and_rejects_drift(self):
        data = b"ID,Spell\n46625,365729\n"
        expected_hash = hashlib.sha256(data).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "SkillLineAbility.csv"
            path.write_bytes(data)
            self.assertEqual(
                read_pinned_rows(path, expected_hash),
                [{"ID": "46625", "Spell": "365729"}],
            )
            path.write_bytes(data.replace(b"365729", b"365730"))
            with self.assertRaisesRegex(ValueError, "pinned 12.1.5.70391 SHA256 mismatch"):
                read_pinned_rows(path, expected_hash)


if __name__ == "__main__":
    unittest.main()
