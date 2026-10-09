"""Frozen retail CVar definitions: literal spelling, prose, opt-in and byte isolation."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/3.0.3-session-2026-10-09'
SOURCE = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/3.0.3-wikitext.txt'


class CVarDefinitionTests(unittest.TestCase):
    def generate(self, source, output, flags=()):
        return subprocess.run([
            sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
            '3.0.3', str(source), '5407654', str(output), *flags,
        ], cwd=ROOT, capture_output=True, text=True)

    def test_literal_definitions_and_default_byte_isolation(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            default = self.generate(SOURCE, output)
            self.assertEqual(default.returncode, 0, default.stderr)
            original = output.read_bytes()
            self.assertEqual(original, (EVIDENCE / 'default-register.json').read_bytes())
            self.assertEqual(json.loads(original)['entries'], [])
            result = self.generate(SOURCE, output, ['--legacy-cvar-definitions', '--client-line', 'retail'])
            self.assertEqual(result.returncode, 0, result.stderr)
            register = json.loads(output.read_text())
            rows = register['entries']
            self.assertEqual(register['client_line'], 'retail')
            self.assertEqual(register['header_counts'], [])
            self.assertEqual([(r['symbol'], r['section'], r['direction'], r['wikitext_line']) for r in rows], [
                ('syncronizeConfig', 'cvars', 'added', 4),
                ('synchronizeBindings', 'cvars', 'added', 5),
                ('synchronizeMacros', 'cvars', 'added', 6),
            ])
            self.assertEqual([r['annotation'] for r in rows], SOURCE.read_text().splitlines()[3:])
            self.assertEqual([r['description'] for r in rows], [
                'Flag - 0/1 to disable/enable synchronization of UI settings.',
                'Flag - 0/1 to disable/enable synchronization of key bindings.',
                'Flag - 0/1 to disable/enable synchronization of macros.',
            ])
            self.assertTrue(all('page_default' not in r and 'signature' not in r for r in rows))
            self.assertEqual(len({r['id'] for r in rows}), len(rows))
            self.assertEqual(self.generate(SOURCE, output).returncode, 0)
            self.assertEqual(output.read_bytes(), original)

    def test_section_boundary_and_malformed_definition_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            source, output = Path(directory) / 'source.txt', Path(directory) / 'register.json'
            source.write_text('== New CVars ==\n; ExactName : Flag - 0/1.\n'
                              '== References ==\n; IgnoreMe : Not a CVar.\n')
            result = self.generate(source, output, ['--legacy-cvar-definitions'])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([r['symbol'] for r in json.loads(output.read_text())['entries']], ['ExactName'])
            source.write_text('== New CVars ==\n; broken definition\n')
            result = self.generate(source, output, ['--legacy-cvar-definitions'])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('malformed CVar definition at line 2', result.stderr)

    def test_recorded_template_registers_and_extracts_reproduce(self):
        for patch in ['3.3.3', '3.3.5', '4.0.1']:
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
                spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
                extractor = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(extractor)
                flags = {flag.removeprefix('--').replace('-', '_'): True
                         for flag in provenance['extractor_flags']}
                text = extractor.extract_text(source.read_text(), **flags)
                self.assertEqual(text.encode(), (sources / f'{patch}-api-changes.txt').read_bytes())


if __name__ == '__main__':
    unittest.main()
