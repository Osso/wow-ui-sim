"""Tests for deriving a Retail CDN index from local installed metadata."""

import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.gen_blizzard_ui_retail_index import (
    parse_needed_archive_locations,
    selected_build_info,
    source_content_keys,
)


class RetailIndexGeneratorTests(unittest.TestCase):
    def test_selects_retail_row_from_build_info(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / ".build.info"
            path.write_text(
                "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\n"
                "us|1|abc|12.1.5.70783|wow\n"
                "us|1|def|12.1.7.70740|wowt\n",
                encoding="utf-8",
            )
            self.assertEqual(selected_build_info(path)["Build Key"], "abc")
            self.assertEqual(selected_build_info(path)["Version"], "12.1.5.70783")

    def test_content_keys_include_line_ending_variant(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "file.lua"
            path.write_bytes(b"a\nb\n")
            keys = source_content_keys(path)
            self.assertEqual(keys[hashlib.md5(b"a\nb\n").hexdigest()], 4)
            self.assertEqual(keys[hashlib.md5(b"a\r\nb\r\n").hexdigest()], 6)

    def test_reads_only_requested_archive_locations_after_footer_validation(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            data = bytearray(4096)
            encoding_key = bytes.fromhex("1234567890abcdef1234567890abcdef")
            data[:16] = encoding_key
            data[16:20] = (73).to_bytes(4, "big")
            data[20:24] = (901).to_bytes(4, "big")
            footer = len(data) - 20
            data[footer:footer + 8] = b"\x01\0\0\x04\x04\x04\x10\x08"
            data[footer + 8:footer + 12] = (1).to_bytes(4, "little")
            archive = hashlib.md5(data[-28:]).hexdigest()
            (directory / f"{archive}.index").write_bytes(data)

            locations = parse_needed_archive_locations(
                directory, {archive}, {encoding_key.hex()}
            )

            self.assertEqual(locations[encoding_key.hex()], {
                "archive": archive,
                "offset": 901,
                "size": 73,
            })
            with self.assertRaisesRegex(ValueError, "encoding keys have no"):
                parse_needed_archive_locations(directory, {archive}, {"00" * 16})


if __name__ == "__main__":
    unittest.main()
