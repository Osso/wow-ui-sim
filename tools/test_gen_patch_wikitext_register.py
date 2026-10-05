"""Behavior fixtures for raw consolidated inventories."""
import unittest

from gen_patch_wikitext_register import parse_section


class InventoryTests(unittest.TestCase):
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
