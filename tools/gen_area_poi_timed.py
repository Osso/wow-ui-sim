#!/usr/bin/env python3
"""Generate static timed-POI classification from pinned client CSV exports."""

import argparse
import csv
from pathlib import Path

POI_CONDITION_COLUMN = "PlayerConditionID"
POI_FLAGS_COLUMN = "Flags"
CONDITION_EXPRESSION_COLUMN = "WorldStateExpressionID"


def read_rows(path):
    with path.open(newline="", encoding="utf-8-sig") as source:
        return list(csv.DictReader(source))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--area-poi", type=Path, required=True)
    parser.add_argument("--player-condition", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    conditions = {
        int(row["ID"]): int(row[CONDITION_EXPRESSION_COLUMN])
        for row in read_rows(args.player_condition)
    }
    rows = []
    for row in read_rows(args.area_poi):
        condition = int(row[POI_CONDITION_COLUMN])
        if conditions.get(condition, 0) != 0:
            rows.append((int(row["ID"]), bool(int(row[POI_FLAGS_COLUMN]) & (1 << 22))))
    if len({row[0] for row in rows}) != len(rows):
        raise ValueError("duplicate timed POI ID")
    lines = [
        "//! PROVENANCE: wago.tools AreaPOI and PlayerCondition 12.1.5.70391 enUS.",
        "//! AreaPOI SHA256: 22961175efd3e12c33e6543a8672cf56f37a1f92e5b21e81608edeaa13fa3e56",
        "//! PlayerCondition SHA256: 5d33dde2303b6fe5287c2f1a6d70e80f4b5157a4538b5b2d401f50a86e2b7658",
        "//! Record equality with the 70688 native evidence is unknown.",
        "//! Selection: referenced condition exists and its expression field is nonzero.",
        "",
        "pub static TIMED_AREA_POIS: &[(i32, bool)] = &[",
    ]
    lines.extend(f"    ({poi}, {str(hidden).lower()})," for poi, hidden in sorted(rows))
    lines.extend(["];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} timed area POIs")


if __name__ == "__main__":
    main()
