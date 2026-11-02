#!/usr/bin/env python3
"""Generate 70391 CurrencyContainer bounds from the supplied DB2 CSV export."""

import argparse
import csv
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("csv", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    with args.csv.open(newline="", encoding="utf-8") as source:
        rows = sorted(
            (int(row["CurrencyTypeID"]), int(row["MinAmount"]), int(row["MaxAmount"]))
            for row in csv.DictReader(source)
        )
    output = [
        "//! Generated from CurrencyContainer.csv build 12.1.5.70391; column names inferred.",
        "pub static CURRENCY_CONTAINERS: &[(i32, i32, i32)] = &[",
    ]
    output.extend(f"    ({currency}, {minimum}, {maximum})," for currency, minimum, maximum in rows)
    output.append("];")
    args.output.write_text("\n".join(output) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
