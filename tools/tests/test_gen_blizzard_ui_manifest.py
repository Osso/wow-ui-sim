from __future__ import annotations

import json
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools import gen_blizzard_ui_manifest as generator


class PtrManifestTests(unittest.TestCase):
    def test_refresh_selects_pinned_revision_not_branch_tip(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            upstream = root / "upstream"
            upstream.mkdir()
            def git(*args: str) -> str:
                return subprocess.check_output(["git", "-C", str(upstream), *args], text=True).strip()
            git("init", "-b", "ptr")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            addons = upstream / "Interface/AddOns/Example"
            addons.mkdir(parents=True)
            (addons / "Selected.lua").write_text("return 70493\n")
            git("add", ".")
            git("commit", "-m", "selected")
            selected = git("rev-parse", "HEAD")
            (addons / "Future.lua").write_text("return 99999\n")
            git("add", ".")
            git("commit", "-m", "future")
            with mock.patch.object(generator, "GETHE_REMOTE", str(upstream)), mock.patch.object(
                generator, "GIT_ROOT", root / "git"
            ), mock.patch.object(generator, "CACHE_ROOT", root / "cache"):
                generator.refresh_branch("ptr", selected)
                self.assertEqual(generator.enumerate_files("ptr"), ["Example/Selected.lua"])

    def test_ptr_enumerates_ptr_not_ptr2(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for branch, filename in (("ptr", "New.lua"), ("ptr2", "Stale.lua")):
                addons = root / branch / "Interface/AddOns/Blizzard_Example"
                addons.mkdir(parents=True)
                (addons / filename).write_text("return true\n")
            output = root / "repo/data/blizzard-ui-files"
            output.mkdir(parents=True)
            with mock.patch.object(generator, "CACHE_ROOT", root), mock.patch.object(
                generator, "REPO_ROOT", root / "repo"
            ), mock.patch.object(sys, "argv", ["generator", "ptr", "--no-refresh"]):
                generator.main()
            self.assertEqual(
                (output / "ptr.txt").read_text(), "Blizzard_Example/New.lua\n"
            )


class RetailManifestTests(unittest.TestCase):
    def test_retail_manifest_uses_pinned_revision_not_branch_tip(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            upstream = root / "upstream"
            upstream.mkdir()
            def git(*args: str) -> str:
                return subprocess.check_output(["git", "-C", str(upstream), *args], text=True).strip()
            git("init", "-b", "live")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            addons = upstream / "Interface/AddOns/Example"
            addons.mkdir(parents=True)
            (addons / "Selected.lua").write_text("return 70575\n")
            git("add", ".")
            git("commit", "-m", "selected")
            selected = git("rev-parse", "HEAD")
            (addons / "Future.lua").write_text("return 70688\n")
            git("add", ".")
            git("commit", "-m", "future")
            repo = root / "repo"
            (repo / "data/blizzard-ui-files").mkdir(parents=True)
            (repo / "data/blizzard-ui-builds").mkdir(parents=True)
            (repo / "data/blizzard-ui-builds/retail.json").write_text(
                json.dumps({"gethe_revision": selected})
            )
            with mock.patch.object(generator, "GETHE_REMOTE", str(upstream)), mock.patch.object(
                generator, "GIT_ROOT", root / "git"
            ), mock.patch.object(generator, "CACHE_ROOT", root / "cache"), mock.patch.object(
                generator, "REPO_ROOT", repo
            ), mock.patch.object(sys, "argv", ["generator", "retail"]):
                generator.main()
            self.assertEqual(
                (repo / "data/blizzard-ui-files/retail.txt").read_text(),
                "Example/Selected.lua\n",
            )


if __name__ == "__main__":
    unittest.main()
