import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release", Path(__file__).with_name("release.py"))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def test_checked_master_rejects_dirty_or_diverged_tree(self):
        for replies, message in [
            (["", "feature"], "requires master"),
            (["", "master", "?? untracked"], "clean tree"),
            (["", "master", "", "abc", "def"], "match origin/master"),
        ]:
            with self.subTest(message=message), patch.object(release, "git", side_effect=replies):
                with self.assertRaisesRegex(ValueError, message):
                    release.checked_master()

    def test_success_tags_verified_head_then_pushes_tag(self):
        with patch.object(release, "checked_master", return_value="abc"), \
             patch.object(release, "git") as git, \
             patch.object(release.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "PASS\n", "")):
            release.release("1.2.3", "/slot")
        self.assertEqual(git.call_args_list[0].args, ("tag", "-a", "v1.2.3", "-m", "v1.2.3", "abc"))
        self.assertEqual(git.call_args_list[1].args, ("push", "origin", "refs/tags/v1.2.3:refs/tags/v1.2.3"))

    def test_dry_run_executes_optional_suite_without_tagging(self):
        with patch.object(release, "checked_master", return_value="abc") as checked, \
             patch.object(release, "git") as git, \
             patch.object(release.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "PASS\n", "")) as run:
            release.release("1.2.3", "/slot", dry_run=True)
        self.assertEqual(checked.call_count, 2)
        self.assertIn("--optional", run.call_args.args[0])
        self.assertIn("abc", run.call_args.args[0])
        git.assert_not_called()

    def test_failure_never_tags(self):
        with patch.object(release, "checked_master", return_value="abc"), \
             patch.object(release, "git") as git, \
             patch.object(release.subprocess, "run", return_value=subprocess.CompletedProcess([], 1, "FAILED\n", "")):
            with self.assertRaisesRegex(ValueError, "full suite failed"):
                release.release("1.2.3", "/slot")
        git.assert_not_called()

    def test_master_change_never_tags(self):
        with patch.object(release, "checked_master", side_effect=["abc", "def"]), \
             patch.object(release, "git") as git, \
             patch.object(release.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "PASS\n", "")):
            with self.assertRaisesRegex(ValueError, "master changed"):
                release.release("1.2.3", "/slot")
        git.assert_not_called()

    def test_invalid_version_never_runs_suite(self):
        with patch.object(release, "checked_master") as checked:
            with self.assertRaises(ValueError):
                release.release("v1.2.3", "/slot")
        checked.assert_not_called()


if __name__ == "__main__":
    unittest.main()
