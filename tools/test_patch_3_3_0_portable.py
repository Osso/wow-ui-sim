"""Own historical replay and serialized sealed-input negative controls."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / 'data/patch-api/evidence/3.3.0-session-2026-10-09'


class PortableTests(unittest.TestCase):
    def test_fresh_process_without_git_target_or_current_sources_and_tamper_restore(self):
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory) / 'proof'
            shutil.copytree(EVIDENCE, copied)
            command = [sys.executable, '-B', str(copied / 'validate.py')]
            environment = dict(os.environ, PATH='/nonexistent', PYTHONPATH='')

            def invoke():
                return subprocess.run(command, cwd=ROOT, env=environment,
                                      capture_output=True, text=True)

            clean = invoke()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            summary = json.loads(clean.stdout)
            self.assertEqual(summary['ledger_rows'], sum(summary['ledger_statuses'].values()))
            self.assertEqual(summary['publication_rows'],
                             summary['publication_ok'] + summary['publication_gaps'])
            self.assertGreater(summary['negative_gaps'], summary['publication_gaps'])
            for name in ['source-response.json', 'own-sweep-green.log',
                         'historical-page-coverage.json', 'historical-blobs.json.gz',
                         'historical-inputs.json']:
                path = copied / name
                original = path.read_bytes()
                try:
                    path.write_bytes(original + b'\nTAMPER')
                    tampered = invoke()
                    self.assertNotEqual(tampered.returncode, 0, name)
                    self.assertIn('seal', tampered.stderr, name)
                finally:
                    path.write_bytes(original)
                restored = invoke()
                self.assertEqual(restored.returncode, 0, restored.stderr)
                self.assertEqual(json.loads(restored.stdout), summary)


if __name__ == '__main__':
    unittest.main()
