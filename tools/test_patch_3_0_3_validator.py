"""Bounded original replay: copied process, no Git/target/current files, exact tampers."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
RELATIVE = Path('data/patch-api/evidence/3.0.3-session-2026-10-09')
EVIDENCE = ROOT / RELATIVE


class HistoricalReplayTests(unittest.TestCase):
    def require_validator(self):
        path = EVIDENCE / 'validate.py'
        self.assertTrue(path.exists(), 'own sealed historical validator is not implemented')
        return path

    def test_fresh_copied_process_and_serialized_tamper_restoration(self):
        self.require_validator()
        manifest = json.loads((EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p303-original-') as directory:
            root = Path(directory)
            here = root / RELATIVE
            here.mkdir(parents=True)
            for name in [*manifest['sealed_files'], 'historical-inputs.json', 'validate.py']:
                shutil.copyfile(EVIDENCE / name, here / name)
            self.assertFalse((root / '.git').exists())
            self.assertFalse((root / 'target').exists())
            self.assertFalse((root / 'src').exists())
            self.assertFalse((root / 'data/patch-api/sources').exists())
            argv = [sys.executable, '-I', '-B', str(here / 'validate.py')]
            environment = dict(os.environ, PATH='/p303-no-git', PYTHONPATH='')

            def replay():
                return subprocess.run(argv, cwd=root, env=environment,
                                      capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual(counts['inventory_rows'], 3)
            self.assertEqual(counts['raw_nonblank_rows'], 5)
            self.assertEqual(counts['ledger_rows'], 8)
            self.assertEqual(counts['signature_rows'], 0)
            self.assertEqual(counts['semantic_unproven'], 3)
            self.assertEqual(counts['publication_unproven'], 3)
            self.assertEqual(counts['meaningful_closures'], 0)
            for relative, value in [
                ('data/patch-api/sources/3.0.3-page-coverage.json', {'source_rows': []}),
                ('tests/data/patch_3_0_3_sweep_known_gaps.json', []),
                ('data/patch-api/sources/3.0.8-wikitext-register.json', {'entries': []}),
            ]:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_text(json.dumps(value) + '\n')
            later = replay()
            self.assertEqual(later.returncode, 0, later.stderr)
            self.assertEqual(later.stdout, clean.stdout)
            for name in [*manifest['sealed_files'], 'historical-inputs.json']:
                with self.subTest(serialized_tamper=name):
                    target = here / name
                    original = target.read_bytes()
                    try:
                        target.write_bytes(original + b'\nTAMPER\n')
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        expected = ('historical manifest seal' if name == 'historical-inputs.json'
                                    else 'sealed input: ' + name)
                        self.assertIn(expected, changed.stderr)
                    finally:
                        target.write_bytes(original)
                    self.assertEqual(target.read_bytes(), original)
                    restored = replay()
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(restored.stdout, clean.stdout)
            target = here / 'historical-known-gaps.json'
            original = target.read_bytes()
            try:
                target.unlink()
                self.assertNotEqual(replay().returncode, 0)
            finally:
                target.write_bytes(original)
            self.assertEqual(target.read_bytes(), original)
            self.assertEqual(replay().stdout, clean.stdout)

    def test_every_original_row_gap_and_identity_is_required(self):
        path = self.require_validator()
        spec = importlib.util.spec_from_file_location('p303_validator', path)
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        register = json.loads((EVIDENCE / 'register.json').read_text())
        raw = (EVIDENCE / 'source.wikitext').read_text()
        text = (EVIDENCE / 'extract.txt').read_text()
        ledger = json.loads((EVIDENCE / 'historical-page-coverage.json').read_text())
        gaps = json.loads((EVIDENCE / 'historical-known-gaps.json').read_text())
        validator.verify_accounting(register, raw, text, ledger, gaps)
        for index in range(len(ledger['source_rows'])):
            with self.subTest(omitted_row=index):
                changed = copy.deepcopy(ledger)
                del changed['source_rows'][index]
                with self.assertRaises(AssertionError):
                    validator.verify_accounting(register, raw, text, changed, gaps)
        for index in range(len(gaps)):
            with self.subTest(omitted_gap=index):
                changed = copy.deepcopy(gaps)
                del changed[index]
                with self.assertRaises(AssertionError):
                    validator.verify_accounting(register, raw, text, ledger, changed)
        for index in range(len(register['entries'])):
            with self.subTest(omitted_inventory=index):
                changed = copy.deepcopy(register)
                del changed['entries'][index]
                with self.assertRaises(AssertionError):
                    validator.verify_accounting(changed, raw, text, ledger, gaps)
        changed = copy.deepcopy(ledger)
        changed['source_rows'][0]['capabilities'] = ['invented-native-parity']
        with self.assertRaises(AssertionError):
            validator.verify_accounting(register, raw, text, changed, gaps)
        changed = copy.deepcopy(register)
        changed['entries'][0]['symbol'] = 'synchronizeConfig'
        with self.assertRaises(AssertionError):
            validator.verify_accounting(changed, raw, text, ledger, gaps)


if __name__ == '__main__':
    unittest.main()
