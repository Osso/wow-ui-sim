#!/usr/bin/env python3
"""Generate follower-level XP data from the pinned GarrFollowerLevelXP CSV."""

import argparse
import csv
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path.home() / "Projects/wow/data/GarrFollowerLevelXP.csv")
    parser.add_argument(
        "--output", type=Path,
        default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/garrison_follower_xp.rs",
    )
    args = parser.parse_args()
    rows = {}
    with args.input.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        required = {"GarrFollowerTypeID", "FollowerLevel", "XpToNextLevel"}
        if not reader.fieldnames or not required.issubset(reader.fieldnames):
            raise ValueError(f"GarrFollowerLevelXP CSV missing required columns: {required}")
        for row in reader:
            key = (int(row["GarrFollowerTypeID"]), int(row["FollowerLevel"]))
            xp = int(row["XpToNextLevel"])
            if key in rows and rows[key] != xp:
                raise ValueError(f"conflicting follower XP rows for {key}")
            rows[key] = xp
    if not rows:
        raise ValueError("GarrFollowerLevelXP CSV is empty")
    lines = [
        "//! Generated from GarrFollowerLevelXP.csv build 12.1.5.70391; run tools/gen_garrison_follower_xp.py.",
        "", "pub static FOLLOWER_LEVEL_XP: &[(i32, i32, i32)] = &[",
    ]
    lines.extend(f"    ({follower}, {level}, {xp})," for (follower, level), xp in sorted(rows.items()))
    lines.extend(["];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} follower level XP rows in {args.output}")


if __name__ == "__main__":
    main()
