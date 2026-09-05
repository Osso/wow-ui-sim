from __future__ import annotations

import hashlib
import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from tools import gen_blizzard_ui_content_index
from tools.gen_blizzard_ui_content_index import (
    PINNED_GETHE_REVISION,
    PINNED_VERSION,
    build_content_index,
    source_checkout_dir,
)


def write_config(path: Path, fields: dict[str, str]) -> None:
    path.write_text("\n".join(f"{key} = {value}" for key, value in fields.items()) + "\n")


def write_download(path: Path, encoding_key: bytes) -> None:
    path.write_bytes(
        b"DL\x01\x10\0" + (1).to_bytes(4, "big") + b"\0\0" + encoding_key + b"\0" * 6
    )


def write_archive_index(index_dir: Path, encoding_key: bytes) -> str:
    entry = encoding_key + struct.pack(">II", 321, 654)
    block = entry + b"\0" * (4096 - len(entry))
    footer = b"\x01\0\0\x04\x04\x04\x10\x08" + (1).to_bytes(4, "little") + b"\0" * 8
    raw = block + b"\0" * 8 + footer
    archive_key = hashlib.md5(raw[-28:]).hexdigest()
    (index_dir / f"{archive_key}.index").write_bytes(raw)
    return archive_key


def write_tvfs(path: Path, fdid: int, content_key: bytes, encoding_prefix: bytes) -> None:
    path_name = f"{fdid:08X}{content_key.hex().upper()}".encode()
    path_table = bytes([len(path_name)]) + path_name + b"\xff" + (0).to_bytes(4, "big")
    vfs_table = b"\x01" + struct.pack(">II", 0, 123) + b"\0"
    cft_table = encoding_prefix + b"\0" * (20 - len(encoding_prefix)) + content_key
    header = (
        b"TVFS\x01\x2e\x09\x09"
        + (1).to_bytes(4, "big")
        + struct.pack(
            ">6I",
            46,
            len(path_table),
            46 + len(path_table),
            len(vfs_table),
            46 + len(path_table) + len(vfs_table),
            len(cft_table),
        )
        + b"\0\x01"
        + b"\0" * 8
    )
    path.write_bytes(header + path_table + vfs_table + cft_table)


def write_root_tvfs(path: Path, content_keys: list[bytes]) -> None:
    path_table = b"".join(
        bytes([40])
        + f"{index:08X}{content_key.hex().upper()}".encode()
        + b"\xff"
        + (index * 10).to_bytes(4, "big")
        for index, content_key in enumerate(content_keys)
    )
    vfs_table = b"".join(
        b"\x01" + struct.pack(">II", 0, 1) + bytes([index * 20])
        for index in range(len(content_keys))
    )
    cft_table = b"\0" * (20 * len(content_keys))
    header = (
        b"TVFS\x01\x2e\x09\x09"
        + (1).to_bytes(4, "big")
        + struct.pack(
            ">6I",
            46,
            len(path_table),
            46 + len(path_table),
            len(vfs_table),
            46 + len(path_table) + len(vfs_table),
            len(cft_table),
        )
        + b"\0\x01"
        + b"\0" * 8
    )
    path.write_bytes(header + path_table + vfs_table + cft_table)


