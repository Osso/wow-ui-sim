#!/usr/bin/env python3
"""Generate descriptor records from the pinned 70391 CSV exports."""
import argparse
import csv
import hashlib
from pathlib import Path

BONUS_SHA = "bbb5dc694d41d2862432c18b637c327d31e43945c58b8e519dd33b9b6375d9f8"
SPARSE_SHA = "b370bc7d3ac06e2142824da92311bcf0542740e6ffefef240103cd12bcb19a40"


def read_rows(root, name, expected):
    path = root / (name + ".csv")
    if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise ValueError(f"{name}: not the pinned 70391 export")
    with path.open(newline="") as source:
        return list(csv.DictReader(source))


def generate(root, output):
    output.mkdir(parents=True, exist_ok=True)
    rows = read_rows(root, "ItemBonus", BONUS_SHA)
    # Stable CSV ordering for equal OrderIndex is an INFERRED seam, not native proof.
    rows.sort(key=lambda row: (int(row["ParentItemBonusListID"]), int(row["OrderIndex"])))
    fields = ["ParentItemBonusListID", "ID", "Type", "OrderIndex"] + [f"Value_{n}" for n in range(4)]
    (output / "bonus.tsv").write_text("".join("\t".join(row[key] for key in fields) + "\n" for row in rows))
    fields = ["ID", "PlayerLevelToItemLevelCurveID", "ItemLevelOffsetCurveID", "ContentTuningID", "ItemSquishEraID"]
    rows = read_rows(root, "ItemSparse", SPARSE_SHA)
    # Zero records need no entry: these columns are unsigned selectors, not presence flags.
    rows = sorted((row for row in rows if any(int(row[key]) for key in fields[1:])), key=lambda row: int(row["ID"]))
    (output / "scaling-selectors.tsv").write_text("".join("\t".join(row[key] for key in fields) + "\n" for row in rows))
    (output / "PROVENANCE.md").write_text(
        "# Descriptor data\n\nPinned wago.tools exports: 12.1.5.70391.\n\n"
        f"- ItemBonus.csv SHA256 `{BONUS_SHA}`\n- ItemSparse.csv SHA256 `{SPARSE_SHA}`\n\n"
        "Regenerate with `python3 tools/gen_item_descriptor_data.py --csv-root PATH`. "
        "Bonus rows retain list membership and ascending OrderIndex within each list. "
        "Equal-OrderIndex CSV stability is INFERRED; native tie order is UNKNOWN. "
        "Scaling selectors are not scaling records; unresolved scaling must error.\n"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--csv-root", type=Path, required=True)
    args = parser.parse_args()
    generate(args.csv_root, Path(__file__).resolve().parents[1] / "data/item-descriptor/70391")
