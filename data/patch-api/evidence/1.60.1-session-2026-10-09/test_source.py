"""Bounded frozen Forever source contracts, not runtime/native acceptance."""
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

EVIDENCE = Path(__file__).resolve().parent


class SourceTests(unittest.TestCase):
    def validator(self):
        path = EVIDENCE / 'validate.py'
        self.assertTrue(path.exists(), 'frozen Forever source validator missing')
        spec = importlib.util.spec_from_file_location('forever_validate', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_frozen_response_manifest_registry(self):
        result = self.validator().replay(EVIDENCE)
        self.assertEqual(result['registry_pages'], 101)
        self.assertEqual(result['registry_endpoint'], '1.0.0')
        self.assertEqual(result['source_interface'], 16001)
        self.assertEqual(result['client_line'], 'wow-forever-camelot')

    def test_literal_inventory_headers_and_command(self):
        result = self.validator().replay(EVIDENCE)
        self.assertEqual(result['inventory_occurrences'], 1876)
        self.assertEqual(result['header_mismatches'], [
            {'section': 'global-api', 'direction': 'added', 'header_count': 176, 'parsed_count': 175},
            {'section': 'events', 'direction': 'added', 'header_count': 24, 'parsed_count': 25}])
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        commands = [row for row in ledger['inventory_rows'] if row.get('kind') == 'command']
        self.assertEqual([(row['symbol'], row['direction']) for row in commands], [('dumpSmallAlloc', 'added')])
        self.assertEqual(commands[0]['literal_fields'], {'type': 'command', 'name': 'dumpSmallAlloc', 'cat': '4'})
        camera = next(row for row in ledger['inventory_rows'] if row['symbol'] == 'CameraFollowPitchOffset')
        self.assertEqual(camera['literal_fields']['default'], '15.000000')
        self.assertEqual(camera['literal_fields']['scope'], 'Account')

    def test_every_nonblank_occurrence_header_signature_prose_and_reference_required(self):
        module = self.validator()
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        for field in ('source_rows', 'inventory_rows', 'signature_ledger', 'prose_ledger', 'header_ledger', 'reference_ledger'):
            for index in range(len(ledger[field])):
                altered = dict(ledger)
                altered[field] = ledger[field][:index] + ledger[field][index + 1:]
                with self.subTest(field=field, index=index), self.assertRaises(AssertionError):
                    module.validate_ledger(altered, ledger)

    def test_signatures_do_not_invent_parameters_or_return_tuples(self):
        self.validator().replay(EVIDENCE)
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        signature = next(row for row in ledger['signature_ledger'] if row['symbol'] == 'C_Spell.GetItemCooldown')
        self.assertIsNone(signature['literal_arguments'])
        self.assertIsNone(signature['literal_returns'])
        self.assertEqual(signature['status'], 'UNPROVEN')
        name = next(row for row in ledger['signature_ledger'] if row['wikitext_line'] == 12 and row['symbol'] == 'UnitName')
        self.assertEqual(name['literal_arguments'], '"player"')
        self.assertEqual(name['literal_returns'], None)
        self.assertIn('"Unknown", nil', name['source_text'])
        self.assertTrue(any(row['symbol'] == 'PaperDollItemSlotButton_OnModifableClick' for row in ledger['inventory_rows']))

    def test_foreign_history_native_and_publication_credit_rejected(self):
        module = self.validator()
        ledger = json.loads((EVIDENCE / 'original/ledger.json').read_bytes())
        for field, value in [('client_line', 'retail'), ('source_interface', 12105), ('actual_successors', ['2.5.6'])]:
            altered = dict(ledger, **{field: value})
            with self.assertRaises(AssertionError):
                module.validate_ledger(altered, ledger)
        altered = copy.deepcopy(ledger)
        altered['inventory_rows'][0]['status'] = 'native-covered'
        with self.assertRaises(AssertionError):
            module.validate_ledger(altered, ledger)

    def test_literal_duplicate_and_malformed_fixture_preserved(self):
        module = self.validator()
        raw = '''===Global API===
! Added <small>(3)</small>
! Removed <small>(0)</small>
| valign="top"
: {{api|C_Example.Read}}
: {{api|C_Example.Read}}
: {{api|C_Example.Mispelled}}
</div>
| valign="top"
</div>
|}
'''
        register = module.accounting.parse_register(EVIDENCE, raw.encode())
        self.assertEqual([r['symbol'] for r in register['entries']], ['C_Example.Read', 'C_Example.Read', 'C_Example.Mispelled'])
        self.assertEqual(len({r['id'] for r in register['entries']}), 3)
        self.assertEqual(register['header_counts'][0]['parsed_count'], 3)

    def test_unexpanded_link_and_template_boundaries(self):
        module = self.validator()
        result = module.accounting.references(9, '{{:Other page}} [[Some API]] [https://example.invalid/doc linked summary]')
        self.assertEqual([r['kind'] for r in result], ['transclusion', 'wiki-link', 'external-link'])
        self.assertTrue(all(r['status'] == 'UNPROVEN' and r['expanded'] is False for r in result))

    def test_git_free_fresh_replay_and_disk_tamper_restore(self):
        self.validator()
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / 'evidence'
            shutil.copytree(EVIDENCE, destination)
            empty_bin = Path(directory) / 'empty-bin'
            empty_bin.mkdir()
            command = [sys.executable, '-B', str(destination / 'validate.py')]
            def run():
                return subprocess.run(command, cwd=directory, capture_output=True, text=True,
                                      env={'PATH': str(empty_bin), 'PYTHONDONTWRITEBYTECODE': '1'})
            result = run()
            self.assertEqual(result.returncode, 0, result.stderr)
            for name in ('original/ledger.json', 'original/red.log'):
                path = destination / name
                original = path.read_bytes()
                try:
                    path.write_bytes(original + b'\nTAMPER\n')
                    failed = run()
                    self.assertNotEqual(failed.returncode, 0)
                    self.assertIn('seal: ' + name, failed.stderr)
                finally:
                    path.write_bytes(original)
                self.assertEqual(path.read_bytes(), original)
                restored = run()
                self.assertEqual(restored.returncode, 0, restored.stderr)
            self.assertFalse((destination / '.git').exists())
            self.assertFalse((destination / 'target').exists())


if __name__ == '__main__':
    unittest.main(verbosity=2)
