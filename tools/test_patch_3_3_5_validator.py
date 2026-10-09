"""Historical replay: relocated, no Git/target, current closures cannot rewrite receipts."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = Path('data/patch-api/evidence/3.3.5-session-2026-10-09')


class HistoricalReplayTests(unittest.TestCase):
    def test_relocated_replay_and_exact_seal_tamper_controls(self):
        script = ROOT / EVIDENCE / 'validate.py'
        self.assertTrue(script.exists(), 'own historical validator is not implemented')
        manifest = json.loads((ROOT / EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p335-relocated-') as directory:
            root = Path(directory)
            paths = list(manifest['sealed_files']) + [
                str(EVIDENCE / 'historical-inputs.json'), str(EVIDENCE / 'validate.py')]
            for relative in paths:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, destination)
            self.assertFalse((root / '.git').exists())
            self.assertFalse((root / 'target').exists())
            argv = [sys.executable, '-B', str(root / EVIDENCE / 'validate.py')]

            def replay():
                return subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            register = json.loads((root / 'data/patch-api/sources/3.3.5-wikitext-register.json').read_text())
            self.assertEqual(counts['publication_rows'], len(register['entries']))
            self.assertEqual(counts['publication_rows'], counts['publication_ok'] + counts['publication_gaps'])
            self.assertEqual(counts['negative_gaps'], counts['publication_gaps'] + 1)
            # Synthetic future current closures and later audit must not affect originals.
            for relative, value in [
                ('data/patch-api/sources/3.3.5-page-coverage.json', {'source_rows': []}),
                ('tests/data/patch_3_3_5_sweep_known_gaps.json', []),
                ('data/patch-api/sources/4.0.1-wikitext-register.json', {'entries': []}),
            ]:
                p = root / relative
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text(json.dumps(value) + '\n')
            future = replay()
            self.assertEqual(future.returncode, 0, future.stderr)
            self.assertEqual(json.loads(future.stdout), counts)
            for relative in [
                'data/patch-api/sources/3.3.5-api-changes.wikitext',
                str(EVIDENCE / 'own-sweep-green.log'),
                str(EVIDENCE / 'historical-page-coverage.json'),
                str(EVIDENCE / 'historical-known-gaps.json'),
                str(EVIDENCE / 'historical-bundle.json.gz'),
            ]:
                target = root / relative
                original = target.read_bytes()
                with self.subTest(seal=relative):
                    try:
                        target.write_bytes(original + b'\nTAMPER\n')
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        self.assertIn('sealed input: ' + relative, changed.stderr)
                    finally:
                        target.write_bytes(original)
                    self.assertEqual(target.read_bytes(), original)
                    restored = replay()
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(json.loads(restored.stdout), counts)


if __name__ == '__main__':
    unittest.main()
