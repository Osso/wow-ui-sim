"""Fresh copied no-Git historical replay and serialized tamper controls."""
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
CWD = '/home/osso/.worktrees/wow-ui-sim-p1143-page'
RECEIPTS = []


class Portable(unittest.TestCase):
    def setUp(self):
        self.assertTrue(OPTIONS.archive.is_file(), 'own replay archive absent')
        self.tmp = tempfile.TemporaryDirectory(dir=OPTIONS.scratch)
        self.addCleanup(self.tmp.cleanup)
        self.copied = Path(self.tmp.name)
        with tarfile.open(OPTIONS.archive, 'r:gz') as archive:
            self.assertTrue(all(not Path(m.name).is_absolute() and '..' not in Path(m.name).parts
                                for m in archive.getmembers()))
            archive.extractall(self.copied, filter='data')
        for name in ('.git', 'target', 'tools'):
            self.assertFalse((self.copied / name).exists())
        self.seals = json.loads((self.copied / 'seals.json').read_bytes())
        self.check_originals()

    def check_originals(self):
        for name, expected in self.seals.items():
            self.assertEqual(hashlib.sha256((self.copied / name).read_bytes()).hexdigest(), expected, name)

    def execute(self, *args):
        command = [sys.executable, '-B', *map(str, args)]
        result = subprocess.run(command, cwd=CWD, env=dict(os.environ, PATH='', PYTHONDONTWRITEBYTECODE='1'),
                                capture_output=True, text=True, timeout=30)
        RECEIPTS.append(dict(test=self.id(), command=command, cwd=CWD, path='empty',
                             exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr))
        return result

    def test_copied_source_and_default_bytes(self):
        replay = self.execute(self.copied / 'audit.py')
        self.assertEqual(replay.returncode, 0, replay.stderr)
        fixtures = self.execute(self.copied / 'test_source.py')
        self.assertEqual(fixtures.returncode, 0, fixtures.stderr)
        self.assertIn('Ran 8 tests', fixtures.stderr)
        output = self.copied / 'reproduced.json'
        generated = self.execute(self.copied / 'historical-tools/gen_patch_wikitext_register.py',
                                 '1.14.3', self.copied / 'original/source.wikitext', '4615755', output)
        self.assertEqual(generated.returncode, 0, generated.stderr)
        self.assertEqual(output.read_bytes(), (self.copied / 'original/default-register.json').read_bytes())
        self.check_originals()

    def reject_and_restore(self, name, changed):
        path = self.copied / name
        original = path.read_bytes()
        path.write_bytes(changed)
        try:
            rejected = self.execute(self.copied / 'audit.py')
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('seal: ' + name, rejected.stderr)
        finally:
            path.write_bytes(original)
        self.assertEqual(path.read_bytes(), original)
        self.check_originals()
        restored = self.execute(self.copied / 'audit.py')
        self.assertEqual(restored.returncode, 0, restored.stderr)
        RECEIPTS.append(dict(restored=name, sha256=hashlib.sha256(original).hexdigest(), original_seals=len(self.seals)))

    def test_disk_ledger_omission_rejects_and_restores(self):
        ledger = json.loads((self.copied / 'original/ledger.json').read_bytes())
        ledger['inventory_rows'].pop()
        self.reject_and_restore('original/ledger.json', (json.dumps(ledger, indent=2) + '\n').encode())

    def test_disk_log_fabrication_rejects_and_restores(self):
        self.reject_and_restore('green.log', b'Invented native/model parity: PASS\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--receipts', type=Path, required=True)
    OPTIONS = parser.parse_args()
    OPTIONS.scratch.mkdir(parents=True, exist_ok=True)
    assert not OPTIONS.receipts.exists(), 'refuse receipt overwrite'
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Portable))
    OPTIONS.receipts.write_text(json.dumps(dict(scope='copied SOURCE only', runs=RECEIPTS,
                                               tests=result.testsRun, failures=len(result.failures),
                                               errors=len(result.errors)), indent=2) + '\n')
    sys.exit(0 if result.wasSuccessful() else 1)
