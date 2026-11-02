#!/usr/bin/env python3
"""Generate item specialization inputs from build 12.1.5.70391 DB2 CSVs."""

import csv
from pathlib import Path

DATA = Path.home() / "Projects/wow/data"
OUTPUT = Path(__file__).resolve().parents[1] / "crates/sim-data/data/item_spec_inputs.rs"


def read_rows(name):
    with (DATA / f"{name}.csv").open(newline="", encoding="utf-8") as source:
        return list(csv.DictReader(source))


def generate():
    items = {int(row["ID"]): row for row in read_rows("Item")}
    gems = {int(row["ID"]): int(row["Type"]) for row in read_rows("GemProperties")}
    sparse = sorted(read_rows("ItemSparse"), key=lambda row: int(row["ID"]))
    overrides = sorted(read_rows("ItemSpecOverride"), key=lambda row: int(row["ItemID"]))
    specs = sorted(read_rows("ChrSpecialization"), key=lambda row: int(row["ID"]))
    with OUTPUT.open("w", encoding="utf-8", newline="\n") as out:
        out.write("//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS).\n")
        out.write("//! Regenerate with `python3 tools/gen_item_spec_data.py`.\n")
        out.write("pub static ITEMS: &[(u32, u8, u8, u8, i32, u32, [i8; 10])] = &[\n")
        for row in sparse:
            item_id = int(row["ID"])
            item = items.get(item_id)
            if item is None:
                continue
            stats = [int(row[f"StatModifier_bonusStat_{index}"]) for index in range(10)]
            gem_type = gems.get(int(row["Gem_properties"]), 0)
            out.write(
                f"    ({item_id}, {item['ClassID']}, {item['SubclassID']}, "
                f"{item['InventoryType']}, {row['AllowableClass']}, {gem_type}, {stats}),\n"
            )
        out.write("];\n\npub static RULES: &[(u8, u8, u8, u8, u8, u32)] = &[\n")
        for row in read_rows("ItemSpec"):
            out.write(
                f"    ({row['MinLevel']}, {row['MaxLevel']}, {row['ItemType']}, "
                f"{row['PrimaryStat']}, {row['SecondaryStat']}, {row['SpecializationID']}),\n"
            )
        out.write("];\n\npub static OVERRIDES: &[(u32, u32)] = &[\n")
        for row in overrides:
            out.write(f"    ({row['ItemID']}, {row['SpecID']}),\n")
        out.write("];\n\npub static SPEC_CLASSES: &[(u32, u8)] = &[\n")
        for row in specs:
            out.write(f"    ({row['ID']}, {row['ClassID']}),\n")
        out.write("];\n")


if __name__ == "__main__":
    generate()
