"""Numbered reference marker is retained explicitly, never expanded."""
import importlib.util
from pathlib import Path
import unittest

TOOL = Path(__file__).with_name('extract_patch_non_inventory.py')
spec = importlib.util.spec_from_file_location('extractor', TOOL)
extractor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(extractor)


class NumberedReferenceTests(unittest.TestCase):
    def test_numbered_reference_is_opt_in_and_keeps_full_prose(self):
        raw = "== Notes ==\n* ''updated'' {{api|UnitCastingInfo(unit)}} -- result\n{{Reflist|2}}\n"
        with self.assertRaisesRegex(ValueError, 'unhandled template'):
            extractor.extract_text(raw)
        self.assertEqual(
            extractor.extract_text(raw, numbered_reflist=True),
            '== Notes ==\n* updated UnitCastingInfo(unit) -- result\n[References list; columns=2; not expanded]\n',
        )
        old = '== Notes ==\n{{Reflist}}\n'
        self.assertEqual(extractor.extract_text(old, numbered_reflist=True),
                         extractor.extract_text(old))


if __name__ == '__main__':
    unittest.main()
