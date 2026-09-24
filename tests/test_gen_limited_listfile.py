import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools import gen_limited_listfile


class LimitedListfileGenerationTests(unittest.TestCase):
    def test_generates_override_missing_from_source_and_requests(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "community-listfile.csv"
            override = root / "listfile-overrides.csv"
            output = root / "generated.csv"
            source.write_text("1;Interface/Icons/Other.blp\n", encoding="utf-8")
            override.write_text(
                "8197078;Interface/Icons/INV_SideTab_Currency_c60.blp\n",
                encoding="utf-8",
            )
            (root / "manifest.rs").write_text("", encoding="utf-8")
            (root / "atlas.rs").write_text("", encoding="utf-8")

            with (
                patch.object(gen_limited_listfile, "LISTFILE_OVERRIDES", override),
                patch.object(gen_limited_listfile, "MANIFEST_PATH", root / "manifest.rs"),
                patch.object(gen_limited_listfile, "ATLAS_PATH", root / "atlas.rs"),
                patch.object(gen_limited_listfile, "SCAN_PATHS", [root / "empty"]),
                patch.object(gen_limited_listfile, "BLIZZARD_UI_FILE_MANIFEST_DIR", root / "empty"),
                patch.object(
                    sys,
                    "argv",
                    ["gen_limited_listfile.py", "--source", str(source), "--output", str(output)],
                ),
            ):
                gen_limited_listfile.main()

            self.assertEqual(
                "8197078;Interface/Icons/INV_SideTab_Currency_c60.blp\n",
                output.read_text(encoding="utf-8"),
            )

    def test_collects_blizzard_ui_files_from_profile_manifests(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest_dir = root / "blizzard-ui-files"
            manifest_dir.mkdir()
            (manifest_dir / "retail.txt").write_text(
                "Blizzard_Test/Blizzard_Test.lua\n",
                encoding="utf-8",
            )
            (manifest_dir / "ptr.txt").write_text(
                "Blizzard_Test/Textures/Icon.blp\n",
                encoding="utf-8",
            )

            with patch.object(
                gen_limited_listfile, "BLIZZARD_UI_FILE_MANIFEST_DIR", manifest_dir
            ):
                blizzard_files = gen_limited_listfile.collect_blizzard_ui_files()

            self.assertEqual(
                {
                    "interface/addons/blizzard_test/blizzard_test.lua",
                    "interface/addons/blizzard_test/textures/icon.blp",
                },
                blizzard_files,
            )

    def test_loads_tracked_listfile_overrides(self):
        with tempfile.TemporaryDirectory() as tmp:
            override = Path(tmp) / "listfile-overrides.csv"
            override.write_text(
                "12345;Interface/AddOns/Blizzard_Test/Override.lua\n",
                encoding="utf-8",
            )
            by_path = {}
            by_fdid = {}

            with patch.object(gen_limited_listfile, "LISTFILE_OVERRIDES", override):
                gen_limited_listfile.load_listfile_overrides(by_path, by_fdid)

            self.assertEqual(
                (12345, "interface/addons/blizzard_test/override.lua"),
                by_path["interface/addons/blizzard_test/override.lua"],
            )
            self.assertEqual(
                "interface/addons/blizzard_test/override.lua",
                by_fdid[12345],
            )

    def test_listfile_overrides_replace_existing_path_mapping(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / "community-listfile.csv"
            override = Path(tmp) / "listfile-overrides.csv"
            source.write_text(
                "111;Interface/AddOns/Blizzard_Test/Override.lua\n",
                encoding="utf-8",
            )
            override.write_text(
                "222;Interface/AddOns/Blizzard_Test/Override.lua\n",
                encoding="utf-8",
            )

            by_path, by_fdid = gen_limited_listfile.load_source(source)
            with patch.object(gen_limited_listfile, "LISTFILE_OVERRIDES", override):
                gen_limited_listfile.load_listfile_overrides(by_path, by_fdid)

            self.assertEqual(
                (222, "interface/addons/blizzard_test/override.lua"),
                by_path["interface/addons/blizzard_test/override.lua"],
            )
            self.assertEqual(
                "interface/addons/blizzard_test/override.lua",
                by_fdid[222],
            )

    def test_prefers_community_listfile_path_match_for_blizzard_ui_file(self):
        blizzard_files = {"interface/addons/blizzard_test/blizzard_test.lua"}
        rows = gen_limited_listfile.resolve_rows(
            by_path={
                "interface/addons/blizzard_test/blizzard_test.lua": (
                    67890,
                    "interface/addons/blizzard_test/blizzard_test.lua",
                )
            },
            by_fdid={},
            requested_paths=set(),
            requested_fdids=set(),
            blizzard_files=blizzard_files,
        )

        self.assertEqual(
            [(67890, "interface/addons/blizzard_test/blizzard_test.lua")],
            rows,
        )


if __name__ == "__main__":
    unittest.main()
