"""Fresh-process own historical source replay; not a current/native acceptance gate."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/3.1.0-session-2026-10-09'


class HistoricalReplayTests(unittest.TestCase):
    def replay(self, evidence):
        return subprocess.run([sys.executable, '-B', str(evidence / 'validate.py')],
                              cwd=ROOT, env=dict(os.environ, PATH='/no-git-or-project-tools'),
                              capture_output=True, text=True)

    def test_relocation_and_future_current_changes_do_not_expand_scope(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = root / 'frozen'
            shutil.copytree(EVIDENCE, evidence)
            self.assertFalse((root / '.git').exists())
            self.assertFalse((root / 'target').exists())
            result = self.replay(evidence)
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads(result.stdout)
            self.assertEqual(summary['publication_rows'], 110)
            self.assertEqual(summary['raw_nonblank'], 189)
            self.assertEqual(summary['signature_fragments'], 33)
            coverage = json.loads((evidence / 'historical-page-coverage.json').read_bytes())
            rows = {row['source_id']: row for row in coverage['source_rows']}
            self.assertEqual(rows['raw-line-013']['status'], 'audit-pending')
            self.assertIn('unnamed restored methods', rows['raw-line-013']['note'])
            self.assertIn('controller fallback', rows['raw-line-171']['note'])
            self.assertEqual(summary['modeled_closures'], 0)
            (root / 'current-page-coverage.json').write_text('{"all-native-contracts":"closed"}')
            (root / 'current-known-gaps.json').write_text('[]')
            (root / '3.3.5-wikitext-register.json').write_text('{"future":"closures"}')
            later = self.replay(evidence)
            self.assertEqual(later.returncode, 0, later.stderr)
            self.assertEqual(later.stdout, result.stdout)

    def test_tampered_sealed_inputs_fail_then_restore_exact_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / 'frozen'
            shutil.copytree(EVIDENCE, evidence)
            for name in ('historical-source.wikitext', 'own-sweep-green.log',
                         'historical-page-coverage.json', 'historical-known-gaps.json',
                         'historical-bundle.json.gz', 'historical-inputs.json'):
                with self.subTest(name=name):
                    path = evidence / name
                    original = path.read_bytes()
                    path.write_bytes(original + b'\nTAMPER')
                    result = self.replay(evidence)
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn('seal', result.stderr)
                    path.write_bytes(original)
                    self.assertEqual(path.read_bytes(), original)
            restored = self.replay(evidence)
            self.assertEqual(restored.returncode, 0, restored.stderr)


if __name__ == '__main__':
    unittest.main()
