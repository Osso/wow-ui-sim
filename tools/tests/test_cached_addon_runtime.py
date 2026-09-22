import hashlib
import importlib.util
import json
import stat
import tempfile
import unittest
import zipfile
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / "cached_addon_runtime.py"
HELPER = None
if SOURCE.exists():
    SPEC = importlib.util.spec_from_file_location("cached_addon_runtime", SOURCE)
    HELPER = importlib.util.module_from_spec(SPEC)
    SPEC.loader.exec_module(HELPER)


class CachedAddonRuntimeTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(HELPER, "cached addon staging helper is not implemented")
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.archive = self.root / "101.zip"
        self.staging = self.root / "isolated"
        self.repo_addons = self.root / "repo-addons"
        self.repo_addons.mkdir()
        for name in (
            "Admin",
            "SimCommands",
            "TestFramework",
            "Blizzard_FrameXML",
            "Other",
        ):
            (self.repo_addons / name).mkdir()

    def make_zip(self, members=None):
        members = members or {
            "Example/Example.toc": "## Title: Example\nMain.lua\n",
            "Example/Main.lua": "ExampleLoaded = true\n",
        }
        with zipfile.ZipFile(self.archive, "w") as archive:
            for name, content in members.items():
                archive.writestr(name, content)
        return hashlib.sha256(self.archive.read_bytes()).hexdigest()

    def stage(self, digest):
        return HELPER.stage_package(
            self.archive, digest, self.staging, self.repo_addons
        )

    def test_selection_preserves_project_with_only_comparison_archive(self):
        archive = {"fileId": 101, "status": "downloaded", "sha256": "abc", "bytes": 3}
        rows = HELPER.select_cached_projects(
            {
                "projects": [
                    {"slug": "ready", "projectId": 1, "archives": {"forever": archive}},
                    {
                        "slug": "blocked",
                        "projectId": 2,
                        "archives": {
                            "forever": {"status": "download-error"},
                            "nonForever": archive,
                        },
                    },
                ]
            }
        )
        self.assertEqual([row["slug"] for row in rows], ["ready", "blocked"])
        self.assertEqual(rows[0]["archive"], archive)
        self.assertEqual(rows[0]["status"], "ready")
        self.assertEqual(rows[1]["status"], "blocked")
        self.assertIsNone(rows[1]["archive"])
        self.assertIn("Forever", rows[1]["reason"])

    def test_stage_preserves_archive_and_writes_isolated_content_and_enable_state(self):
        digest = self.make_zip(
            {
                "Example/Main.lua": "return 42\n",
                "Example/Example.toc": "Main.lua",
                "Options/Options.toc": "## LoadOnDemand: 1",
            }
        )
        staged = self.stage(digest)
        self.assertEqual(staged["roots"], ["Example", "Options"])
        self.assertEqual(
            (self.staging / "Interface/AddOns/Example/Main.lua").read_text(),
            "return 42\n",
        )
        self.assertEqual(hashlib.sha256(self.archive.read_bytes()).hexdigest(), digest)
        self.assertTrue((self.staging / "fake-install").is_dir())
        self.assertTrue((self.staging / "wtf").is_dir())
        states = dict(
            line.split(": ")
            for line in (self.staging / "AddOns.txt").read_text().splitlines()
        )
        self.assertEqual(
            states,
            {
                "Admin": "enabled",
                "SimCommands": "enabled",
                "TestFramework": "enabled",
                "Blizzard_FrameXML": "enabled",
                "Other": "disabled",
                "Example": "enabled",
                "Options": "enabled",
            },
        )
        self.assertEqual(self.stage(digest), staged)
        self.assertTrue((self.staging / "load-observer.lua").is_file())

    def test_backslash_directory_members_stage_as_directories(self):
        with zipfile.ZipFile(self.archive, "w") as archive:
            directory = zipfile.ZipInfo("AnimatedBlizzPortrait\\media\\")
            archive.writestr(directory, b"")
            directory.external_attr = 0
            archive.writestr("AnimatedBlizzPortrait\\media\\icon.txt", b"icon bytes")
            archive.writestr(
                "AnimatedBlizzPortrait\\AnimatedBlizzPortrait.toc", b"Main.lua"
            )
        with zipfile.ZipFile(self.archive) as archive:
            directory = archive.infolist()[0]
            self.assertEqual(directory.external_attr, 0)
            self.assertEqual(directory.file_size, 0)
            self.assertFalse(directory.is_dir())
        digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        staged = self.stage(digest)
        self.assertEqual(staged["roots"], ["AnimatedBlizzPortrait"])
        media = self.staging / "Interface/AddOns/AnimatedBlizzPortrait/media"
        self.assertTrue(media.is_dir())
        self.assertEqual((media / "icon.txt").read_bytes(), b"icon bytes")
        self.assertEqual(hashlib.sha256(self.archive.read_bytes()).hexdigest(), digest)

    def test_hash_mismatch_does_not_create_staging_root(self):
        self.make_zip()
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            self.stage("0" * 64)
        self.assertFalse(self.staging.exists())

    def test_unsafe_members_rejected_before_any_write(self):
        for unsafe in (
            "../outside.lua",
            "/absolute.lua",
            "Example/../../escape.lua",
            "C:\\outside.lua",
            "..\\outside.lua",
        ):
            with self.subTest(member=unsafe):
                digest = self.make_zip({"Example/good.lua": "ok", unsafe: "bad"})
                with self.assertRaises(ValueError):
                    self.stage(digest)
                self.assertFalse(self.staging.exists())

    def test_archive_symlink_rejected(self):
        link = zipfile.ZipInfo("Example/link")
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr(link, "../../outside")
        digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.stage(digest)
        self.assertFalse(self.staging.exists())

    def test_existing_content_conflict_does_not_overwrite_or_partially_extract(self):
        digest = self.make_zip(
            {"Example/new.lua": "new", "Example/Main.lua": "changed"}
        )
        target = self.staging / "Interface/AddOns/Example/Main.lua"
        target.parent.mkdir(parents=True)
        target.write_text("original")
        with self.assertRaisesRegex(ValueError, "conflict"):
            self.stage(digest)
        self.assertEqual(target.read_text(), "original")
        self.assertFalse(target.with_name("new.lua").exists())

    def test_existing_symlink_cannot_redirect_extraction(self):
        digest = self.make_zip()
        outside = self.root / "outside"
        outside.mkdir()
        self.staging.mkdir()
        (self.staging / "Interface").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.stage(digest)
        self.assertEqual(list(outside.iterdir()), [])

    def test_duplicate_conflicting_archive_member_is_rejected(self):
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr("Example/Main.lua", "first")
            archive.writestr("Example\\Main.lua", "second")
        digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        with self.assertRaisesRegex(ValueError, "conflict"):
            self.stage(digest)
        self.assertFalse(self.staging.exists())

    def test_file_directory_collision_is_rejected_before_writes(self):
        digest = self.make_zip({"Example": "file", "Example/Main.lua": "nested"})
        with self.assertRaisesRegex(ValueError, "conflict"):
            self.stage(digest)
        self.assertFalse(self.staging.exists())

    def test_existing_observer_conflict_preserves_all_content(self):
        digest = self.make_zip()
        self.staging.mkdir()
        observer = self.staging / "load-observer.lua"
        observer.write_text("caller-owned")
        with self.assertRaisesRegex(ValueError, "conflict"):
            self.stage(digest)
        self.assertEqual(observer.read_text(), "caller-owned")
        self.assertFalse((self.staging / "Interface").exists())

    def test_argv_is_exact_bounded_isolated_command_without_execution(self):
        staged = self.stage(self.make_zip())
        binary = self.root / "wow-sim"
        argv = HELPER.build_argv(staged, binary, timeout=45)
        self.assertEqual(
            argv,
            [
                "env",
                "-u",
                "WOW_SIM_NO_ADDONS",
                f"WOW_SIM_ADDONS_PATH={self.staging}/Interface/AddOns",
                f"WOW_SIM_ADDONS_TXT={self.staging}/AddOns.txt",
                f"WOW_INSTALL_PATH={self.staging}/fake-install",
                f"WOW_DATA_PATH={self.staging}/fake-install",
                f"WOW_SIM_WTF_PATH={self.staging}/wtf",
                "WOW_SIM_CASC=0",
                "timeout",
                "45",
                str(binary),
                "--no-saved-vars",
                "--exec-lua",
                f"@{self.staging}/load-observer.lua",
                "lua-errors",
            ],
        )
        for timeout in (0, 91, -1, 2.5):
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                HELPER.build_argv(staged, binary, timeout=timeout)

    def test_clean_startup_requires_loaded_root_but_allows_unloaded_lod(self):
        output = "other output\nAUDIT_ADDON\tExample\ttrue\ttrue\tfalse\t\nAUDIT_ADDON\tOptions\tfalse\tfalse\ttrue\tDEMAND_LOADED\nAUDIT_DONE\n[]\n"
        result = HELPER.parse_result(0, output, ["Example", "Options"])
        self.assertEqual(result["status"], "clean-startup")
        self.assertEqual(result["errors"], [])
        self.assertEqual(
            result["observations"][1],
            {
                "name": "Options",
                "loadingOrLoaded": False,
                "fullyLoaded": False,
                "loadOnDemand": True,
                "reason": "DEMAND_LOADED",
            },
        )

    def test_all_unloaded_is_not_clean_even_when_lod(self):
        result = HELPER.parse_result(
            0, "AUDIT_ADDON\tOptions\tfalse\tfalse\ttrue\t\nAUDIT_DONE\n[]", ["Options"]
        )
        self.assertEqual(result["status"], "unloaded")

    def test_unloaded_non_lod_root_remains_explicit_despite_loaded_library(self):
        output = "AUDIT_ADDON\tLibrary\ttrue\ttrue\tfalse\t\nAUDIT_ADDON\tExample\tfalse\tfalse\tfalse\tMISSING\nAUDIT_DONE\n[]"
        result = HELPER.parse_result(0, output, ["Library", "Example"])
        self.assertEqual(result["status"], "unloaded")
        self.assertEqual(result["observations"][1]["reason"], "MISSING")

    def test_loading_without_fully_loaded_is_not_clean(self):
        result = HELPER.parse_result(
            0, "AUDIT_ADDON\tExample\ttrue\tfalse\tfalse\t\nAUDIT_DONE\n[]", ["Example"]
        )
        self.assertEqual(result["status"], "unloaded")

    def test_lua_errors_or_nonzero_exit_are_failed(self):
        prefix = "AUDIT_ADDON\tExample\ttrue\ttrue\tfalse\t\nAUDIT_DONE\n"
        errors = [{"message": "bad aura\ncallback", "count": 2}]
        result = HELPER.parse_result(
            0, prefix + json.dumps(errors, indent=2), ["Example"]
        )
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["errors"], errors)
        self.assertEqual(
            HELPER.parse_result(124, prefix + "[]", ["Example"])["status"], "failed"
        )

    def test_incomplete_observer_or_json_never_passes(self):
        row = "AUDIT_ADDON\tExample\ttrue\ttrue\tfalse\t\n"
        for output in (
            row + "[]",
            "AUDIT_DONE\n[]",
            row + "AUDIT_DONE\n",
            row + "AUDIT_DONE\n[]\ntrailing",
            row + row + "AUDIT_DONE\n[]",
            row.replace("true", "unknown") + "AUDIT_DONE\n[]",
            row + "AUDIT_DONE\n{}",
        ):
            with self.subTest(output=output):
                self.assertEqual(
                    HELPER.parse_result(0, output, ["Example"])["status"], "incomplete"
                )


if __name__ == "__main__":
    unittest.main()
