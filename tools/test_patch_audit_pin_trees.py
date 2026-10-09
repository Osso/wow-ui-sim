"""Exercise compact content proofs against real Git objects without old commits."""
import copy
import hashlib
from pathlib import Path
import subprocess
import tempfile
import unittest

from patch_audit_pin_trees import ProofGit, expand_pinned_inputs, verify_pinned_inputs, tree_id


class PinTreeTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.here = self.root / 'proof'
        self.here.mkdir()
        self.git('init', '-q')
        (self.root / 'src').mkdir()
        (self.root / 'src/a.txt').write_bytes(b'original\n')
        (self.root / 'src/b.txt').write_bytes(b'unchanged\n')
        self.commit()
        original_tree = self.git('rev-parse', 'HEAD:src').strip()
        original_blob = self.git('rev-parse', 'HEAD:src/a.txt').strip()
        (self.root / 'src/a.txt').write_bytes(b'rebased\n')
        (self.root / 'src/new.txt').write_bytes(b'unrelated addition\n')
        self.commit()
        revision = self.git('rev-parse', 'HEAD').strip()
        (self.here / 'original.txt').write_bytes(b'original\n')
        self.scope = {
            'recorded_revision': '0' * 40,  # Deliberately unavailable; only hashes are needed.
            'rebased_revision': revision,
            'inventories': {'src': {'recorded_tree': original_tree,
                                   'rebased_tree': self.git('rev-parse', 'HEAD:src').strip(),
                                   'excluded': ['new.txt'],
                                   'overrides': {'a.txt': ['100644', 'blob', original_blob]}}},
            'trees': {'src': {'rebased_tree': self.git('rev-parse', 'HEAD:src').strip(),
                              'recorded_tree': original_tree, 'excluded': ['new.txt']}},
            'inputs': {'src/a.txt': {'recorded_blob': original_blob,
                                    'rebased_blob': self.git('rev-parse', 'HEAD:src/a.txt').strip(),
                                    'sha256': hashlib.sha256(b'original\n').hexdigest(),
                                    'historical_file': 'original.txt'}},
        }

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, text=True)

    def commit(self):
        self.git('add', 'src')
        self.git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                 '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null',
                 'commit', '-qm', 'Fixture')

    def test_tree_hash_matches_git_with_modes_and_nested_paths(self):
        (self.root / 'src/nested').mkdir()
        (self.root / 'src/nested/run').write_bytes(b'execute\n')
        (self.root / 'src/nested/run').chmod(0o755)
        (self.root / 'src/link').symlink_to('a.txt')
        (self.root / 'src/nested.txt').write_bytes(b'sort before directory\n')
        self.commit()
        entries = {}
        for line in self.git('ls-tree', '-r', 'HEAD:src').splitlines():
            metadata, path = line.split('\t')
            mode, kind, oid = metadata.split()
            entries[path] = (mode, kind, oid)
        self.assertEqual(tree_id(entries), self.git('rev-parse', 'HEAD:src').strip())

    def test_sparse_tree_and_historical_exception_pass_without_old_revision(self):
        verify_pinned_inputs(self.root, self.here, [self.scope])

    def test_expansion_preserves_exact_names_blobs_and_historical_bytes(self):
        expanded = expand_pinned_inputs(self.root, self.here, [self.scope])[0]
        self.assertEqual(expanded['inventories']['src'], ['src/a.txt', 'src/b.txt'])
        self.assertEqual(set(expanded['inputs']), {'src/a.txt', 'src/b.txt'})
        self.assertEqual(expanded['inputs']['src/a.txt'], self.scope['inputs']['src/a.txt'])
        self.assertEqual(expanded['inputs']['src/b.txt']['sha256'],
                         hashlib.sha256(b'unchanged\n').hexdigest())
        self.assertEqual(expanded['inputs']['src/b.txt']['recorded_blob'],
                         self.git('rev-parse', 'HEAD:src/b.txt').strip())

    def test_historical_git_label_uses_reachable_tree_not_commit_object(self):
        alias = 'f' * 40
        original_blob = self.scope['inputs']['src/a.txt']['recorded_blob']
        old_entries = {'src/a.txt': ('100644', 'blob', original_blob),
                       'src/b.txt': ('100644', 'blob', self.git('rev-parse', 'HEAD:src/b.txt').strip())}
        pin = {'revision': self.scope['rebased_revision'],
               'snapshot': {'rebased_tree': self.git('rev-parse', 'HEAD^{tree}').strip(),
                            'recorded_tree': tree_id(old_entries),
                            'excluded': ['src/new.txt'],
                            'overrides': {'src/a.txt': ['100644', 'blob', original_blob]}}}
        proof = ProofGit(self.root, self.here, {'revision_pins': {alias: pin}, 'archived_blobs': {}})
        self.assertEqual(proof('show', alias + ':src/a.txt'), b'original\n')
        self.assertEqual(proof('rev-parse', alias + ':src'),
                         (self.scope['trees']['src']['recorded_tree'] + '\n').encode())
        self.assertEqual(proof('diff', '--name-only', alias, 'HEAD', '--', 'src'),
                         b'src/a.txt\nsrc/new.txt\n')
        pin['snapshot']['recorded_tree'] = '0' * 40
        with self.assertRaises(AssertionError):
            ProofGit(self.root, self.here, {'revision_pins': {alias: pin}, 'archived_blobs': {}})('show', alias + ':src/a.txt')

    def test_recorded_content_pin_tampering_is_rejected(self):
        mutated = copy.deepcopy(self.scope)
        mutated['trees']['src']['recorded_tree'] = '0' * 40
        with self.assertRaises(AssertionError):
            verify_pinned_inputs(self.root, self.here, [mutated])

    def test_rebased_directory_pin_tampering_is_rejected(self):
        mutated = copy.deepcopy(self.scope)
        mutated['trees']['src']['rebased_tree'] = '0' * 40
        with self.assertRaises(AssertionError):
            verify_pinned_inputs(self.root, self.here, [mutated])

    def test_inclusion_scope_tampering_is_rejected(self):
        mutated = copy.deepcopy(self.scope)
        mutated['trees']['src']['excluded'] = ['new.txt', 'b.txt']
        with self.assertRaises(AssertionError):
            verify_pinned_inputs(self.root, self.here, [mutated])

    def test_historical_bytes_tampering_is_rejected(self):
        (self.here / 'original.txt').write_bytes(b'tampered\n')
        with self.assertRaises(AssertionError):
            verify_pinned_inputs(self.root, self.here, [self.scope])

    def test_individual_rebased_blob_tampering_is_rejected(self):
        mutated = copy.deepcopy(self.scope)
        mutated['inputs']['src/a.txt']['rebased_blob'] = '0' * 40
        with self.assertRaises(AssertionError):
            verify_pinned_inputs(self.root, self.here, [mutated])


if __name__ == '__main__':
    unittest.main()
