import importlib.util
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "forever_strings", Path(__file__).resolve().parents[1] / "gen_forever_global_strings.py"
)
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)


class ForeverStringGeneratorTests(unittest.TestCase):
    def test_multiline_quotes_and_format_tokens_survive_generation(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "input.csv"
            output = Path(directory) / "output.rs"
            source.write_text('ID,BaseTag,TagText_lang,Flags\n1,HELP,"Line one\n""quoted"" %d",1\n')
            GEN.generate(source, output)
            self.assertIn('"HELP" => "Line one\\n\\"quoted\\" %d",', output.read_text())

    def test_duplicate_tag_fails_before_replacing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "input.csv"
            output = Path(directory) / "output.rs"
            source.write_text('ID,BaseTag,TagText_lang,Flags\n1,A,first,1\n2,A,second,1\n')
            output.write_text("unchanged")
            with self.assertRaisesRegex(ValueError, "duplicate tag"):
                GEN.generate(source, output)
            self.assertEqual(output.read_text(), "unchanged")


if __name__ == "__main__":
    unittest.main()
