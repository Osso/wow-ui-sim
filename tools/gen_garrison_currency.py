#!/usr/bin/env python3
"""Generate the garrison currency catalogue from the pinned GarrType CSV."""

import argparse
import csv
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path.home() / "Projects/wow/data/GarrType.csv")
    parser.add_argument(
        "--output", type=Path,
        default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/garrison_currency.rs",
    )
    args = parser.parse_args()
    with args.input.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        required = {"ID", "PrimaryCurrencyTypeID", "SecondaryCurrencyTypeID"}
        if not reader.fieldnames or not required.issubset(reader.fieldnames):
            raise ValueError(f"GarrType CSV missing required columns: {required}")
        rows = sorted(
            (int(row["ID"]), int(row["PrimaryCurrencyTypeID"]), int(row["SecondaryCurrencyTypeID"]))
            for row in reader
        )
    if not rows or len({row[0] for row in rows}) != len(rows):
        raise ValueError("GarrType CSV is empty or has duplicate IDs")
    lines = [
        "//! Generated from GarrType.csv build 12.1.5.70391; run tools/gen_garrison_currency.py.",
        "", "pub static GARRISON_CURRENCIES: &[(i32, i32, i32)] = &[",
    ]
    lines.extend(f"    ({garrison}, {primary}, {secondary})," for garrison, primary, secondary in rows)
    lines.extend([
        "];", "",
        "pub fn currency_types(garrison: i32) -> Option<(i32, i32)> {",
        "    let index = GARRISON_CURRENCIES",
        "        .binary_search_by_key(&garrison, |row| row.0)",
        "        .ok()?;",
        "    let (_, primary, secondary) = GARRISON_CURRENCIES[index];",
        "    Some((primary, secondary))", "}", "",
    ])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} garrison currency rows in {args.output}")


if __name__ == "__main__":
    main()
