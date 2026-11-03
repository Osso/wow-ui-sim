#!/usr/bin/env python3
"""Generate Profession rows (ID, SkillLineID, ProfessionEnumValue) from build 12.1.5.70391."""

import csv
import hashlib
from pathlib import Path

DATA = Path.home() / "Projects/wow/data"
OUTPUT = Path(__file__).resolve().parents[1] / "crates/sim-data/data/profession_rows.rs"


def generate():
    source = DATA / "Profession.csv"
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    with source.open(newline="", encoding="utf-8") as handle:
        rows = sorted(csv.DictReader(handle), key=lambda row: int(row["ID"]))
    with OUTPUT.open("w", encoding="utf-8", newline="\n") as out:
        out.write("//! Generated from wago.tools DB2 CSV export, build 12.1.5.70391 (enUS).\n")
        out.write(f"//! Profession.csv SHA-256: {digest}.\n")
        out.write("//! Regenerate with `python3 tools/gen_profession_rows.py`.\n")
        out.write("pub static PROFESSION_ROWS: &[(u32, u32, u32)] = &[\n")
        for row in rows:
            out.write(f"    ({row['ID']}, {row['SkillLineID']}, {row['ProfessionEnumValue']}),\n")
        out.write("];\n")
    print(f"Generated {len(rows)} Profession rows")


if __name__ == "__main__":
    generate()
