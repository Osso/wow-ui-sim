"""Fresh copied SOURCE replay and serialized seal controls, not final gates."""
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
ROOT = EVIDENCE.parents[3]
RECEIPTS = []

def digest(data):
    return hashlib.sha256(data).hexdigest()

def invoke(directory, *args):
    environment = {k: v for k, v in os.environ.items() if k not in ('PYTHONPATH', 'PYTHONHOME')}
    environment.update(PATH='/nonexistent', PYTHONNOUSERSITE='1')
    command = [sys.executable, '-B', *map(str, args)]
    result = subprocess.run(command, cwd=directory, env=environment, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=30)
    RECEIPTS.append({'argv': command, 'cwd': str(directory), 'PATH': environment['PATH'], 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
    return result

class PortableSource(unittest.TestCase):

    def setUp(self):
        archive = EVIDENCE / 'replay-archive.tar.gz'
        self.assertTrue(archive.is_file(), 'copied immutable SOURCE archive absent')
        scratch = ROOT / 'target/source-replay-1.15.7'
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temp.cleanup)
        self.copied = Path(self.temp.name)
        with tarfile.open(archive, 'r:gz') as handle:
            names = handle.getnames()
            self.assertEqual(len(names), len(set(names)))
            self.assertFalse(any((Path(n).is_absolute() or '..' in Path(n).parts for n in names)))
            self.assertFalse(any((set(Path(n).parts) & {'.git', 'target', '__pycache__', 'Interface'} for n in names)))
            handle.extractall(self.copied, filter='data')
        self.seals = json.loads((self.copied / 'seals.json').read_bytes())
        self.original_map = (self.copied / 'seals.json').read_bytes()
        self.assertEqual(self.original_map, (EVIDENCE / 'seals.json').read_bytes())
        for name, seal in self.seals.items():
            self.assertEqual(digest((self.copied / name).read_bytes()), seal, name)

    def test_fresh_git_free_source_and_default_generator_replay(self):
        result = invoke(self.copied, self.copied / 'audit.py')
        self.assertEqual(result.returncode, 0, result.stderr)
        totals = json.loads(result.stdout)
        self.assertEqual(totals['historical_seals'], len(self.seals))
        self.assertEqual(totals['totals']['contracts'], 2)
        self.assertEqual(totals['totals']['api_occurrences'], 0)
        result = invoke(self.copied, self.copied / 'test_source_accounting.py')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Ran 8 tests', result.stderr)
        output = self.copied / 'replayed-register.json'
        result = invoke(self.copied, self.copied / 'historical-tools/gen_patch_wikitext_register.py', '1.15.7', self.copied / 'source.wikitext', '6778069', output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output.read_bytes(), (self.copied / 'default-register.json').read_bytes())
        self.assertEqual(json.loads(output.read_bytes())['entries'], [])

    def reject_and_restore(self, name, changed):
        path = self.copied / name
        original = path.read_bytes()
        path.write_bytes(changed)
        try:
            result = invoke(self.copied, self.copied / 'audit.py')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(f'seal: {name}', result.stderr)
            RECEIPTS.append({'tampered_file': name, 'original_sha256': digest(original), 'tampered_sha256': digest(changed), 'rejected': True})
        finally:
            path.write_bytes(original)
        self.assertEqual(path.read_bytes(), original)
        self.assertEqual(digest(path.read_bytes()), self.seals[name])
        self.assertEqual((self.copied / 'seals.json').read_bytes(), self.original_map)
        result = invoke(self.copied, self.copied / 'audit.py')
        self.assertEqual(result.returncode, 0, result.stderr)
        RECEIPTS.append({'restored_file': name, 'restored_sha256': digest(path.read_bytes()), 'original_seals_sha256': digest(self.original_map)})

    def test_serialized_ledger_omission_rejected_then_restored(self):
        changed = json.loads((self.copied / 'ledger.json').read_bytes())
        changed['contracts'].pop()
        self.reject_and_restore('ledger.json', (json.dumps(changed, indent=2) + '\n').encode())

    def test_serialized_log_fabrication_rejected_then_restored(self):
        original = (self.copied / 'green.log').read_bytes()
        self.reject_and_restore('green.log', original + b'fabricated native parity PASS\n')
if __name__ == '__main__':
    outcome = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(PortableSource))
    if len(sys.argv) == 2:
        destination = Path(sys.argv[1])
        assert not destination.exists(), 'refuse receipt overwrite'
        destination.write_text(json.dumps({'scope': 'fresh copied SOURCE-only development controls', 'success': outcome.wasSuccessful(), 'tests': outcome.testsRun, 'receipts': RECEIPTS}, indent=2) + '\n')
    sys.exit(not outcome.wasSuccessful())
