"""Build the pinned PTR 12.1.5 Blizzard CDN content index from offline metadata."""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
import subprocess
from pathlib import Path

PINNED_GETHE_REVISION = "49b69918fcdc77e109813281e4f537d45ec7dcbf"
PINNED_VERSION = "12.1.5.69594"
PINNED_BUILD_KEY = "4a9973f37906f8cfb344f8a9fe6777e0"
PINNED_CDN_KEY = "3aa83893a3ce9b722a5f51328ad9a552"
SCHEMA = 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata-prefix", type=Path, required=True)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def read_config(path: Path) -> dict[str, str]:
    fields: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        key, separator, value = raw.partition(" = ")
        if separator:
            fields[key] = value
    return fields


def metadata_path(prefix: Path, suffix: str) -> Path:
    return prefix.with_name(f"{prefix.name}-{suffix}")


def md5_file(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def verify_pinned_file(path: Path, expected_key: str, description: str) -> None:
    actual_key = md5_file(path)
    if actual_key != expected_key:
        raise ValueError(
            f"pinned {description} key mismatch: expected {expected_key}, got {actual_key}"
        )


def config_content_key(config: dict[str, str], field: str) -> str:
    content_key = config.get(field, "").split(maxsplit=1)[0]
    if len(content_key) != 32 or not all(character in "0123456789abcdef" for character in content_key):
        raise ValueError(f"build metadata has no {field} content key")
    return content_key


def parse_tvfs_entries(path: Path) -> list[dict[str, int | str]]:
    raw = path.read_bytes()
    if raw[:4] != b"TVFS":
        raise ValueError(f"invalid TVFS magic: {path}")
    if len(raw) < 46:
        raise ValueError(f"truncated TVFS header: {path}")

    key_size = raw[6]
    if key_size != 9:
        raise ValueError(f"unsupported TVFS encoding-key size {key_size}: {path}")
    path_offset, path_size, vfs_offset, _, cft_offset, cft_size = struct.unpack_from(
        ">6I", raw, 12
    )
    cft_index_width = max(1, (cft_size.bit_length() + 7) // 8)
    entries: list[dict[str, int | str]] = []

    def visit(offset: int, end: int, prefix: str) -> None:
        initial_prefix = prefix
        while offset < end:
            current = prefix
            if raw[offset] == 0:
                current += "/"
                offset += 1
            if raw[offset] != 0xFF:
                length = raw[offset]
                offset += 1
                current += raw[offset : offset + length].decode("ascii")
                offset += length
            if offset < end and raw[offset] == 0:
                current += "/"
                offset += 1
            if offset < end and raw[offset] == 0xFF:
                reference = int.from_bytes(raw[offset + 1 : offset + 5], "big")
                offset += 5
                if reference & 0x80000000:
                    nested_end = offset + (reference & 0x7FFFFFFF) - 4
                    if nested_end > end:
                        raise ValueError(f"truncated TVFS path subtree: {path}")
                    visit(offset, nested_end, current)
                    offset = nested_end
                else:
                    if len(current) != 40:
                        prefix = initial_prefix
                        continue
                    vfs_entry = vfs_offset + reference
                    spans = raw[vfs_entry]
                    if spans != 1:
                        raise ValueError(
                            f"multi-span TVFS entry unsupported: {current}"
                        )
                    content_offset, content_size = struct.unpack_from(
                        ">II", raw, vfs_entry + 1
                    )
                    cft_relative = int.from_bytes(
                        raw[vfs_entry + 9 : vfs_entry + 9 + cft_index_width], "big"
                    )
                    cft_entry = cft_offset + cft_relative
                    encoding_prefix = raw[cft_entry : cft_entry + key_size]
                    entries.append(
                        {
                            "file_data_id": int(current[:8], 16),
                            "content_key": current[8:].lower(),
                            "encoding_prefix": encoding_prefix.hex(),
                            "content_offset": content_offset,
                            "content_size": content_size,
                        }
                    )
                prefix = initial_prefix
            else:
                prefix = current + "/"

    visit(path_offset, path_offset + path_size, "")
    return entries


def source_paths_by_normalized_path(source_dir: Path) -> dict[str, str]:
    return {
        path.relative_to(source_dir).as_posix().lower(): path.relative_to(
            source_dir
        ).as_posix()
        for path in source_dir.rglob("*")
        if path.is_file()
    }


def content_keys_by_source_path(source_dir: Path) -> dict[str, list[str]]:
    keys: dict[str, list[str]] = {}
    for source in sorted(path for path in source_dir.rglob("*") if path.is_file()):
        raw = source.read_bytes()
        variants = (raw, raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"))
        for variant in variants:
            key = hashlib.md5(variant).hexdigest()
            relative = source.relative_to(source_dir).as_posix()
            keys.setdefault(key, []).append(relative)
    return keys


def parse_listfile_paths(path: Path) -> dict[int, str]:
    paths: dict[int, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        file_data_id, separator, asset_path = raw.partition(";")
        if not separator or not file_data_id.isdecimal():
            continue
        prefix = "interface/addons/"
        normalized = asset_path.replace("\\", "/").lower()
        if normalized.startswith(prefix):
            paths[int(file_data_id)] = normalized[len(prefix) :]
    return paths


def parse_download_encoding_keys(path: Path) -> dict[str, str]:
    raw = path.read_bytes()
    if raw[:2] != b"DL" or raw[2] != 1 or raw[3] != 16:
        raise ValueError(f"unsupported download manifest: {path}")
    count = int.from_bytes(raw[5:9], "big")
    start = 11
    entry_size = 22
    expected = start + count * entry_size
    if len(raw) < expected:
        raise ValueError(f"truncated download manifest: {path}")
    return {
        raw[offset : offset + 9].hex(): raw[offset : offset + 16].hex()
        for offset in range(start, expected, entry_size)
    }


def parse_archive_locations(
    index_dir: Path, archives: set[str]
) -> dict[str, dict[str, int | str]]:
    locations: dict[str, dict[str, int | str]] = {}
    for path in sorted(index_dir.glob("*.index")):
        archive = path.stem
        if archive not in archives:
            raise ValueError(f"archive index is absent from CDN config: {archive}")
        raw = path.read_bytes()
        if len(raw) < 28 or hashlib.md5(raw[-28:]).hexdigest() != archive:
            raise ValueError(f"archive index hash mismatch: {path}")
        footer = raw[-20:]
        if footer[:8] != b"\x01\0\0\x04\x04\x04\x10\x08":
            raise ValueError(f"unsupported archive index footer: {path}")
        entries = int.from_bytes(footer[8:12], "little")
        for index in range(entries):
            offset = (index // 170) * 4096 + (index % 170) * 24
            key = raw[offset : offset + 16]
            if len(key) != 16:
                raise ValueError(f"truncated archive index entry: {path}")
            size, archive_offset = struct.unpack_from(">II", raw, offset + 16)
            locations[key.hex()] = {
                "archive": archive,
                "offset": archive_offset,
                "size": size,
            }
    return locations


def source_checkout_dir(source_dir: Path) -> Path:
    if source_dir.name != "AddOns" or source_dir.parent.name != "Interface":
        raise ValueError(
            f"source directory must be an Interface/AddOns tree: {source_dir}"
        )
    return source_dir.parent.parent


def verify_pinned_source(source_dir: Path) -> None:
    checkout = source_checkout_dir(source_dir)
    version = (checkout / "version.txt").read_text(encoding="utf-8").strip()
    if version != PINNED_VERSION:
        raise ValueError(f"expected Gethe version {PINNED_VERSION}, got {version}")
    result = subprocess.run(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    revision = result.stdout.strip()
    if revision != PINNED_GETHE_REVISION:
        raise ValueError(
            f"expected Gethe revision {PINNED_GETHE_REVISION}, got {revision}"
        )


def verify_pinned_metadata(prefix: Path) -> tuple[dict[str, str], dict[str, str]]:
    build_config_path = metadata_path(prefix, "build-config.txt")
    cdn_config_path = metadata_path(prefix, "cdn-config.txt")
    verify_pinned_file(build_config_path, PINNED_BUILD_KEY, "build config")
    verify_pinned_file(cdn_config_path, PINNED_CDN_KEY, "CDN config")
    build = read_config(build_config_path)
    cdn = read_config(cdn_config_path)
    if (
        build.get("build-name") != "WOW-69594patch12.1.5_XPTR"
        or build.get("build-uid") != "wowxptr"
    ):
        raise ValueError("metadata is not the pinned wowxptr 12.1.5 build")

    root_path = metadata_path(prefix, "vfs-root.bin")
    verify_pinned_file(root_path, config_content_key(build, "vfs-root"), "TVFS root")
    root_data = root_path.read_bytes()
    for suffix in ("vfs-neutral.bin", "vfs-enUS.bin"):
        leaf_path = metadata_path(prefix, suffix)
        leaf_key = bytes.fromhex(md5_file(leaf_path))
        encoded_leaf_key = leaf_key.hex().upper().encode("ascii")
        if leaf_key not in root_data and encoded_leaf_key not in root_data:
            raise ValueError(f"pinned TVFS root does not reference {suffix}")

    download_path = metadata_path(prefix, "download.bin")
    verify_pinned_file(download_path, config_content_key(build, "download"), "download manifest")
    return build, cdn


def build_content_index(
    prefix: Path,
    source_dir: Path,
    listfile_path: Path | None = None,
    *,
    verify_source: bool = False,
) -> dict[str, object]:
    if verify_source:
        verify_pinned_source(source_dir)
    build, cdn = verify_pinned_metadata(prefix)
    archives = cdn.get("archives", "").split()
    if not archives:
        raise ValueError("CDN config has no archives")

    tvfs_entries = []
    for suffix in ("vfs-neutral.bin", "vfs-enUS.bin"):
        tvfs_entries.extend(parse_tvfs_entries(metadata_path(prefix, suffix)))
    sources = content_keys_by_source_path(source_dir)
    source_paths = source_paths_by_normalized_path(source_dir)
    if listfile_path is None:
        listfile_path = (
            Path(__file__).resolve().parent.parent / "data/wow-ui-sim-listfile.csv"
        )
    paths_by_file_data_id = parse_listfile_paths(listfile_path)
    encoding_keys = parse_download_encoding_keys(metadata_path(prefix, "download.bin"))
    locations = parse_archive_locations(
        metadata_path(prefix, "archive-indices"), set(archives)
    )

    files = []
    for entry in tvfs_entries:
        content_key = entry["content_key"]
        source_matches = sources.get(content_key, [])
        listed_path = paths_by_file_data_id.get(entry["file_data_id"])
        if listed_path is None:
            if len(source_matches) != 1:
                continue
            path = source_matches[0]
        else:
            path = source_paths.get(listed_path)
            if path is None:
                continue
        if path not in source_matches:
            raise ValueError(f"content key does not match source path: {path}")
        encoding_key = encoding_keys.get(entry["encoding_prefix"])
        if encoding_key is None:
            raise ValueError(f"missing download encoding key for {path}")
        location = locations.get(encoding_key)
        if location is None:
            raise ValueError(f"missing archive location for {path}")
        files.append(
            {
                "path": path,
                "file_data_id": entry["file_data_id"],
                "content_key": content_key,
                "encoding_key": encoding_key,
                "content_size": entry["content_size"],
                **location,
            }
        )
    files.sort(key=lambda item: item["path"])
    if len({item["path"] for item in files}) != len(files):
        raise ValueError("multiple CDN records resolve to one source path")
    if len(files) != len(list(source_dir.rglob("*"))):
        source_files = sum(1 for path in source_dir.rglob("*") if path.is_file())
        if len(files) != source_files:
            raise ValueError(
                f"missing source records: indexed {len(files)} of {source_files}"
            )

    install_key = build.get("install", "").split()[0]
    if len(install_key) != 32:
        raise ValueError("build config has no install content key")
    return {
        "schema": SCHEMA,
        "product": "wowxptr",
        "version": PINNED_VERSION,
        "build_key": PINNED_BUILD_KEY,
        "cdn_key": PINNED_CDN_KEY,
        "install_key": install_key,
        "gethe_revision": PINNED_GETHE_REVISION,
        "files": files,
    }


def main() -> None:
    args = parse_args()
    index = build_content_index(
        args.metadata_prefix, args.source_dir, verify_source=True
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"Wrote {len(index['files'])} pinned PTR files to {args.output}")


if __name__ == "__main__":
    main()
