#!/usr/bin/env python3
"""Generate the bundled CASC listfile subset used by wow-ui-sim."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE = (
    Path(os.environ["ASSET_RESOLVER_DATA_DIR"])
    if "ASSET_RESOLVER_DATA_DIR" in os.environ
    else Path(os.environ.get("XDG_CACHE_HOME", str(Path.home() / ".cache"))) / "asset-resolver/data"
) / "community-listfile.csv"
DEFAULT_OUTPUT = ROOT / "data/wow-ui-sim-listfile.csv"
DEFAULT_PROVENANCE = ROOT / "data/wow-ui-sim-listfile.provenance.json"
MANIFEST_PATH = ROOT / "crates/sim-data/data/manifest_interface_data.rs"
ATLAS_PATH = ROOT / "crates/sim-data-profile/data/atlas.rs"
BLIZZARD_UI_FILE_MANIFEST_DIR = ROOT / "data/blizzard-ui-files"
LISTFILE_OVERRIDES = ROOT / "data/listfile-overrides.csv"
SCAN_PATHS = [
    ROOT / "src",
    ROOT / "Interface/AddOns",
]
FONT_PATHS = [
    "Fonts/FRIZQT__.TTF",
    "Fonts/ARIALN.TTF",
    "Fonts/FRIZQT___CYR.TTF",
    "fonts/arheiuhk_bd.ttf",
    "fonts/morpheus.ttf",
    "fonts/skurri.ttf",
]
PROBE_PATHS = [
    "interface/buttons/ui-panel-button-up.blp",
    "interface/buttons/ui-panel-button-down.blp",
    "interface/buttons/ui-panel-button-highlight.blp",
    "interface/buttons/ui-panel-button-disabled.blp",
    "interface/glues/models/ui_nightelf/ui_nightelf.mdx",
]
SOUNDKIT_FDIDS = [
    567407,
    567422,
    567433,
    567440,
    567457,
    567464,
    567472,
    567490,
    567496,
    567502,
    567507,
]
EXTENSIONS = ["blp", "BLP", "tga", "TGA", "ttf", "TTF", "otf", "OTF"]


def main() -> None:
    args = parse_args()
    provenance = load_provenance(args.provenance)
    rows, path_count, fdid_count = generate_verified_rows(args.source, provenance)
    content = serialize_rows(rows)
    if args.check:
        if not args.output.is_file() or args.output.read_bytes() != content:
            raise ValueError(f"output drift: {args.output} differs from pinned regeneration")
    else:
        write_rows(args.output, rows)
    print(
        f"Generated {len(rows)} rows from "
        f"{path_count} path candidates and {fdid_count} fileDataIDs"
    )
    print(f"Output: {args.output}")


def generate_verified_rows(source: Path, provenance: dict) -> tuple[list[tuple[int, str]], int, int]:
    verify_identity(file_identity(source), provenance["source"], "source listfile")
    verify_identity(file_identity(LISTFILE_OVERRIDES), provenance["extra_sources"][0], "override source")
    by_path, by_fdid = load_source(source)
    override_fdids = load_listfile_overrides(by_path, by_fdid)
    requested_paths, requested_fdids = collect_requests()
    requested_fdids.update(override_fdids)
    blizzard_files = collect_blizzard_ui_files()
    verify_requests(provenance, requested_paths, requested_fdids, blizzard_files)
    rows = resolve_rows(by_path, by_fdid, requested_paths, requested_fdids, blizzard_files)
    verify_identity(
        {**bytes_identity(serialize_rows(rows)), "rows": len(rows)},
        provenance["output"],
        "generated output",
    )
    return rows, len(requested_paths), len(requested_fdids)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--provenance", type=Path, default=DEFAULT_PROVENANCE)
    parser.add_argument("--check", action="store_true", help="verify the output without writing it")
    return parser.parse_args()


def bytes_identity(content: bytes) -> dict:
    return {"bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}


def file_identity(path: Path) -> dict:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return {"bytes": path.stat().st_size, "sha256": digest.hexdigest()}


def load_provenance(path: Path) -> dict:
    provenance = json.loads(path.read_text(encoding="utf-8"))
    if provenance.get("schema") != 1:
        raise ValueError(f"unsupported listfile provenance schema: {path}")
    extras = provenance.get("extra_sources", [])
    if len(extras) != 1 or extras[0].get("path") != "data/listfile-overrides.csv":
        raise ValueError("provenance must pin the authoritative listfile override source")
    return provenance


def verify_identity(actual: dict, expected: dict, label: str) -> None:
    for key, value in actual.items():
        if expected.get(key) != value:
            raise ValueError(f"{label} drift: {key} expected {expected.get(key)}, got {value}")


def selection_identity(values) -> dict:
    entries = sorted(map(str, values))
    content = ("\n".join(entries) + "\n").encode("utf-8")
    return {"count": len(entries), "sha256": hashlib.sha256(content).hexdigest()}


def request_identity(paths: set[str], fdids: set[int], ui_files: set[str]) -> dict:
    return {
        "path_candidates": selection_identity(paths),
        "file_data_ids": selection_identity(fdids),
        "blizzard_ui_files": selection_identity(ui_files),
    }


def verify_requests(provenance: dict, paths: set[str], fdids: set[int], ui_files: set[str]) -> None:
    for group, actual in request_identity(paths, fdids, ui_files).items():
        if provenance["requests"].get(group) != actual:
            raise ValueError(f"request selection drift: {group}; review and update the input pin")


def load_source(path: Path) -> tuple[dict[str, tuple[int, str]], dict[int, str]]:
    by_path: dict[str, tuple[int, str]] = {}
    by_fdid: dict[int, str] = {}
    load_listfile_rows(path, by_path, by_fdid, authoritative=False)
    return by_path, by_fdid


def load_listfile_overrides(
    by_path: dict[str, tuple[int, str]], by_fdid: dict[int, str]
) -> set[int]:
    if LISTFILE_OVERRIDES.exists():
        return load_listfile_rows(LISTFILE_OVERRIDES, by_path, by_fdid, authoritative=True)
    return set()


def load_listfile_rows(
    path: Path,
    by_path: dict[str, tuple[int, str]],
    by_fdid: dict[int, str],
    *,
    authoritative: bool,
) -> set[int]:
    fdids: set[int] = set()
    with path.open("r", encoding="utf-8", errors="replace", newline="") as handle:
        for raw in handle:
            raw = raw.rstrip("\r\n")
            fdid_text, sep, asset_path = raw.partition(";")
            if not sep:
                continue
            try:
                fdid = int(fdid_text)
            except ValueError:
                continue
            if authoritative:
                fdids.add(fdid)
            normalized = normalize_path(asset_path)
            display_path = normalize_slashes(asset_path) if authoritative else normalized
            by_path[normalized] = (fdid, display_path)
            if authoritative:
                by_fdid[fdid] = display_path
            else:
                by_fdid.setdefault(fdid, display_path)
    return fdids


def collect_requests() -> tuple[set[str], set[int]]:
    paths = {normalize_path(path) for path in FONT_PATHS + PROBE_PATHS}
    fdids = collect_manifest_fdids()
    fdids.update(SOUNDKIT_FDIDS)
    paths.update(collect_atlas_paths())
    paths.update(scan_literal_paths())
    return paths, fdids


def collect_manifest_fdids() -> set[int]:
    text = MANIFEST_PATH.read_text(encoding="utf-8")
    return {int(match) for match in re.findall(r"\(\s*(\d+)\s*,\s*\"", text)}


def collect_atlas_paths() -> set[str]:
    text = ATLAS_PATH.read_text(encoding="utf-8")
    return {
        normalize_path(match)
        for match in re.findall(r'file:\s*r"([^"]+)"', text)
        if match
    }


def scan_literal_paths() -> set[str]:
    paths: set[str] = set()
    for root in SCAN_PATHS:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.suffix.lower() not in {".rs", ".lua", ".xml"}:
                continue
            paths.update(extract_interface_literals(path))
    return paths


def collect_blizzard_ui_files() -> set[str]:
    files: set[str] = set()
    for manifest in blizzard_ui_manifest_paths():
        files.update(
            normalize_path(f"interface/addons/{line.strip()}")
            for line in manifest.read_text(encoding="utf-8").splitlines()
            if line.strip()
        )
    return files


def blizzard_ui_manifest_paths() -> list[Path]:
    if not BLIZZARD_UI_FILE_MANIFEST_DIR.is_dir():
        return []
    return sorted(BLIZZARD_UI_FILE_MANIFEST_DIR.glob("*.txt"))


def extract_interface_literals(path: Path) -> set[str]:
    try:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return set()
    matches = re.findall(r"(?i)(?:Interface|Fonts)[\\/][A-Za-z0-9_ .@+()/\\-]+", text)
    return {trim_literal_path(match) for match in matches}


def trim_literal_path(path: str) -> str:
    return normalize_path(path).strip(" \t\r\n'\"),;:{}[]<>")


def resolve_rows(
    by_path: dict[str, tuple[int, str]],
    by_fdid: dict[int, str],
    requested_paths: set[str],
    requested_fdids: set[int],
    blizzard_files: set[str] | None = None,
) -> list[tuple[int, str]]:
    rows: dict[int, str] = {}
    for fdid in requested_fdids:
        if path := by_fdid.get(fdid):
            rows[fdid] = path
    for path in requested_paths:
        if row := resolve_path(by_path, path):
            rows[row[0]] = row[1]
    for path in blizzard_files or set():
        if row := resolve_path(by_path, path):
            rows[row[0]] = row[1]
    return sorted(rows.items(), key=lambda row: normalize_path(row[1]))


def resolve_path(
    by_path: dict[str, tuple[int, str]], path: str
) -> tuple[int, str] | None:
    for candidate in path_candidates(path):
        if row := by_path.get(candidate):
            return row
    return None


def path_candidates(path: str) -> list[str]:
    normalized = normalize_path(path)
    bases = [normalized]
    if not normalized.startswith("interface/") and not normalized.startswith("fonts/"):
        bases.append(f"interface/{normalized}")
    candidates: list[str] = []
    for base in bases:
        candidates.append(base)
        if "." not in Path(base).name:
            candidates.extend(f"{base}.{ext}" for ext in EXTENSIONS)
    return [normalize_path(candidate) for candidate in candidates]


def normalize_slashes(path: str) -> str:
    return path.replace("\\", "/")


def normalize_path(path: str) -> str:
    return normalize_slashes(path).lower()


def serialize_rows(rows: list[tuple[int, str]]) -> bytes:
    return "".join(f"{fdid};{asset_path}\n" for fdid, asset_path in rows).encode("utf-8")


def write_rows(path: Path, rows: list[tuple[int, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(serialize_rows(rows))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise SystemExit(f"Listfile generation failed: {error}") from error
