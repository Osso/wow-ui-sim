"""Bounded complete source-accounting and serialized historical replay tests."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

EVIDENCE = Path(__file__).resolve().parent


class AccountingTests(unittest.TestCase):
    def validator(self):
        path = EVIDENCE / 'validate.py'
        self.assertTrue(path.exists(), 'historical accounting validator missing')
        spec = importlib.util.spec_from_file_location('launch_validate', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_complete_accounting_and_separate_limits(self):
        module = self.validator()
        result = module.replay(EVIDENCE)
        self.assertEqual(result['native_observations'], 0)
        self.assertEqual(result['runtime_observations'], 0)
        self.assertEqual(result['meaningful_closures'], 0)
        self.assertGreater(result['inventory_occurrences'], 300)
        self.assertGreater(result['prose_limits'], 20)
        self.assertEqual(result['literal_command_occurrences'], 0)
        self.assertEqual(result['actual_successors'], ['3.3.0', '3.3.3', '3.3.5', '4.0.1'])
        self.assertEqual(result['queued_successors'], ['3.0.3', '3.0.8', '3.1.0', '3.2.0'])

    def test_every_literal_row_occurrence_signature_and_prose_required(self):
        module = self.validator()
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        for field in ('source_rows', 'inventory_rows', 'signature_ledger', 'prose_ledger', 'header_ledger'):
            for index in range(len(ledger[field])):
                altered = copy.deepcopy(ledger)
                altered[field].pop(index)
                with self.subTest(field=field, index=index), self.assertRaises(AssertionError):
                    module.validate_ledger(EVIDENCE, altered)

    def test_positive_credit_and_foreign_successor_rejected(self):
        module = self.validator()
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        altered = copy.deepcopy(ledger)
        altered['inventory_rows'][0]['status'] = 'native-covered'
        with self.assertRaises(AssertionError):
            module.validate_ledger(EVIDENCE, altered)
        altered = copy.deepcopy(ledger)
        altered['actual_successors'].append('3.4.3')
        with self.assertRaises(AssertionError):
            module.validate_ledger(EVIDENCE, altered)

    def test_git_free_replay_and_serialized_tampers_restore_bytes(self):
        self.validator()
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / 'evidence'
            shutil.copytree(EVIDENCE, destination)
            empty_path = Path(directory) / 'empty-bin'
            empty_path.mkdir()
            command = [sys.executable, '-B', str(destination / 'validate.py')]
            def run():
                return subprocess.run(command, cwd=directory, capture_output=True, text=True,
                                      env={'PATH': str(empty_path), 'PYTHONDONTWRITEBYTECODE': '1'})
            result = run()
            self.assertEqual(result.returncode, 0, result.stderr)
            for name in ('original/ledger.json', 'green.log', 'original/source.wikitext',
                         'historical-gen_patch_wikitext_register.py'):
                path = destination / name
                original = path.read_bytes()
                path.write_bytes(original + b'\nTAMPER\n')
                failed = run()
                self.assertNotEqual(failed.returncode, 0)
                self.assertIn('seal: ' + name, failed.stderr)
                path.write_bytes(original)
                self.assertEqual(path.read_bytes(), original)
                restored = run()
                self.assertEqual(restored.returncode, 0, restored.stderr)


if __name__ == '__main__':
    unittest.main(verbosity=2)
