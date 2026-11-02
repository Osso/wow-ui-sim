#!/usr/bin/env python3
"""Generate the MapChallengeMode ID-to-map relation from a DB2 CSV export."""

import csv
import sys
from pathlib import Path

source = Path(sys.argv[1])
output = Path(__file__).resolve().parents[1] / "crates/sim-data/data/map_challenge_modes.rs"
with source.open(newline="", encoding="utf-8") as file:
    rows = sorted((int(row["ID"]), int(row["MapID"])) for row in csv.DictReader(file))
if len({row[0] for row in rows}) != len(rows) or any(not 0 <= m <= 65535 for _, m in rows):
    raise ValueError("duplicate challenge ID or MapID outside u16 range")
with output.open("w", encoding="utf-8") as file:
    file.write("//! Generated from MapChallengeMode.csv, build 12.1.5.70391.\n")
    file.write("//! Regenerate with `python3 tools/gen_map_challenge_modes.py <MapChallengeMode.csv>`.\n")
    file.write("pub static MAPS: &[(u32, u16)] = &[\n")
    for challenge, map_id in rows:
        file.write(f"    ({challenge}, {map_id}),\n")
    file.write("];\n")
print(f"Generated {len(rows)} map challenge records in {output}")
