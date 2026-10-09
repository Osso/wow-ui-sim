"""Exercise the gate in real repositories, including worktree cleanup."""
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

GATE = Path(__file__).with_name('check_patch_validators.py')


class CheckPatchValidatorsTests(unittest.TestCase):
    def run_gate(self, validator, ignored=False, revision='HEAD', evidence_size=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True)
            git('init', '-q')
            files = {
                'src/c_api/mod.rs': '// original\n',
                'tools/gen_patch_wikitext_register.py': '# original\n',
                'data/patch-api/sources/1.0.0-wikitext-register.json': '{"patch":"1.0.0","entries":[]}\n',
                'docs/wiki/log.md': '# Log\n',
                'data/patch-api/evidence/1.0.0-session/validate.py': validator,
                '.gitignore': 'scratch.txt\n',
            }
            for name, content in files.items():
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            if evidence_size is not None:
                (root / 'data/patch-api/evidence/1.0.0-session/large.bin').write_bytes(b'x' * evidence_size)
            self.assertTrue(GATE.is_file(), 'fresh-checkout gate has not been implemented')
            shutil.copyfile(GATE, root / 'tools/check_patch_validators.py')
            git('add', '.')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                'commit', '-qm', 'Fixture')
            head = git('rev-parse', 'HEAD').strip()
            if ignored:
                (root / 'scratch.txt').write_text('not evidence')
            result = subprocess.run(['python3', '-B', 'tools/check_patch_validators.py', revision],
                                    cwd=root, capture_output=True, text=True)
            self.assertEqual(git('rev-parse', 'HEAD').strip(), head)
            self.assertEqual(git('status', '--porcelain'), '')
            self.assertEqual(len(git('worktree', 'list', '--porcelain').split('worktree ')) - 1, 1)
            self.assertEqual(list(root.rglob('__pycache__')), [])
            return result, json.loads(result.stdout)

    def test_pinned_validator_passes_both_phases_and_new_validator_runs(self):
        validator = '''from pathlib import Path
import subprocess
root = Path(__file__).resolve().parents[4]
original = subprocess.check_output(['git', 'show', 'HEAD:src/c_api/mod.rs'], cwd=root)
# The synthetic commit's parent is the original revision.
revision = subprocess.check_output(['git', 'rev-list', '--max-parents=0', 'HEAD'], cwd=root).decode().strip()
assert subprocess.check_output(['git', 'show', revision + ':src/c_api/mod.rs'], cwd=root) == b'// original\\n'
'''
        result, report = self.run_gate(validator)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report['summary'], {'clean': {'passed': 1, 'failed': 0},
                                             'later_audit': {'passed': 2, 'failed': 0}})

    def test_ignored_scratch_dependency_fails_in_clean_worktree(self):
        result, report = self.run_gate("from pathlib import Path\nassert Path('scratch.txt').is_file()\n", ignored=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report['summary']['clean']['failed'], 1)

    def test_live_shared_file_dependency_fails_only_after_later_audit(self):
        result, report = self.run_gate("from pathlib import Path\nassert Path('src/c_api/mod.rs').read_text() == '// original\\n'\n")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(report['summary']['clean']['failed'], 0)
        self.assertEqual(report['summary']['later_audit']['failed'], 1)

    def test_oversized_tracked_evidence_fails_and_reports_path(self):
        result, report = self.run_gate('pass\n', evidence_size=5_000_001)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(report['oversized_evidence']['clean'],
                         [{'path': 'data/patch-api/evidence/1.0.0-session/large.bin',
                           'bytes': 5_000_001}])
        self.assertEqual(report['oversized_evidence']['later_audit'],
                         report['oversized_evidence']['clean'])

    def test_exact_size_limit_passes(self):
        result, report = self.run_gate('pass\n', evidence_size=5_000_000)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(report['oversized_evidence'], {'clean': [], 'later_audit': []})

    def test_invalid_revision_reports_error_and_leaves_no_worktree(self):
        result, report = self.run_gate('pass\n', revision='not-a-revision')
        self.assertEqual(result.returncode, 1)
        self.assertIn('error', report)


if __name__ == '__main__':
    unittest.main()
