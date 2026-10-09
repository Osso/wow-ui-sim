"""Fresh serialized historical replay with no Git/target/current source tree."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/2.4.2-session-2026-10-09'


class ReplayTests(unittest.TestCase):
    def run_copy(self, directory):
        return subprocess.run([sys.executable, '-B', str(directory / 'validate.py')],
                              cwd=directory, env=dict(os.environ, PATH='/nonexistent', PYTHONDONTWRITEBYTECODE='1'),
                              capture_output=True, text=True)

    def test_fresh_copy_replays_and_serialized_tampers_reject_then_restore(self):
        with tempfile.TemporaryDirectory() as parent:
            directory = Path(parent) / 'historical-only'
            shutil.copytree(EVIDENCE, directory)
            self.assertFalse((directory / '.git').exists())
            self.assertFalse((directory / 'target').exists())
            initial = self.run_copy(directory)
            self.assertEqual(initial.returncode, 0, initial.stderr)
            summary = json.loads(initial.stdout)
            print(json.dumps(summary, sort_keys=True))
            self.assertEqual(summary['ledger_rows'], 53)
            self.assertEqual(summary['publication_gaps'], 9)
            self.assertEqual(summary['native_acceptance'], 'unmeasured')
            for relative in ('historical-page-coverage.json', 'factory-green.log', 'historical-blobs.json.gz'):
                path = directory / relative
                original = path.read_bytes()
                if relative.endswith('.json'):
                    ledger = json.loads(original)
                    ledger['source_rows'][0]['note'] = 'serialized tamper'
                    altered = (json.dumps(ledger, indent=2) + '\n').encode()
                elif relative.endswith('.gz'):
                    altered = original[:-1] + bytes([original[-1] ^ 1])
                else:
                    altered = original + b'\nserialized receipt tamper\n'
                try:
                    path.write_bytes(altered)
                    rejected = self.run_copy(directory)
                    self.assertNotEqual(rejected.returncode, 0)
                    self.assertIn('seal: ' + relative, rejected.stderr)
                finally:
                    path.write_bytes(original)
                self.assertEqual(hashlib.sha256(path.read_bytes()).digest(), hashlib.sha256(original).digest())
                restored = self.run_copy(directory)
                self.assertEqual(restored.returncode, 0, restored.stderr)
                self.assertEqual(json.loads(restored.stdout), summary)


if __name__ == '__main__':
    unittest.main()
