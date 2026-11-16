#!/usr/bin/env python3
"""Build the Retail Blizzard CDN index from an exact installed CASC and Gethe pin."""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import subprocess
from pathlib import Path
from typing import Any


PRODUCT = "wow"
PROFILE = "retail"
SCHEMA = 1


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def md5(path: Path) -> str:
    digest = hashlib.md5()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_config(path: Path) -> dict[str, str]:
    result = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        name, separator, value = line.partition(" = ")
        if separator:
            result[name] = value
    return result


def selected_build_info(path: Path) -> dict[str, str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    header = lines[0].split("|")
    fields = [column.split("!", 1)[0] for column in header]
    matches = [dict(zip(fields, line.split("|"))) for line in lines[1:]
               if len(line.split("|")) == len(fields)
               and dict(zip(fields, line.split("|"))).get("Product") == PRODUCT]
    if len(matches) != 1:
        raise ValueError(f"expected one installed {PRODUCT} row, found {len(matches)}")
    return matches[0]


def listfile_paths(path: Path) -> dict[str, set[int]]:
    paths: dict[str, set[int]] = {}
    with path.open("r", encoding="utf-8", errors="replace") as stream:
        for line in stream:
            fdid_text, separator, asset_path = line.rstrip("\r\n").partition(";")
            if not separator or not fdid_text.isdecimal():
                continue
            normalized = asset_path.replace("\\", "/").lower()
            prefix = "interface/addons/"
            if normalized.startswith(prefix):
                paths.setdefault(normalized[len(prefix):], set()).add(int(fdid_text))
    return paths


def source_content_keys(path: Path) -> dict[str, int]:
    raw = path.read_bytes()
    variants = (raw, raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"))
    return {hashlib.md5(content).hexdigest(): len(content) for content in variants}


def parse_needed_archive_locations(
    index_dir: Path, archives: set[str], needed_keys: set[str]
) -> dict[str, dict[str, int | str]]:
    locations: dict[str, dict[str, int | str]] = {}
    for archive in sorted(archives):
        path = index_dir / f"{archive}.index"
        raw = path.read_bytes()
        if len(raw) < 4096 or hashlib.md5(raw[-28:]).hexdigest() != archive:
            raise ValueError(f"archive index hash mismatch: {path}")
        footer = raw[-20:]
        if footer[:8] != b"\x01\0\0\x04\x04\x04\x10\x08":
            raise ValueError(f"unsupported archive index footer: {path}")
        count = int.from_bytes(footer[8:12], "little")
        for index in range(count):
            offset = (index // 170) * 4096 + (index % 170) * 24
            key = raw[offset:offset + 16].hex()
            if key not in needed_keys:
                continue
            size, archive_offset = int.from_bytes(raw[offset + 16:offset + 20], "big"), int.from_bytes(
                raw[offset + 20:offset + 24], "big"
            )
            locations[key] = {"archive": archive, "offset": archive_offset, "size": size}
    missing = needed_keys - locations.keys()
    if missing:
        raise ValueError(f"{len(missing)} encoding keys have no configured archive index entry")
    return locations


def source_revision(source_dir: Path) -> tuple[str, str]:
    checkout = source_dir.parent.parent
    version = (checkout / "version.txt").read_text(encoding="utf-8").strip()
    revision = subprocess.run(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    return version, revision


def read_verified_install_configs(
    install: Path, decoded_root: Path, decoded_encoding: Path
) -> tuple[str, str, str, str, dict[str, str]]:
    installed = selected_build_info(install / ".build.info")
    version = installed["Version"]
    build_key, cdn_key = installed["Build Key"], installed["CDN Key"]
    build_path = install / "Data/config" / build_key[:2] / build_key[2:4] / build_key
    cdn_path = install / "Data/config" / cdn_key[:2] / cdn_key[2:4] / cdn_key
    if md5(build_path) != build_key or md5(cdn_path) != cdn_key:
        raise ValueError("installed build or CDN config does not match .build.info")
    build, cdn = read_config(build_path), read_config(cdn_path)
    expected_name = (
        f"WOW-{version.rsplit('.', 1)[-1]}patch"
        f"{'.'.join(version.split('.')[:-1])}_Retail"
    )
    if build.get("build-uid") != PRODUCT or build.get("build-name") != expected_name:
        raise ValueError("installed build config does not describe selected Retail identity")
    install_key = build.get("install", "").split()[0]
    root_key = build.get("root", "").split()[0]
    encoding_key = build.get("encoding", "").split()[0]
    if md5(decoded_root) != root_key or md5(decoded_encoding) != encoding_key:
        raise ValueError("decoded root or encoding cache does not match installed build config")
    return version, build_key, cdn_key, install_key, cdn


def resolve_source_file(
    db: sqlite3.Connection,
    path: str,
    source_dir: Path,
    listfile_by_path: dict[str, set[int]],
) -> tuple[str, int, str, str, int]:
    ids = listfile_by_path.get(path.lower(), set())
    if len(ids) != 1:
        raise ValueError(f"manifest path has {len(ids)} listfile FDIDs: {path}")
    fdid = next(iter(ids))
    row = db.execute(
        "SELECT hex(content_key), hex(encoding_key) FROM resolution WHERE fdid=?",
        (fdid,),
    ).fetchone()
    if row is None:
        raise ValueError(f"FDID {fdid} is absent from local root resolution: {path}")
    content_key, encoding_key = row[0].lower(), row[1].lower()
    content_sizes = source_content_keys(source_dir / path)
    content_size = content_sizes.get(content_key)
    if content_size is None:
        raise ValueError(f"Gethe content key differs from installed root for {path}")
    return path, fdid, content_key, encoding_key, content_size


def resolve_source_files(
    manifest: list[str],
    source_dir: Path,
    listfile: Path,
    decoded_root: Path,
    decoded_encoding: Path,
    resolution_db: Path,
) -> list[tuple[str, int, str, str, int]]:
    listfile_by_path = listfile_paths(listfile)
    mapped: list[tuple[str, int, str, str, int]] = []
    db_uri = f"file:{resolution_db}?mode=ro"
    with sqlite3.connect(db_uri, uri=True) as db:
        metadata = db.execute(
            "SELECT root_mtime, enc_mtime, schema_version FROM metadata"
        ).fetchone()
        if metadata is None or metadata[2] != 2:
            raise ValueError("local CASC resolution database has no supported metadata")
        root_mtime = int(decoded_root.stat().st_mtime)
        if metadata[0] != root_mtime:
            raise ValueError("local resolution database does not match decoded root/encoding files")
        encoding_mtime = int(decoded_encoding.stat().st_mtime)
        if metadata[1] != encoding_mtime:
            raise ValueError("local resolution database does not match decoded root/encoding files")
        for path in manifest:
            mapped.append(resolve_source_file(db, path, source_dir, listfile_by_path))
    return mapped


def read_cdn_file_records(
    install: Path,
    cdn: dict[str, str],
    mapped: list[tuple[str, int, str, str, int]],
) -> list[dict[str, Any]]:
    archives = set(
        (cdn.get("archives", "") + " " + cdn.get("patch-archives", "")).split()
    )
    if not archives:
        raise ValueError("installed CDN config has no archives")
    locations = parse_needed_archive_locations(
        install / "Data/indices", archives, {item[3] for item in mapped}
    )
    files = [
        {
            "path": path,
            "file_data_id": fdid,
            "content_key": content_key,
            "encoding_key": encoding_key,
            "content_size": content_size,
            **locations[encoding_key],
        }
        for path, fdid, content_key, encoding_key, content_size in mapped
    ]
    files.sort(key=lambda entry: entry["path"])
    return files


def build_index_document(
    version: str,
    build_key: str,
    cdn_key: str,
    install_key: str,
    revision: str,
    files: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "product": PRODUCT,
        "version": version,
        "build_key": build_key,
        "cdn_key": cdn_key,
        "install_key": install_key,
        "gethe_revision": revision,
        "files": files,
    }


def build_index(
    install: Path,
    source_dir: Path,
    listfile: Path,
    decoded_root: Path,
    decoded_encoding: Path,
    resolution_db: Path,
) -> tuple[dict[str, Any], list[str]]:
    version, build_key, cdn_key, install_key, cdn = read_verified_install_configs(
        install, decoded_root, decoded_encoding
    )
    revision_version, revision = source_revision(source_dir)
    if revision_version != version:
        raise ValueError(f"Gethe source version {revision_version} != installed {version}")

    manifest = sorted(
        path.relative_to(source_dir).as_posix()
        for path in source_dir.rglob("*")
        if path.is_file()
    )
    mapped = resolve_source_files(
        manifest, source_dir, listfile, decoded_root, decoded_encoding, resolution_db
    )
    files = read_cdn_file_records(install, cdn, mapped)
    index = build_index_document(
        version, build_key, cdn_key, install_key, revision, files
    )
    return index, manifest


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--install", type=Path, required=True)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--listfile", type=Path, required=True)
    parser.add_argument("--decoded-root", type=Path, required=True)
    parser.add_argument("--decoded-encoding", type=Path, required=True)
    parser.add_argument("--resolution-db", type=Path, required=True)
    parser.add_argument("--manifest-output", type=Path, required=True)
    parser.add_argument("--index-output", type=Path, required=True)
    args = parser.parse_args()
    index, manifest = build_index(
        args.install, args.source_dir, args.listfile, args.decoded_root,
        args.decoded_encoding, args.resolution_db,
    )
    args.manifest_output.write_text("\n".join(manifest) + "\n", encoding="utf-8")
    args.index_output.write_text(json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Retail {index['version']} Gethe {index['gethe_revision']}: {len(manifest)} files -> {args.manifest_output}")
    print(f"Indexed {len(index['files'])} verified CDN entries -> {args.index_output}")


if __name__ == "__main__":
    main()
