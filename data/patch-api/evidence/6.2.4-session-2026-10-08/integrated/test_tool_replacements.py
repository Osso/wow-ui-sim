"""The 7.0.3 gate accepts exact opt-in additions, not arbitrary input drift."""
import hashlib
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[5]
VALIDATOR = ROOT / 'data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/validate.py'
spec = importlib.util.spec_from_file_location('p703_gate', VALIDATOR)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class ToolReplacementTests(unittest.TestCase):
    def test_exact_replacements_reject_tampering_wrong_digest_and_unrelated_path(self):
        paths = ('tools/extract_patch_non_inventory.py', 'tools/gen_patch_wikitext_register.py',
                 'tools/test_extract_patch_non_inventory.py', 'tools/test_gen_patch_wikitext_register.py')
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'tools').mkdir()
            for name in paths:
                old = subprocess.check_output(['git', 'show', '846a30663:' + name], cwd=ROOT)
                new = subprocess.check_output(['git', 'show', '0e40aff7e:' + name], cwd=ROOT)
                recorded = hashlib.sha256(old).hexdigest()
                target = root / name
                for accepted in (old, new):
                    target.write_bytes(accepted)
                    self.assertTrue(gate.scope_input_matches(name, recorded, root), name)
                target.write_bytes(new + b'\n')
                self.assertFalse(gate.scope_input_matches(name, recorded, root), name)
                target.write_bytes(new)
                self.assertFalse(gate.scope_input_matches(name, '0' * 64, root), name)
                other = root / 'tools/unrelated.py'
                other.write_bytes(new)
                self.assertFalse(gate.scope_input_matches('tools/unrelated.py', recorded, root), name)


if __name__ == '__main__':
    unittest.main()
