import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

GENERATOR = Path(__file__).resolve().parents[1] / "gen_spell_aura_secrecy.py"
spec = importlib.util.spec_from_file_location("gen_spell_aura_secrecy", GENERATOR)
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


class SpellAuraSecrecyGenerationTests(unittest.TestCase):
    def test_base_rows_keep_both_flags_and_ignore_other_difficulties(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "SpellMisc.csv"
            path.write_text(
                "SpellID,DifficultyID,Attributes_15\n"
                "35395,0,0\n1126,0,67108864\n343960,0,33554432\n"
                "1317008,0,-942733312\n1126,2,0\n77,1,33554432\n"
            )
            self.assertEqual(
                generator.read_base_aura_flags(path, 0x2000000, 0x4000000),
                [(1126, 0x4000000), (343960, 0x2000000), (1317008, 0x6000000)],
            )

    def test_conflicting_base_rows_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "SpellMisc.csv"
            path.write_text("SpellID,DifficultyID,Attributes_15\n1126,0,0\n1126,0,67108864\n")
            with self.assertRaisesRegex(ValueError, "conflicting base rows for spell 1126"):
                generator.read_base_aura_flags(path, 0x2000000, 0x4000000)

    def test_missing_source_columns_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "SpellMisc.csv"
            path.write_text("SpellID,DifficultyID\n1126,0\n")
            with self.assertRaisesRegex(ValueError, "missing columns"):
                generator.read_base_aura_flags(path, 0x2000000, 0x4000000)

    def test_flag_definition_contents_must_match_pinned_blob(self):
        with tempfile.TemporaryDirectory() as directory:
            flags = Path(directory) / "flags.dbdf"
            flags.write_text("0x2000000 AURA_ALWAYS_SECRET\n0x4000000 AURA_NEVER_SECRET\n")
            self.assertEqual(generator.read_flag_definitions(flags), (0x2000000, 0x4000000))
            data = flags.read_bytes()
            blob_sha = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
            tree = Path(directory) / "tree.json"
            tree.write_text(json.dumps({"sha": "pinned-tree", "tree": [{"path": generator.FLAG_PATH, "sha": blob_sha}]}))
            self.assertEqual(generator.verify_flag_provenance(flags, tree), ("pinned-tree", blob_sha))
            flags.write_text(flags.read_text() + "0x1 CHANGED\n")
            with self.assertRaisesRegex(ValueError, "do not match pinned definition blob"):
                generator.verify_flag_provenance(flags, tree)


if __name__ == "__main__":
    unittest.main()
