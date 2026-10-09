"""Behavioral historical-validator fixtures; tamper only disposable copies."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
HERE = ROOT / 'data/patch-api/evidence/4.0.1-session-2026-10-09'
VALIDATOR = HERE / 'validate.py'


class HistoricalValidatorTests(unittest.TestCase):
    def run_validator(self, evidence):
        return subprocess.run([sys.executable, '-B', str(VALIDATOR), '--evidence', str(evidence)],
                              cwd=ROOT, capture_output=True, text=True,
                              env={"PATH": "/p401-no-git"})

    def copied_evidence(self, directory):
        context = json.loads((HERE / 'historical-context.json').read_text())
        names = set(context['evidence_sha256']) | set(context['archives']) | {'historical-context.json'}
        copy = Path(directory) / 'evidence'
        copy.mkdir()
        for name in names:
            shutil.copyfile(HERE / name, copy / name)
        return copy

    def test_sealed_recorded_scope_validates_without_git_commands(self):
        result = self.run_validator(HERE)
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout.split('PASS: ', 1)[1])
        register = json.loads((ROOT / 'data/patch-api/sources/4.0.1-wikitext-register.json').read_text())
        results = json.loads((HERE / 'publication-green-results.json').read_text())
        self.assertEqual(summary['inventory'], len(register['entries']))
        self.assertEqual(summary['publication_gaps'], sum(not row['ok'] for row in results.values()))

    def test_source_response_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'target') as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / 'source-response.json'
            path.write_bytes(path.read_bytes() + b' ')
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('evidence seal: source-response.json', result.stderr)

    def test_own_green_log_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'target') as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / 'retail-prefork-green.log'
            path.write_bytes(path.read_bytes() + b'\ntampered\n')
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('evidence seal: retail-prefork-green.log', result.stderr)

    def test_historical_archive_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'target') as directory:
            evidence = self.copied_evidence(directory)
            path = evidence / 'historical-inputs.json.gz'
            path.write_bytes(path.read_bytes() + b'bad')
            result = self.run_validator(evidence)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('archive seal: historical-inputs.json.gz', result.stderr)

    def test_unrelated_later_inventory_does_not_change_recorded_scope(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'target') as directory:
            evidence = self.copied_evidence(directory)
            (evidence / '99.0.0-wikitext-register.json').write_text('{"entries":[]}')
            result = self.run_validator(evidence)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
