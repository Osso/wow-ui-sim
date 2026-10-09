"""Copied historical replay and serialized seal controls; SOURCE only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
EVIDENCE = Path(__file__).resolve().parent
RECEIPTS = []

def digest(data):
    return hashlib.sha256(data).hexdigest()

class PortableSource(unittest.TestCase):

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(dir=OPTIONS.scratch)
        self.addCleanup(self.directory.cleanup)
        self.copied = Path(self.directory.name)
        self.assertTrue(OPTIONS.archive.is_file(), 'own replay archive absent')
        with tarfile.open(OPTIONS.archive, 'r:gz') as archive:
            members = archive.getnames()
            self.assertTrue(all((not Path(m).is_absolute() and '..' not in Path(m).parts for m in members)))
            archive.extractall(self.copied, filter='data')
        self.assertFalse((self.copied / '.git').exists())
        self.assertFalse((self.copied / 'target').exists())
        self.assertFalse((self.copied / 'tools').exists())
        self.seals = json.loads((self.copied / 'seals.json').read_bytes())
        self.check_originals()

    def check_originals(self):
        for name, expected in self.seals.items():
            self.assertEqual(digest((self.copied / name).read_bytes()), expected, name)

    def execute(self, *args):
        command = [sys.executable, '-B', *map(str, args)]
        result = subprocess.run(command, cwd=self.copied, env=dict(os.environ, PATH='', PYTHONDONTWRITEBYTECODE='1'), capture_output=True, text=True, timeout=30)
        RECEIPTS.append({'test': self.id(), 'command': command, 'cwd': str(self.copied), 'path': 'empty; Git/current commands unavailable', 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
        return result

    def test_copied_source_and_default_register(self):
        result = self.execute(self.copied / 'audit.py')
        self.assertEqual(result.returncode, 0, result.stderr)
        fixtures = self.execute(self.copied / 'test_source_accounting.py')
        self.assertEqual(fixtures.returncode, 0, fixtures.stderr)
        self.assertIn('Ran 8 tests', fixtures.stderr)
        output = self.copied / 'reproduced-register.json'
        generated = self.execute(self.copied / 'historical-tools/gen_patch_wikitext_register.py', '1.13.4', self.copied / 'source.wikitext', '3216451', output)
        self.assertEqual(generated.returncode, 0, generated.stderr)
        self.assertEqual(output.read_bytes(), (self.copied / 'default-register.json').read_bytes())
        extracted = self.execute(self.copied / 'audit.py', 'default-extract')
        self.assertEqual(extracted.returncode, 0, extracted.stderr)
        self.assertEqual(extracted.stdout.encode(), (self.copied / 'default-extract.txt').read_bytes())
        self.check_originals()

    def reject_and_restore(self, name, changed):
        path = self.copied / name
        original = path.read_bytes()
        path.write_bytes(changed)
        try:
            result = self.execute(self.copied / 'audit.py')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(f'seal: {name}', result.stderr)
        finally:
            path.write_bytes(original)
        self.assertEqual(path.read_bytes(), original)
        self.check_originals()
        restored = self.execute(self.copied / 'audit.py')
        self.assertEqual(restored.returncode, 0, restored.stderr)
        RECEIPTS.append({'restored': name, 'sha256': digest(path.read_bytes()), 'all_original_seals': len(self.seals)})

    def test_serialized_ledger_omission_rejected_and_restored(self):
        ledger = json.loads((self.copied / 'ledger.json').read_bytes())
        ledger['contracts'].pop()
        self.reject_and_restore('ledger.json', (json.dumps(ledger, indent=2) + '\n').encode())

    def test_serialized_log_fabrication_rejected_and_restored(self):
        self.reject_and_restore('source-green.log', b'Invented runtime/native parity: PASS\n')
if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--receipts', type=Path, required=True)
    OPTIONS = parser.parse_args()
    OPTIONS.scratch.mkdir(parents=True, exist_ok=True)
    assert not OPTIONS.receipts.exists(), 'refuse receipt overwrite'
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(PortableSource)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    OPTIONS.receipts.write_text(json.dumps({'scope': 'copied historical SOURCE; no runtime/model/native credit', 'runs': RECEIPTS, 'tests_run': result.testsRun, 'failures': len(result.failures), 'errors': len(result.errors)}, indent=2) + '\n')
    sys.exit(0 if result.wasSuccessful() else 1)
