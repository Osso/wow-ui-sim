"""Targeted source reproduction and fresh-process historical tamper controls."""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/3.2.0-session-2026-10-09'


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SourceAccountingTests(unittest.TestCase):
    def test_literal_inventory_and_full_extract_reproduce(self):
        sources = ROOT / 'data/patch-api/sources'
        raw = (sources / '3.2.0-api-changes.wikitext').read_text()
        register = json.loads((sources / '3.2.0-wikitext-register.json').read_text())
        generator = load_tool('gen_patch_wikitext_register')
        extractor = load_tool('extract_patch_non_inventory')
        self.assertEqual(generator.parse_wrath_retail_change_bullets(raw), register['entries'])
        self.assertEqual(extractor.extract_text(raw, numbered_reflist=True),
                         (sources / '3.2.0-api-changes.txt').read_text())
        literal = json.loads((EVIDENCE / 'literal-inventory.json').read_text())
        self.assertEqual([row['symbol'] for row in literal['slash_command_occurrences']],
                         ['/dump', '/eventtrace', '/framestack', '/reload'])
        self.assertEqual([row['literal'] for row in literal['call_signatures']],
                         [row['source_signature'] for row in register['entries']
                          if 'source_signature' in row])
        self.assertEqual(len(register['entries']), 43)
        self.assertEqual(len(literal['call_signatures']), 24)
        uncertain = next(row for row in register['entries'] if row['symbol'] == 'UnitIsPlusMob')
        self.assertEqual((uncertain['direction'], uncertain['source_change']),
                         ('changed', 'updated or removed'))
        self.assertIn('GetDifficutlyColor', [row['symbol'] for row in register['entries']])
        self.assertNotIn('GetDifficultyColor', [row['symbol'] for row in register['entries']])

    def test_original_ledger_exact_id_set_and_negative_boundary(self):
        literal = json.loads((EVIDENCE / 'literal-inventory.json').read_text())
        coverage = json.loads((EVIDENCE / 'historical-page-coverage.json').read_text())
        extractor = load_tool('extract_patch_non_inventory')
        extract = (ROOT / 'data/patch-api/sources/3.2.0-api-changes.txt').read_text()
        ids = {row['source_id'] for group in literal.values() for row in group}
        ids.update(row['source_id'] for row in extractor.seed_rows(extract, '3.2.0'))
        rows = coverage['source_rows']
        self.assertEqual(len(rows), len(ids))
        self.assertEqual({row['source_id'] for row in rows}, ids)
        self.assertTrue(all(row['note'] for row in rows))
        green = json.loads((EVIDENCE / 'own-sweep-green-results.json').read_text())
        negative = json.loads((EVIDENCE / 'negative-results.json').read_text())
        known = set(json.loads((EVIDENCE / 'historical-known-gaps.json').read_text()))
        self.assertEqual({key for key, row in green.items() if not row['ok']}, known)
        self.assertEqual({key for key, row in negative.items() if not row['ok']},
                         known | {'p320-negative-control'})
        self.assertEqual(len(negative), len(green))

    def test_fresh_process_archive_ignores_future_closure_rejects_tampering(self):
        script = EVIDENCE / 'validate.py'
        self.assertTrue(script.is_file(), 'portable historical validator missing')
        manifest = json.loads((EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(dir=EVIDENCE) as directory:
            fixture = Path(directory)
            for relative in list(manifest['external']) + [
                    'validate.py', 'historical-inputs.json', 'historical-blobs.json.gz']:
                shutil.copyfile(EVIDENCE / relative, fixture / relative)
            self.assertFalse((fixture / '.git').exists())
            self.assertFalse((fixture / 'target').exists())
            self.assertFalse((fixture / 'tools').exists())

            def replay():
                return subprocess.run([sys.executable, '-B', str(fixture / 'validate.py')],
                                      cwd=fixture, env={'PATH': ''}, capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            summary = json.loads(clean.stdout)
            self.assertEqual(summary['publication_rows'], len(json.loads(
                (ROOT / 'data/patch-api/sources/3.2.0-wikitext-register.json').read_text())['entries']))
            self.assertEqual(summary['negative_gaps'], summary['publication_gaps'] + 1)
            self.assertEqual(summary['ledger_rows'], sum(summary['ledger_statuses'].values()))
            # Unarchived future accounting cannot alter original receipt replay.
            current_sources = fixture / 'data/patch-api/sources'
            current_tests = fixture / 'tests/data'
            current_sources.mkdir(parents=True)
            current_tests.mkdir(parents=True)
            (current_sources / '3.2.0-page-coverage.json').write_text('{"source_rows": []}\n')
            (current_tests / 'patch_3_2_0_sweep_known_gaps.json').write_text('[]\n')
            self.assertEqual(json.loads(replay().stdout), summary)
            for relative in ['source-response.json', 'own-sweep-green.log',
                             'historical-page-coverage.json', 'historical-known-gaps.json']:
                target = fixture / relative
                original = target.read_bytes()
                try:
                    if relative.endswith('.json'):
                        value = json.loads(original)
                        if isinstance(value, list):
                            value.pop()
                        elif 'source_rows' in value:
                            value['source_rows'].pop()
                        else:
                            value['query']['pages']['499137']['revisions'][0]['revid'] += 1
                        target.write_text(json.dumps(value) + '\n')
                    else:
                        target.write_bytes(original + b'\nSERIALIZED TAMPER\n')
                    bad = replay()
                    self.assertNotEqual(bad.returncode, 0)
                    self.assertIn('sealed input: ' + relative, bad.stderr)
                finally:
                    target.write_bytes(original)
                self.assertEqual(target.read_bytes(), original)
                restored = replay()
                self.assertEqual(restored.returncode, 0, restored.stderr)
                self.assertEqual(json.loads(restored.stdout), summary)


if __name__ == '__main__':
    unittest.main()
