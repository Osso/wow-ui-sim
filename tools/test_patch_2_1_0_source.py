"""Owned frozen retail source contracts; no runtime or native measurement."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
CACHE = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09'
EVIDENCE = ROOT / 'data/patch-api/evidence/2.1.0-session-2026-10-09'


class LiteralSourceTests(unittest.TestCase):
    def generate(self, output, flags=()):
        return subprocess.run([
            sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
            '2.1.0', str(CACHE / '2.1.0-wikitext.txt'), '6767102', str(output), *flags,
        ], cwd=ROOT, capture_output=True, text=True)

    def test_literal_inventory_and_default_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            self.assertEqual(self.generate(output).returncode, 0)
            default = output.read_bytes()
            result = self.generate(output, ['--legacy-retail-profiling-summary', '--client-line', 'retail'])
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = json.loads(output.read_text())['entries']
            pairs = [(row['symbol'], row['wikitext_line']) for row in rows]
            for pair in [('GetAddOnMemoryUsage', 16), ('GetCursorInfo', 42),
                         ('GameTooltip:Set*CompareItem', 38), ('IsFeignDeath', 46),
                         ('IsFeignDeath', 85), ('scriptProfile', 19), ('scriptErrors', 140),
                         ('/mt', 121), ('PLAYER_REGEN_ENABLED', 172),
                         ('PLAYER_LEAVING_WORLD', 172), ('ScrollFrame:SetScrollChild', 173),
                         ('SecureGroupPetHeaderTemplate', 105), ('GetWorldStateUIInfo', 176)]:
                self.assertIn(pair, pairs)
            self.assertEqual(len(pairs), len(set(pairs)))
            self.assertEqual(len(rows), len({row['id'] for row in rows}))
            self.assertTrue(all(row['annotation'] == (CACHE / '2.1.0-wikitext.txt').read_text().splitlines()[row['wikitext_line'] - 1] for row in rows))
            self.assertTrue(all('page_default' not in row for row in rows))
            self.assertEqual(self.generate(output).returncode, 0)
            self.assertEqual(output.read_bytes(), default)

    def test_full_accounting_and_omission_controls(self):
        path = EVIDENCE / 'validate.py'
        self.assertTrue(path.exists(), 'own sealed source validator is missing')
        spec = importlib.util.spec_from_file_location('p210', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        ledger = json.loads((EVIDENCE / 'historical-page-coverage.json').read_text())
        gaps = json.loads((EVIDENCE / 'historical-known-gaps.json').read_text())
        register = json.loads((EVIDENCE / 'register.json').read_text())
        raw = (EVIDENCE / 'source.wikitext').read_text()
        text = (EVIDENCE / 'extract.txt').read_text()
        result = module.verify_accounting(register, raw, text, ledger, gaps)
        self.assertEqual(result['meaningful_closures'], 0)
        self.assertEqual(result['named_headers'], 19)
        for key in ['source_rows', 'signature_rows', 'headers']:
            for index in range(len(ledger[key])):
                with self.subTest(omitted=key, index=index):
                    changed = copy.deepcopy(ledger)
                    del changed[key][index]
                    with self.assertRaises(AssertionError):
                        module.verify_accounting(register, raw, text, changed, gaps)
        for index in range(len(gaps)):
            changed = copy.deepcopy(gaps)
            del changed[index]
            with self.assertRaises(AssertionError):
                module.verify_accounting(register, raw, text, ledger, changed)
        for index in range(len(register['entries'])):
            changed = copy.deepcopy(register)
            del changed['entries'][index]
            with self.assertRaises(AssertionError):
                module.verify_accounting(changed, raw, text, ledger, gaps)
        changed = copy.deepcopy(ledger)
        changed['source_rows'][1]['capabilities'] = ['native-parity']
        with self.assertRaises(AssertionError):
            module.verify_accounting(register, raw, text, changed, gaps)

    def test_wrath_template_default_isolation(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            result = subprocess.run([
                sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                '3.4.0', str(CACHE / '3.4.0-wikitext.txt'), '165668', str(output),
            ], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(output.read_bytes(), (EVIDENCE / 'default-3.4.0-register.json').read_bytes())

    def test_recorded_template_outputs(self):
        path = ROOT / 'tools/extract_patch_non_inventory.py'
        spec = importlib.util.spec_from_file_location('extractor', path)
        extractor = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(extractor)
        for patch in ['3.0.3', '3.0.8', '3.1.0']:
            with self.subTest(patch=patch), tempfile.TemporaryDirectory() as directory:
                sources = ROOT / 'data/patch-api/sources'
                provenance = json.loads((sources / f'{patch}-api-changes.provenance.json').read_text())
                source = sources / f'{patch}-api-changes.wikitext'
                output = Path(directory) / 'register.json'
                result = subprocess.run([
                    sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                    patch, str(source), str(provenance['revid']), str(output),
                    *provenance['generator_flags'],
                ], cwd=ROOT, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(output.read_bytes(), (sources / f'{patch}-wikitext-register.json').read_bytes())
                flags = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
                self.assertEqual(extractor.extract_text(source.read_text(), **flags).encode(), (sources / f'{patch}-api-changes.txt').read_bytes())


if __name__ == '__main__':
    unittest.main()
