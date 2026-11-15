"""Selected Retail source identity and its shipped file-index contract."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class RetailUiPinTests(unittest.TestCase):
    def test_retail_content_index_selects_installed_70783(self):
        index = json.loads((ROOT / "data/blizzard-ui-builds/retail.json").read_text())
        self.assertEqual(index["product"], "wow")
        self.assertEqual(index["version"], "12.1.5.70783")
        self.assertEqual(index["build_key"], "bc493ecb2898766c0f8947a99b4b3d65")
        self.assertEqual(index["cdn_key"], "3363ac656da5f261cdc675ab28b2744c")
        self.assertEqual(index["install_key"], "e58758dc26b4c93c71f5fb65054f90c4")
        self.assertEqual(
            index["gethe_revision"], "297d7aaaa736f346b3e036f38ee088eaf7f8c07f"
        )

    def test_retail_manifest_and_index_describe_the_same_unique_files(self):
        index = json.loads((ROOT / "data/blizzard-ui-builds/retail.json").read_text())
        manifest = (ROOT / "data/blizzard-ui-files/retail.txt").read_text().splitlines()
        paths = [entry["path"] for entry in index["files"]]
        self.assertEqual(paths, sorted(set(paths)))
        self.assertEqual(paths, manifest)
        self.assertEqual(len({entry["file_data_id"] for entry in index["files"]}), len(paths))


if __name__ == "__main__":
    unittest.main()
