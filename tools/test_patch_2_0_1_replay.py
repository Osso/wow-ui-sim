"""Own serialized historical proof, independent of mutable current files/Git/target."""
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

ROOT = Path(__file__).resolve().parents[1]
REL = Path('data/patch-api/evidence/2.0.1-session-2026-10-09')
HERE = ROOT / REL


class ReplayTests(unittest.TestCase):
    def require_validator(self):
        path = HERE / 'validate.py'
        self.assertTrue(path.exists(), 'own sealed historical validator missing')
        return path

    def test_copied_no_git_no_target_and_serialized_tamper_restoration(self):
        self.require_validator()
        manifest = json.loads((HERE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p201-source-') as directory:
            root = Path(directory)
            here = root / REL
            here.mkdir(parents=True)
            for name in [*manifest['sealed_files'], 'historical-inputs.json', 'validate.py']:
                shutil.copyfile(HERE / name, here / name)
            for name in ['.git', 'target', 'src', 'tools', 'data/patch-api/sources']:
                self.assertFalse((root / name).exists())
            argv = [sys.executable, '-I', '-B', str(here / 'validate.py')]
            env = dict(os.environ, PATH='/p201-no-git-or-tools', PYTHONPATH='')

            def replay():
                return subprocess.run(argv, cwd=root, env=env, capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual(counts['nonblank_rows'], 316)
            self.assertEqual(counts['headers'], 35)
            self.assertEqual(counts['prose_contract_rows'], 280)
            self.assertEqual(counts['runtime_observations'], 0)
            for path in ['data/patch-api/sources/2.0.1-page-coverage.json',
                         'data/patch-api/sources/2.1.0-wikitext-register.json']:
                dest = root / path
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text('{"invented_current_native_credit": true}\n')
            later = replay()
            self.assertEqual((later.returncode, later.stdout), (0, clean.stdout), later.stderr)
            for name in [*manifest['sealed_files'], 'historical-inputs.json']:
                with self.subTest(tamper=name):
                    target = here / name
                    original = target.read_bytes()
                    try:
                        target.write_bytes(original + b'\nTAMPER\n')
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        self.assertIn('manifest seal' if name == 'historical-inputs.json'
                                      else 'sealed input: ' + name, changed.stderr)
                    finally:
                        target.write_bytes(original)
                    self.assertEqual(target.read_bytes(), original)
                    restored = replay()
                    self.assertEqual((restored.returncode, restored.stdout), (0, clean.stdout),
                                     restored.stderr)

    def test_current_closure_copied_source_and_log_seals(self):
        validator = HERE / 'current/validate.py'
        self.assertTrue(validator.exists(), 'separate current source validator missing')
        original = json.loads((HERE / 'historical-inputs.json').read_text())
        current = json.loads((HERE / 'current/inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p201-current-') as directory:
            root = Path(directory)
            here = root / REL
            (here / 'current').mkdir(parents=True)
            for name in [*original['sealed_files'], 'historical-inputs.json', 'validate.py']:
                shutil.copyfile(HERE / name, here / name)
            for name in [*current['sealed_files'], 'inputs.json', 'validate.py']:
                shutil.copyfile(HERE / 'current' / name, here / 'current' / name)
            argv = [sys.executable, '-I', '-B', str(here / 'current/validate.py')]
            env = dict(os.environ, PATH='/p201-no-git-or-tools', PYTHONPATH='')

            def replay():
                return subprocess.run(argv, cwd=root, env=env, capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual((counts['references'], counts['occurrences']), (12, 295))
            self.assertEqual((counts['current_gap_records'], counts['archived_source_cases']), (793, 6))
            self.assertFalse((root / '.git').exists())
            self.assertFalse((root / 'target').exists())
            for name in [*current['sealed_files'], 'inputs.json']:
                target = here / 'current' / name
                content = target.read_bytes()
                try:
                    target.write_bytes(content + b'\nTAMPER\n')
                    changed = replay()
                    self.assertNotEqual(changed.returncode, 0)
                    self.assertIn('current manifest seal' if name == 'inputs.json'
                                  else 'current sealed input: ' + name, changed.stderr)
                finally:
                    target.write_bytes(content)
                self.assertEqual(target.read_bytes(), content)
                restored = replay()
                self.assertEqual((restored.returncode, restored.stdout), (0, clean.stdout), restored.stderr)
        spec = importlib.util.spec_from_file_location('p201_current', validator)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        expected, gaps = module.load_accounting(HERE / 'current')
        for group in ['source_rows', 'occurrences', 'signature_rows', 'headers', 'references']:
            for index in range(len(expected[group])):
                changed = copy.deepcopy(expected)
                del changed[group][index]
                with self.assertRaises(AssertionError):
                    module.verify_current(changed, gaps, expected)
        for index in range(len(gaps)):
            changed = copy.deepcopy(gaps)
            del changed[index]
            with self.assertRaises(AssertionError):
                module.verify_current(expected, changed, expected)

    def test_every_serialized_row_and_credit_is_required(self):
        path = self.require_validator()
        spec = importlib.util.spec_from_file_location('p201_replay', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        manifest, bundle = module.load_inputs(HERE)
        auditor = module.load_tool(bundle, 'tools/audit_patch_2_0_1_source.py')
        raw = (HERE / 'source.wikitext').read_text()
        ledger = json.loads((HERE / 'historical-page-coverage.json').read_text())
        gaps = json.loads((HERE / 'historical-known-gaps.json').read_text())
        expected = auditor.account(raw)
        module.verify_accounting(ledger, gaps, expected)
        for key in ['source_rows', 'occurrences', 'signature_rows', 'headers', 'references']:
            for index in range(len(ledger[key])):
                changed = copy.deepcopy(ledger)
                del changed[key][index]
                with self.assertRaises(AssertionError, msg=(key, index)):
                    module.verify_accounting(changed, gaps, expected)
        for index in range(len(gaps)):
            changed = copy.deepcopy(gaps)
            del changed[index]
            with self.assertRaises(AssertionError, msg=index):
                module.verify_accounting(ledger, changed, expected)
        for mutation in ['capability', 'spelling', 'signature', 'history']:
            changed = copy.deepcopy(ledger)
            if mutation == 'capability':
                changed['source_rows'][1]['capabilities'] = ['native-parity']
            elif mutation == 'spelling':
                next(r for r in changed['occurrences'] if 'ButtoName' in r['literal'])['literal'] = 'ButtonName'
            elif mutation == 'signature':
                changed['signature_rows'][0]['signature'] = 'invented()'
            else:
                changed['client_line'] = 'classic-tbc'
            with self.assertRaises(AssertionError, msg=mutation):
                module.verify_accounting(changed, gaps, expected)


if __name__ == '__main__':
    unittest.main(verbosity=2)
