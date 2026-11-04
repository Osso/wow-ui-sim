#!/usr/bin/env python3
"""Generate descriptor records from the pinned 70391 CSV exports."""
import argparse
import csv
import hashlib
from pathlib import Path

BONUS_SHA = "bbb5dc694d41d2862432c18b637c327d31e43945c58b8e519dd33b9b6375d9f8"
SPARSE_SHA = "b370bc7d3ac06e2142824da92311bcf0542740e6ffefef240103cd12bcb19a40"
SCALING_FILES = (
    "ItemScalingConfig",
    "ItemOffsetCurve",
    "ItemSquishEra",
    "ContentTuning",
    "Curve",
    "CurvePoint",
)
SPARSE_SELECTORS = (
    "PlayerLevelToItemLevelCurveID",
    "ItemLevelOffsetCurveID",
    "ContentTuningID",
    "ItemSquishEraID",
)


def read_csv(path, expected=None):
    if expected and hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise ValueError(f"{path.name}: not the pinned 70391 export")
    with path.open(newline="") as source:
        return list(csv.DictReader(source))


def write_tsv(path, header, rows):
    body = [header] + [[str(value) for value in row] for row in rows]
    path.write_text("".join("\t".join(row) + "\n" for row in body))


def generate(csv_root, scaling_root, output):
    output.mkdir(parents=True, exist_ok=True)
    rows = read_csv(csv_root / "ItemBonus.csv", BONUS_SHA)
    # Stable CSV ordering for equal OrderIndex is an INFERRED seam, not native proof.
    rows.sort(key=lambda row: (int(row["ParentItemBonusListID"]), int(row["OrderIndex"])))
    fields = ["ParentItemBonusListID", "ID", "OrderIndex", "Type"] + [f"Value_{n}" for n in range(4)]
    write_tsv(output / "bonus.tsv", fields, ([row[key] for key in fields] for row in rows))

    sparse = read_csv(csv_root / "ItemSparse.csv", SPARSE_SHA)
    # Zero records need no entry: these columns are unsigned selectors, not presence flags.
    selectors = sorted(
        (row for row in sparse if any(int(row[key]) for key in SPARSE_SELECTORS)),
        key=lambda row: int(row["ID"]),
    )
    write_tsv(
        output / "scaling-selectors.tsv",
        ("ID", *SPARSE_SELECTORS, "ItemLevelOffsetItemLevel"),
        ([row[key] for key in ("ID", *SPARSE_SELECTORS, "ItemLevelOffsetItemLevel")] for row in selectors),
    )

    scaling = {name: read_csv(scaling_root / f"{name}.csv") for name in SCALING_FILES}
    offset_curves = {row["ID"]: row for row in scaling["ItemOffsetCurve"]}
    reachable = {row["PlayerLevelToItemLevelCurveID"] for row in selectors}
    reachable |= {row["ItemLevelOffsetCurveID"] for row in sparse}
    reachable |= {row["CurveID"] for row in offset_curves.values()}
    reachable |= {row["CurveID"] for row in scaling["ItemSquishEra"]}
    reachable.discard("0")
    write_tsv(
        output / "curves.tsv",
        ("ID", "Type"),
        ((row["ID"], row["Type"]) for row in scaling["Curve"] if row["ID"] in reachable),
    )
    # Only points of reachable curves are needed; unreachable curves cannot be selected.
    points = [row for row in scaling["CurvePoint"] if row["CurveID"] in reachable]
    points.sort(key=lambda row: (int(row["CurveID"]), int(row["OrderIndex"])))
    write_tsv(
        output / "curve-points.tsv",
        ("CurveID", "OrderIndex", "Pos_0", "Pos_1"),
        ([row[key] for key in ("CurveID", "OrderIndex", "Pos_0", "Pos_1")] for row in points),
    )
    write_tsv(
        output / "offset-curves.tsv",
        ("ID", "CurveID", "Offset"),
        ((row["ID"], row["CurveID"], row["Offset"]) for row in offset_curves.values()),
    )
    write_tsv(
        output / "scaling-configs.tsv",
        ("ID", "ItemOffsetCurveID", "ItemLevel", "RequiredLevel", "ItemSquishEraID", "Flags"),
        (
            [row[key] for key in ("ID", "ItemOffsetCurveID", "ItemLevel", "RequiredLevel", "ItemSquishEraID", "Flags")]
            for row in scaling["ItemScalingConfig"]
        ),
    )
    write_tsv(
        output / "squish-eras.tsv",
        ("ID", "Patch", "CurveID", "Flags"),
        ([row[key] for key in ("ID", "Patch", "CurveID", "Flags")] for row in scaling["ItemSquishEra"]),
    )
    write_tsv(
        output / "content-tuning.tsv",
        ("ID", "Flags", "MinLevelSquish", "MaxLevelSquish", "MinLevelScalingOffset", "MaxLevelScalingOffset", "AllowedMinOffset"),
        (
            [row[key] for key in ("ID", "Flags", "MinLevelSquish", "MaxLevelSquish", "MinLevelScalingOffset", "MaxLevelScalingOffset", "AllowedMinOffset")]
            for row in scaling["ContentTuning"]
        ),
    )
    (output / "PROVENANCE.md").write_text(PROVENANCE.format(bonus=BONUS_SHA, sparse=SPARSE_SHA))


PROVENANCE = """# Descriptor data

Pinned wago.tools exports: 12.1.5.70391. Item bonus/sparse records from the
shared CSV export directory; scaling tables from `data/db2/70391` (commit
`887cc18a5`).

- ItemBonus.csv SHA256 `{bonus}`
- ItemSparse.csv SHA256 `{sparse}`

Regenerate with `python3 tools/gen_item_descriptor_data.py --csv-root PATH
--scaling-root PATH`. Bonus rows retain list membership and ascending
OrderIndex within each list. WM97c verifies the 70391 base export has no repeated
OrderIndex within a parent while the native sort is not stable for large ranges;
the reader raises a named gap if a duplicate appears.

Curves are limited to those referenced by ItemSparse selectors, ItemOffsetCurve
and ItemSquishEra. Curve 121405/121406 and their points are absent from the
export; any lookup that reaches an absent curve raises a named gap error.
Scaling selectors are not themselves scaling records.
"""


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--csv-root", type=Path, required=True)
    parser.add_argument("--scaling-root", type=Path, required=True)
    args = parser.parse_args()
    generate(
        args.csv_root,
        args.scaling_root,
        Path(__file__).resolve().parents[1] / "data/item-descriptor/70391",
    )
