"""Fresh copied historical proof; no runtime/native acceptance."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/2.4.0-source-2026-10-09'


def replay(directory):
    return subprocess.run([sys.executable, '-B', str(directory / 'validate.py')],
                          cwd=directory, env=dict(os.environ, PATH=''),
                          capture_output=True, text=True)


class ReplayTests(unittest.TestCase):
    def test_fresh_copy_without_git_target_or_current_sources(self):
        with tempfile.TemporaryDirectory() as temporary:
            copy = Path(temporary) / 'historical'
            shutil.copytree(EVIDENCE, copy)
            self.assertFalse((copy / '.git').exists())
            self.assertFalse((copy / 'target').exists())
            result = replay(copy)
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads(result.stdout)
            self.assertEqual(summary['source_rows'], 99)
            self.assertEqual(summary['inventory_rows'], 50)
            self.assertEqual(summary['meaningful_closures'], 0)
            self.assertEqual(summary['native_parity'], False)

    def test_serialized_tamper_rejects_then_restores(self):
        with tempfile.TemporaryDirectory() as temporary:
            copy = Path(temporary) / 'historical'
            shutil.copytree(EVIDENCE, copy)
            clean = replay(copy)
            self.assertEqual(clean.returncode, 0, clean.stderr)
            controls = json.loads((copy / 'tamper-targets.json').read_text())
            self.assertGreaterEqual(len(controls), 6)
            for name in controls:
                with self.subTest(path=name):
                    path = copy / name
                    original = path.read_bytes()
                    path.write_bytes(original + b'\nTAMPER\n')
                    rejected = replay(copy)
                    self.assertNotEqual(rejected.returncode, 0)
                    self.assertIn(name, rejected.stderr)
                    path.write_bytes(original)
                    restored = replay(copy)
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(restored.stdout, clean.stdout)

    def test_omitted_literal_line_cannot_become_closure(self):
        with tempfile.TemporaryDirectory() as temporary:
            copy = Path(temporary) / 'historical'
            shutil.copytree(EVIDENCE, copy)
            # A serialized edited ledger remains invalid; no current regenerated
            # file can silently replace the sealed original proof.
            path = copy / 'original/ledger.json'
            self.assertTrue(path.exists(), 'missing serialized original ledger')
            ledger = json.loads(path.read_text())
            ledger['source_rows'] = [row for row in ledger['source_rows']
                                     if row['wikitext_line'] != 70]
            path.write_text(json.dumps(ledger))
            result = replay(copy)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('original/ledger.json', result.stderr)


if __name__ == '__main__':
    unittest.main()
