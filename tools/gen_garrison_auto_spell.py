#!/usr/bin/env python3
"""Generate GarrAutoSpell data from the pinned 70391 enUS CSV."""

import argparse
import csv
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path.home() / "Projects/wow/data/GarrAutoSpell.csv")
    parser.add_argument(
        "--output", type=Path,
        default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/garrison_auto_spell.rs",
    )
    args = parser.parse_args()
    required = ("ID", "Name_lang", "Description_lang", "Cooldown", "Duration", "Flags", "SchoolMask", "IconFileDataID")
    with args.input.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        if not reader.fieldnames or not set(required).issubset(reader.fieldnames):
            raise ValueError(f"GarrAutoSpell CSV missing required columns: {required}")
        rows = sorted(reader, key=lambda row: int(row["ID"]))
    ids = [int(row["ID"]) for row in rows]
    if not rows or len(ids) != len(set(ids)):
        raise ValueError("GarrAutoSpell CSV is empty or has duplicate IDs")
    lines = [
        "//! Generated from GarrAutoSpell.csv build 12.1.5.70391; run tools/gen_garrison_auto_spell.py.",
        "", "#[rustfmt::skip]",
        "pub static GARR_AUTO_SPELLS: &[(u32, &str, &str, u32, u32, u32, u32, u32)] = &[",
    ]
    for row in rows:
        name = json.dumps(row["Name_lang"], ensure_ascii=False)
        description = json.dumps(row["Description_lang"], ensure_ascii=False)
        numbers = [int(row[key]) for key in ("Cooldown", "Duration", "Flags", "SchoolMask", "IconFileDataID")]
        lines.append(f"    ({int(row['ID'])}, {name}, {description}, {', '.join(map(str, numbers))}),")
    lines.extend(["];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} GarrAutoSpell rows in {args.output}")


if __name__ == "__main__":
    main()
