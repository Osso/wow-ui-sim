"""Own historical evidence replay and restored source/log tamper controls."""
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = Path('data/patch-api/evidence/4.1.0-session-2026-10-09')


class OwnValidatorTests(unittest.TestCase):
    def test_clean_replay_rejects_source_and_own_log_tampering(self):
        script = ROOT / EVIDENCE / 'validate.py'
        self.assertTrue(script.exists(), 'own historical validator is not implemented')
        spec = importlib.util.spec_from_file_location('p410_validator', script)
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        manifest = json.loads((ROOT / EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            root = Path(directory)
            here = root / EVIDENCE
            paths = list(manifest['sealed_files']) + [
                str(EVIDENCE / 'historical-inputs.json'),
                str(EVIDENCE / 'historical-blobs.json.gz'),
            ]
            for relative in paths:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, destination)
            clean = validator.validate(root, here)
            self.assertEqual(clean['publication_rows'], len(json.loads(
                (root / 'data/patch-api/sources/4.1.0-wikitext-register.json').read_text())['entries']))
            for relative in [
                'data/patch-api/sources/4.1.0-api-changes.wikitext',
                str(EVIDENCE / 'own-sweep-green.log'),
            ]:
                target = root / relative
                original = target.read_bytes()
                try:
                    target.write_bytes(original + b'\nTAMPER\n')
                    with self.assertRaisesRegex(AssertionError, 'sealed input: ' + relative):
                        validator.validate(root, here)
                finally:
                    target.write_bytes(original)
                self.assertEqual(validator.validate(root, here), clean)


if __name__ == '__main__':
    unittest.main()
