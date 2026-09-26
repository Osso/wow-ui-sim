import contextlib
import io
from pathlib import Path
import runpy
import sys
import tempfile
import unittest
from unittest.mock import patch


SOURCE = Path(__file__).resolve().parents[2] / "docs/addons/ServerSnapshot"
SCRIPT = SOURCE / "deploy.sh"


class ServerSnapshotDeployTest(unittest.TestCase):
    def invoke(self, root: Path):
        namespace = runpy.run_path(str(SCRIPT))
        with patch.object(sys, "argv", [str(SCRIPT), str(root)]):
            with contextlib.redirect_stdout(io.StringIO()):
                namespace["main"]()

    def test_installs_only_addon_files_and_preserves_other_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "AddOns"
            target = root / "ServerSnapshot"
            target.mkdir(parents=True)
            (target / "keep.txt").write_text("keep")
            other = root / "OtherAddon.lua"
            other.write_text("untouched")
            (target / "ServerSnapshot.lua").write_text("old addon")
            self.invoke(root)
            for name in ("ServerSnapshot.lua", "ServerSnapshot.toc"):
                self.assertEqual(
                    (SOURCE / name).read_bytes(), (target / name).read_bytes()
                )
            self.assertEqual((target / "keep.txt").read_text(), "keep")
            self.assertEqual(other.read_text(), "untouched")
            self.assertEqual(
                {path.name for path in target.iterdir()},
                {"ServerSnapshot.lua", "ServerSnapshot.toc", "keep.txt"},
            )
            self.invoke(root)
            self.assertEqual(
                (SOURCE / "ServerSnapshot.lua").read_bytes(),
                (target / "ServerSnapshot.lua").read_bytes(),
            )

    def test_missing_addons_root_fails_without_creating_install_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            missing = Path(directory) / "Missing" / "AddOns"
            errors = io.StringIO()
            with (
                contextlib.redirect_stderr(errors),
                self.assertRaises(SystemExit) as failure,
            ):
                self.invoke(missing)
            self.assertEqual(failure.exception.code, 2)
            self.assertIn("AddOns directory does not exist", errors.getvalue())
            self.assertFalse(missing.exists())


if __name__ == "__main__":
    unittest.main()
