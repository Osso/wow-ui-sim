#!/usr/bin/env python3
"""Generate currency transfer inputs from matching CurrencyTypes and CurrencyCategory CSVs."""
import argparse
import csv
from pathlib import Path


def read_rows(path):
    with path.open(newline="", encoding="utf-8") as source:
        return list(csv.DictReader(source))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("currency_types", type=Path)
    parser.add_argument("categories", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    categories = {int(row["ID"]): int(row["Flags"]) for row in read_rows(args.categories)}
    rows = []
    for row in read_rows(args.currency_types):
        category = int(row["CategoryID"])
        rows.append(
            (int(row["ID"]), int(row["Flags_0"]) & 0xFFFFFFFF,
             categories[category], float(row["WarbondTransferPercentage"]))
        )
    rows.sort()
    output = [
        "//! Generated from CurrencyTypes and CurrencyCategory CSVs, build 12.1.5.70391.",
        "//! WarbondTransferPercentage column identification is inferred (WM64).",
        "pub static CURRENCY_TRANSFER: &[(i32, u32, u32, f64)] = &[",
    ]
    output.extend(
        f"    ({currency_id}, {flags}, {category_flags}, {percentage!r}),"
        for currency_id, flags, category_flags, percentage in rows
    )
    output.extend([
        "];", 
        "",
        "pub fn get(id: i32) -> Option<(u32, u32, f64)> {",
        "    CURRENCY_TRANSFER.binary_search_by_key(&id, |row| row.0).ok()",
        "        .map(|index| { let (_, flags, category, percentage) = CURRENCY_TRANSFER[index]; (flags, category, percentage) })",
        "}",
    ])
    args.output.write_text("\n".join(output) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
