#!/usr/bin/env python3
"""Generate Perks vendor category data from a pinned client CSV export."""

import argparse
import csv
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path.home() / "Projects/wow/data/PerksVendorCategory.csv")
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/perks_categories.rs")
    args = parser.parse_args()
    with args.input.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        required = {"ID", "DisplayName_lang", "DefaultUIModelSceneID"}
        if not reader.fieldnames or not required.issubset(reader.fieldnames):
            raise ValueError(f"missing required columns: {required}")
        rows = [(int(r["ID"]), r["DisplayName_lang"], int(r["DefaultUIModelSceneID"])) for r in reader]
    if not rows or len({row[0] for row in rows}) != len(rows):
        raise ValueError("empty category catalogue or duplicate category IDs")
    lines = ["//! Generated from PerksVendorCategory.csv build 12.1.5.70391.", "", "pub static CATEGORIES: &[(i32, &str, i32)] = &["]
    lines.extend(f"    ({id}, {json.dumps(name, ensure_ascii=False)}, {scene})," for id, name, scene in sorted(rows))
    lines.extend(["];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} Perks vendor categories")


if __name__ == "__main__":
    main()
