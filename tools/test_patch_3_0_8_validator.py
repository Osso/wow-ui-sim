"""Fresh-process historical replay, current-state drift and physical tamper controls."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/3.0.8-session-2026-10-09'


class HistoricalReplayTests(unittest.TestCase):
    def test_portable_replay_and_restored_tampering(self):
        self.assertTrue((EVIDENCE / 'validate.py').exists(), 'own validator is missing')
        manifest = json.loads((EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p308-history-') as directory:
            destination = Path(directory) / 'history'
            destination.mkdir()
            for name in list(manifest['sealed_files']) + ['validate.py', 'historical-inputs.json']:
                shutil.copyfile(EVIDENCE / name, destination / name)
            self.assertFalse((destination / '.git').exists())
            self.assertFalse((destination / 'target').exists())
            argv = [sys.executable, '-B', str(destination / 'validate.py')]

            def replay():
                return subprocess.run(argv, cwd=ROOT, capture_output=True, text=True,
                                      env={'PATH': '/no-git-or-cargo'})

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual(counts['inventory_rows'], 56)
            self.assertEqual(counts['negative_gaps'], counts['publication_gaps'] + 1)
            (destination.parent / 'data/patch-api/sources').mkdir(parents=True)
            (destination.parent / 'data/patch-api/sources/3.0.8-page-coverage.json').write_text('{}')
            (destination.parent / 'target').mkdir()
            self.assertEqual(replay().stdout, clean.stdout)
            for name in list(manifest['sealed_files']) + ['historical-inputs.json']:
                target = destination / name
                original = target.read_bytes()
                with self.subTest(tampered=name):
                    try:
                        target.write_bytes(original + b'\nTAMPER\n')
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        self.assertIn('sealed input: ' + name, changed.stderr)
                    finally:
                        target.write_bytes(original)
                    self.assertEqual(target.read_bytes(), original)
                    restored = replay()
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(restored.stdout, clean.stdout)
            target = destination / 'historical-known-gaps.json'
            original = target.read_bytes()
            target.unlink()
            self.assertNotEqual(replay().returncode, 0)
            target.write_bytes(original)
            self.assertEqual(replay().stdout, clean.stdout)


if __name__ == '__main__':
    unittest.main()