class ContentIndexTests(unittest.TestCase):
    def make_fixture(self) -> tuple[Path, Path, Path]:
        root = Path(tempfile.mkdtemp())
        prefix = root / "metadata"
        source = root / "source"
        source.mkdir()
        payload = b"local value = true\n"
        (source / "Example.lua").write_bytes(payload)
        content_key = hashlib.md5(payload.replace(b"\n", b"\r\n")).digest()
        encoding_key = bytes.fromhex("00112233445566778899aabbccddeeff")
        index_dir = prefix.with_name(prefix.name + "-archive-indices")
        index_dir.mkdir()
        archive_key = write_archive_index(index_dir, encoding_key)
        cdn_path = prefix.with_name(prefix.name + "-cdn-config.txt")
        write_config(cdn_path, {"archives": archive_key})
        (root / "listfile.csv").write_text("42;interface/addons/example.lua\n")
        neutral_path = prefix.with_name(prefix.name + "-vfs-neutral.bin")
        write_tvfs(neutral_path, 42, content_key, encoding_key[:9])
        en_us_path = prefix.with_name(prefix.name + "-vfs-enUS.bin")
        write_tvfs(en_us_path, 44, hashlib.md5(b"locale").digest(), b"\0" * 9)
        root_path = prefix.with_name(prefix.name + "-vfs-root.bin")
        write_root_tvfs(
            root_path,
            [
                hashlib.md5(neutral_path.read_bytes()).digest(),
                hashlib.md5(en_us_path.read_bytes()).digest(),
            ],
        )
        download_path = prefix.with_name(prefix.name + "-download.bin")
        write_download(download_path, encoding_key)
        write_config(
            prefix.with_name(prefix.name + "-build-config.txt"),
            {
                "build-name": "WOW-69594patch12.1.5_XPTR",
                "build-uid": "wowxptr",
                "install": "06feedbc851542370f3d2081fce81e19 21c1e5624dd78f5487c97caea43bf035",
                "download": hashlib.md5(download_path.read_bytes()).hexdigest(),
                "vfs-root": hashlib.md5(root_path.read_bytes()).hexdigest(),
            },
        )
        return prefix, source, root / "index.json"

    def fixture_keys(self, prefix: Path) -> tuple[str, str]:
        return (
            hashlib.md5(prefix.with_name(prefix.name + "-build-config.txt").read_bytes()).hexdigest(),
            hashlib.md5(prefix.with_name(prefix.name + "-cdn-config.txt").read_bytes()).hexdigest(),
        )

    def build_fixture_index(
        self, prefix: Path, source: Path, keys: tuple[str, str] | None = None
    ) -> dict[str, object]:
        build_key, cdn_key = keys or self.fixture_keys(prefix)
        with mock.patch.object(gen_blizzard_ui_content_index, "PINNED_BUILD_KEY", build_key), mock.patch.object(
            gen_blizzard_ui_content_index, "PINNED_CDN_KEY", cdn_key
        ):
            return build_content_index(prefix, source, prefix.parent / "listfile.csv")

    def test_builds_deterministic_pinned_record_from_crlf_source_match(self) -> None:
        prefix, source, output = self.make_fixture()
        index = self.build_fixture_index(prefix, source)

        self.assertEqual(index["schema"], 1)
        self.assertEqual(index["gethe_revision"], PINNED_GETHE_REVISION)
        self.assertEqual(index["version"], PINNED_VERSION)
        self.assertEqual(index["product"], "wowxptr")
        build_key, cdn_key = self.fixture_keys(prefix)
        self.assertEqual(index["build_key"], build_key)
        self.assertEqual(index["cdn_key"], cdn_key)
        self.assertEqual(index["install_key"], "06feedbc851542370f3d2081fce81e19")
        self.assertEqual(
            index["files"][0],
            {
                "archive": index["files"][0]["archive"],
                "content_key": hashlib.md5(b"local value = true\r\n").hexdigest(),
                "content_size": 123,
                "encoding_key": "00112233445566778899aabbccddeeff",
                "file_data_id": 42,
                "offset": 654,
                "path": "Example.lua",
                "size": 321,
            },
        )
        self.assertRegex(index["files"][0]["archive"], r"^[0-9a-f]{32}$")
        output.write_text(json.dumps(index, sort_keys=True) + "\n")
        self.assertEqual(json.loads(output.read_text()), index)

    def test_uses_unique_content_key_when_listfile_lacks_new_file_data_id(self) -> None:
        prefix, source, _ = self.make_fixture()
        (prefix.parent / "listfile.csv").write_text("")
        index = self.build_fixture_index(prefix, source)
        self.assertEqual(index["files"][0]["path"], "Example.lua")

    def test_finds_checkout_root_from_addons_directory(self) -> None:
        self.assertEqual(source_checkout_dir(Path("/tmp/pinned/Interface/AddOns")), Path("/tmp/pinned"))

    def test_rejects_tampered_pinned_metadata_inputs(self) -> None:
        suffixes = (
            "build-config.txt",
            "cdn-config.txt",
            "vfs-root.bin",
            "vfs-neutral.bin",
            "vfs-enUS.bin",
            "download.bin",
        )
        for suffix in suffixes:
            with self.subTest(suffix=suffix):
                prefix, source, _ = self.make_fixture()
                keys = self.fixture_keys(prefix)
                metadata = prefix.with_name(f"{prefix.name}-{suffix}")
                metadata.write_bytes(metadata.read_bytes() + b"tampered")
                with self.assertRaisesRegex(ValueError, "pinned|metadata"):
                    self.build_fixture_index(prefix, source, keys)

    def test_rejects_missing_archive_location(self) -> None:
        prefix, source, _ = self.make_fixture()
        for file in prefix.with_name(prefix.name + "-archive-indices").iterdir():
            file.unlink()
        with self.assertRaisesRegex(ValueError, "missing archive location"):
            self.build_fixture_index(prefix, source)

    def test_rejects_archive_index_with_wrong_footer_hash(self) -> None:
        prefix, source, _ = self.make_fixture()
        index_file = next(prefix.with_name(prefix.name + "-archive-indices").iterdir())
        raw = bytearray(index_file.read_bytes())
        raw[-1] ^= 1
        index_file.write_bytes(raw)
        with self.assertRaisesRegex(ValueError, "archive index hash mismatch"):
            self.build_fixture_index(prefix, source)


if __name__ == "__main__":
    unittest.main()
