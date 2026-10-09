"""Literal labeled 2009 retail inventory; default CLI stays byte-compatible."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/3.1.0-wikitext.txt'


class LegacyFunctionLabelsTests(unittest.TestCase):
    def generate(self, source, output, flags=()):
        result = subprocess.run([sys.executable, '-B',
                                 str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                                 '3.1.0', str(source), '2259594', str(output), *flags],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(output.read_bytes())

    def test_labeled_and_supplemental_contracts_preserve_literal_lines(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'register.json'
            default = self.generate(SOURCE, output)
            original = output.read_bytes()
            self.assertEqual(default['entries'], [])
            register = self.generate(SOURCE, output, ['--legacy-function-labels', '--client-line', 'retail'])
            rows = register['entries']
            self.assertEqual(register['client_line'], 'retail')
            self.assertEqual(register['header_counts'], [])
            self.assertEqual(len({row['id'] for row in rows}), len(rows))
            self.assertEqual([(row['symbol'], row['direction']) for row in rows[:4]],
                             [('GetNumTalentGroups', 'added'), ('GetActiveTalentGroup', 'added'),
                              ('SetActiveTalentGroup', 'added'), ('GetGlyphLink', 'changed')])
            self.assertEqual(rows[3]['annotation'], '* UPDATED - link = GetGlyphLink(index [,talentGroup])')
            self.assertEqual([(row['symbol'], row['direction']) for row in rows[-10:]],
                             [('UnitAura', 'changed'), ('UnitBuff', 'changed'), ('UnitDebuff', 'changed'),
                              ('RegisterAutoHide', 'added'), ('UnregisterAutoHide', 'added'),
                              ('AddToAutoHide', 'added'), ('GetInventoryItemsForSlot', 'added'),
                              ('GameTooltip:SetGlyph', 'changed'), ('GetPlayerFacing', 'changed'),
                              ('GetPlayerFacing', 'changed')])
            lines = SOURCE.read_text().splitlines()
            for row in rows:
                self.assertEqual(row['annotation'], lines[row['wikitext_line'] - 1])
            self.assertNotIn('GetTargetFacing', {row['symbol'] for row in rows})
            self.generate(SOURCE, output)
            self.assertEqual(output.read_bytes(), original)

    def test_questions_and_unlabeled_calls_are_not_publication_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'source.wikitext'
            output = Path(directory) / 'register.json'
            source.write_text('== Talent Functions ==\n* NEW - result = Known(a [,b]) -- note\n'
                              '== Other Info ==\n=== Target Facing ===\nAre we getting GetTargetFacing()?\n'
                              '=== UnitAura ===\nQuote: UnitAura()\n'
                              'name, caster = UnitAura("unit", index)\n')
            rows = self.generate(source, output, ['--legacy-function-labels'])['entries']
            self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows],
                             [('Known', 'added', 2), ('UnitAura', 'changed', 8)])


class SourceAccountingReplayTests(unittest.TestCase):
    def test_original_and_current_ledgers_reproduce_in_disposable_root(self):
        evidence = ROOT / 'data/patch-api/evidence/3.1.0-session-2026-10-09'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tools = root / 'tools'
            sources = root / 'data/patch-api/sources'
            frozen = root / 'data/patch-api/evidence/3.1.0-session-2026-10-09'
            for path in (tools, sources, frozen / 'current'):
                path.mkdir(parents=True, exist_ok=True)
            for name in ('build_patch_3_1_0_accounting.py', 'extract_patch_non_inventory.py'):
                shutil.copyfile(ROOT / 'tools' / name, tools / name)
            for name, target in [('historical-source.wikitext', 'api-changes.wikitext'),
                                 ('historical-register.json', 'wikitext-register.json'),
                                 ('historical-extract.txt', 'api-changes.txt')]:
                shutil.copyfile(evidence / name, sources / ('3.1.0-' + target))
            shutil.copyfile(evidence / 'own-sweep-green-results.json', frozen / 'own-sweep-green-results.json')
            for name in ('publication-green-results.json', 'model-green-receipt.json'):
                shutil.copyfile(evidence / 'current' / name, frozen / 'current' / name)
            argv = [sys.executable, '-B', str(tools / 'build_patch_3_1_0_accounting.py')]
            result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((sources / '3.1.0-page-coverage.json').read_bytes(),
                             (evidence / 'historical-page-coverage.json').read_bytes())
            self.assertEqual((sources / '3.1.0-signatures.json').read_bytes(),
                             (evidence / 'historical-signatures.json').read_bytes())
            result = subprocess.run(argv + ['--current-radians'], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            for suffix in ('page-coverage.json', 'signatures.json'):
                self.assertEqual((sources / ('3.1.0-' + suffix)).read_bytes(),
                                 (evidence / 'current' / suffix).read_bytes())
            integrated = frozen / 'integrated'
            integrated.mkdir()
            shutil.copyfile(evidence / 'integrated' / 'publication-results.json',
                            integrated / 'publication-results.json')
            result = subprocess.run(argv + ['--current-radians', '--observations',
                'data/patch-api/evidence/3.1.0-session-2026-10-09/integrated/publication-results.json'],
                cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            for suffix in ('page-coverage.json', 'signatures.json'):
                self.assertEqual((sources / ('3.1.0-' + suffix)).read_bytes(),
                                 (ROOT / 'data/patch-api/sources' / ('3.1.0-' + suffix)).read_bytes())
            coverage = json.loads((sources / '3.1.0-page-coverage.json').read_bytes())
            modeled = [row for row in coverage['source_rows'] if 'modeled-radians-state-read' in row['capabilities']]
            self.assertEqual([row['source_id'] for row in modeled], ['signature-239-1', 'signature-241-1'])
            self.assertTrue(all('UNPROVEN' in row['note'] for row in modeled))


if __name__ == '__main__':
    unittest.main()
