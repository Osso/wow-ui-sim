#!/usr/bin/env python3
"""Generate the Retail Mythic+ season-to-map table from a pinned DB2 CSV."""

import csv
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUILD = "12.1.5.70391"
URL = f"https://wago.tools/db2/MythicPlusSeasonTrackedMap/csv?build={BUILD}&locale=enUS"
SOURCE = ROOT / "data/db2/retail-12.1.5.70391/MythicPlusSeasonTrackedMap.csv"
OUTPUT = ROOT / "crates/sim-data/data/mythic_plus_season_maps.rs"


def read_rows(source):
    with source.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        required = {"ID", "MapChallengeModeID", "DisplaySeasonID"}
        if reader.fieldnames is None or not required <= set(reader.fieldnames):
            raise ValueError(f"{source}: missing required columns {sorted(required)}")
        rows = [
            (
                int(row["DisplaySeasonID"]),
                int(row["ID"]),
                int(row["MapChallengeModeID"]),
            )
            for row in reader
        ]
    if not rows or len({row[1] for row in rows}) != len(rows):
        raise ValueError("tracked-map rows must be nonempty with unique IDs")
    if any(season < 0 or row_id < 0 or map_id < 0 for season, row_id, map_id in rows):
        raise ValueError("tracked-map IDs must be nonnegative")
    return sorted(rows)


def render(rows, csv_hash):
    newest_season = max(season for season, _, _ in rows)
    lines = [
        f"//! Generated from MythicPlusSeasonTrackedMap.csv, build {BUILD}.",
        f"//! Source: {URL}",
        f"//! CSV SHA-256: {csv_hash}",
        "//! Regenerate with `python3 tools/gen_mythic_plus_season_maps.py`.",
        "pub const NEWEST_DISPLAY_SEASON_ID: u32 = " + f"{newest_season};",
        "pub static MAPS: &[(u32, u32)] = &[",
    ]
    lines.extend(f"    ({season}, {map_id})," for season, _, map_id in rows)
    return "\n".join([*lines, "];", ""])


def generate(source=SOURCE):
    rows = read_rows(source)
    csv_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    OUTPUT.write_text(render(rows, csv_hash), encoding="utf-8", newline="\n")
    print(f"Generated {len(rows)} tracked-map rows in {OUTPUT.relative_to(ROOT)}")


if __name__ == "__main__":
    generate(Path(sys.argv[1]) if len(sys.argv) > 1 else SOURCE)
