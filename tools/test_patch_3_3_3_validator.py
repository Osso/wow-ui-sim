"""Own fresh-process historical replay; serialized tampering restores exact bytes."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = Path('data/patch-api/evidence/3.3.3-session-2026-10-09')


class HistoricalReplayTests(unittest.TestCase):
    def test_portable_replay_preserves_original_inputs_and_rejects_tampering(self):
        original = ROOT / EVIDENCE
        self.assertTrue((original / 'validate.py').is_file(),
                        'own historical validator is not implemented')
        with tempfile.TemporaryDirectory(dir=ROOT / 'tools') as directory:
            fixture = Path(directory) / 'detached-snapshot'
            here = fixture / EVIDENCE
            here.parent.mkdir(parents=True)
            shutil.copytree(original, here)
            self.assertFalse((fixture / '.git').exists())
            self.assertFalse((fixture / 'target').exists())
            self.assertFalse((fixture / 'src').exists())

            def replay():
                return subprocess.run([sys.executable, '-I', '-B', str(here / 'validate.py')],
                                      cwd=ROOT, capture_output=True, text=True)

            first = replay()
            self.assertEqual(first.returncode, 0, first.stderr)
            clean = json.loads(first.stdout)
            self.assertEqual(clean['publication_rows'], 36)
            self.assertEqual(clean['publication_ok'], 25)
            self.assertEqual(clean['publication_gaps'], 11)
            self.assertEqual(clean['negative_gaps'], 12)
            self.assertEqual(clean['ledger_rows'], 113)
            self.assertEqual(clean['signature_rows'], 36)
            self.assertEqual(clean['extract_rows'], 41)
            self.assertEqual(clean['prose_gaps'], 3)
            self.assertEqual(clean['modeled_scalar_reads'], 1)
            # Main may close current gaps later: originals are not mutable current files.
            current_ledger = fixture / 'data/patch-api/sources/3.3.3-page-coverage.json'
            current_ledger.parent.mkdir(parents=True, exist_ok=True)
            current_ledger.write_text('{"patch":"3.3.3","source_rows":[]}\n')
            current_gaps = fixture / 'tests/data/patch_3_3_3_sweep_known_gaps.json'
            current_gaps.parent.mkdir(parents=True, exist_ok=True)
            current_gaps.write_text('[]\n')
            self.assertEqual(json.loads(replay().stdout), clean)
            for name in ['historical-source.wikitext', 'source-response.json',
                         'own-sweep-green.txt', 'historical-page-coverage.json',
                         'historical-known-gaps.json', 'command-ledger.json',
                         'historical-inputs.json', 'historical-blobs.json.gz']:
                with self.subTest(serialized_input=name):
                    target = here / name
                    before = target.read_bytes()
                    try:
                        target.write_bytes(before + b'\nTAMPER\n')
                        failed = replay()
                        self.assertNotEqual(failed.returncode, 0, failed.stdout)
                        self.assertIn('seal', failed.stderr)
                    finally:
                        target.write_bytes(before)
                    self.assertEqual(target.read_bytes(), before)
                    restored = replay()
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(json.loads(restored.stdout), clean)
            target = here / 'historical-known-gaps.json'
            before = target.read_bytes()
            try:
                target.unlink()
                failed = replay()
                self.assertNotEqual(failed.returncode, 0)
            finally:
                target.write_bytes(before)
            self.assertEqual(json.loads(replay().stdout), clean)


if __name__ == '__main__':
    unittest.main()
