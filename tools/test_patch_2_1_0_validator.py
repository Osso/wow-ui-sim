"""Owned copied-process historical replay, isolated from Git/target/current state."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
RELATIVE = Path('data/patch-api/evidence/2.1.0-session-2026-10-09')
EVIDENCE = ROOT / RELATIVE


class HistoricalReplayTests(unittest.TestCase):
    def test_copied_replay_and_ledger_log_tamper_restoration(self):
        self.assertTrue((EVIDENCE / 'validate.py').exists(), 'own historical validator is missing')
        manifest = json.loads((EVIDENCE / 'historical-inputs.json').read_text())
        with tempfile.TemporaryDirectory(prefix='p210-original-') as directory:
            root = Path(directory)
            here = root / RELATIVE
            here.mkdir(parents=True)
            for name in [*manifest['sealed_files'], 'historical-inputs.json', 'validate.py']:
                shutil.copyfile(EVIDENCE / name, here / name)
            for path in ['.git', 'target', 'src', 'tools', 'data/patch-api/sources']:
                self.assertFalse((root / path).exists())
            argv = [sys.executable, '-I', '-B', str(here / 'validate.py')]
            environment = dict(os.environ, PATH='/p210-no-git-no-cargo', PYTHONPATH='')

            def replay():
                return subprocess.run(argv, cwd=root, env=environment, capture_output=True, text=True)

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual(counts['inventory_rows'], 92)
            self.assertEqual(counts['raw_nonblank_rows'], 150)
            self.assertEqual(counts['signature_rows'], 124)
            self.assertEqual(counts['meaningful_closures'], 0)
            self.assertEqual(counts['runtime_observations'], 0)
            for path, content in [
                ('data/patch-api/sources/2.1.0-page-coverage.json', {'source_rows': []}),
                ('data/patch-api/sources/2.2.0-wikitext-register.json', {'entries': []}),
                ('tests/data/patch_2_1_0_sweep_known_gaps.json', []),
            ]:
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(json.dumps(content))
            drift = replay()
            self.assertEqual(drift.returncode, 0, drift.stderr)
            self.assertEqual(drift.stdout, clean.stdout)
            for name in [*manifest['sealed_files'], 'historical-inputs.json']:
                with self.subTest(serialized_tamper=name):
                    target = here / name
                    original = target.read_bytes()
                    try:
                        target.write_bytes(original + b'\nTAMPER\n')
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        expected = 'historical manifest seal' if name == 'historical-inputs.json' else 'sealed input: ' + name
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
                missing = replay()
                self.assertNotEqual(missing.returncode, 0)
                self.assertIn('historical-known-gaps.json', missing.stderr)
            finally:
                target.write_bytes(original)
            self.assertEqual(target.read_bytes(), original)
            self.assertEqual(replay().stdout, clean.stdout)
            print(json.dumps({'copied_replay': 'PASS', 'no_git_target_current': True,
                              'tamper_restore_controls': len(manifest['sealed_files']) + 1,
                              'missing_gap_restoration': 'PASS', 'future_drift': 'identical',
                              'counts': counts}, sort_keys=True))


if __name__ == '__main__':
    unittest.main()
