import tempfile
import unittest
from pathlib import Path

from gen_ptr_documented_events import generate


class GeneratePtrEventsTest(unittest.TestCase):
    def test_sorted_unique_events_from_all_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            addons = Path(tmp)
            (addons / ".wow-ui-sim-blizzard-ui-provenance").write_text("profile=ptr\nversion=12.1.7.70493\n")
            docs = addons / "Blizzard_APIDocumentationGenerated"
            docs.mkdir()
            (docs / "a.lua").write_text('Type = "Event", LiteralName = "Z_EVENT",\nType = "Function", LiteralName = "NOT_EVENT",')
            (docs / "b.lua").write_text('Type = "Event", LiteralName = "A_EVENT",\nType = "Event", LiteralName = "Z_EVENT",')
            result = generate(addons)
            self.assertIn("files: 2; events: 6", result)
            self.assertIn('// Blizzard_GlueXML/Mainline/CharacterSelect.lua:40\n    "CHARACTER_UNDELETE_STARTED",', result)
            self.assertIn('// Blizzard_DeclensionFrame/Mainline/DeclensionFrameGlue.lua:12\n    "FORCE_DECLINE_CHARACTER",', result)
            self.assertLess(result.index('"A_EVENT"'), result.index('"Z_EVENT"'))
            self.assertEqual(result.count('"Z_EVENT"'), 1)
            self.assertNotIn("NOT_EVENT", result)

    def test_rejects_unrecognized_event_declaration(self):
        with tempfile.TemporaryDirectory() as tmp:
            addons = Path(tmp)
            (addons / ".wow-ui-sim-blizzard-ui-provenance").write_text("profile=ptr\nversion=1\n")
            docs = addons / "Blizzard_APIDocumentationGenerated"
            docs.mkdir()
            (docs / "a.lua").write_text('Type = "Event", LiteralName = "bad-name",')
            with self.assertRaises(ValueError):
                generate(addons)


if __name__ == "__main__":
    unittest.main()
