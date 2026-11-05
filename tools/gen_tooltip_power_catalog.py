#!/usr/bin/env python3
"""Generate the C_TooltipInfo power-catalog tables from a pinned DB2 export.

Usage: python3 tools/gen_tooltip_power_catalog.py <csv-dir>

The CSV directory must hold AzeritePower.csv, SoulbindConduit.csv,
SoulbindConduitRank.csv, SoulbindConduitRankProperties.csv, ArtifactPower.csv
and ArtifactPowerRank.csv from the same Retail build. The committed file records each input's sha256 so the data can be
reproduced from the pinned export.
"""

import csv
import hashlib
import pathlib
import sys

OUTPUT = pathlib.Path("src/c_api/tooltip_power_catalog_data.rs")
TABLES = (
    "AzeritePower",
    "SoulbindConduit",
    "SoulbindConduitRank",
    "SoulbindConduitRankProperties",
    "ArtifactPower",
    "ArtifactPowerRank",
)


def read_rows(csv_dir, table):
    path = csv_dir / f"{table}.csv"
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))


def sha256(csv_dir, table):
    return hashlib.sha256((csv_dir / f"{table}.csv").read_bytes()).hexdigest()


def uint(row, column):
    return int(row[column])


def write_rows(out, name, doc, rows, pattern):
    out.write(f"\n/// {doc}\n")
    out.write(f"pub static {name}: &[({pattern})] = &[\n")
    for row in rows:
        out.write(f"    ({row}),\n")
    out.write("];\n")


def generate(csv_dir):
    azerite = sorted(
        {
            (uint(row, "ID"), uint(row, "SpellID"))
            for row in read_rows(csv_dir, "AzeritePower")
            if uint(row, "SpellID") != 0
        }
    )
    conduits = sorted(
        (uint(row, "ID"), uint(row, "ConduitType"))
        for row in read_rows(csv_dir, "SoulbindConduit")
    )
    conduit_ranks = sorted(
        (
            uint(row, "SoulbindConduitID"),
            uint(row, "RankIndex"),
            uint(row, "SpellID"),
        )
        for row in read_rows(csv_dir, "SoulbindConduitRank")
    )
    rank_properties = sorted(
        (uint(row, "Rank"), uint(row, "ItemLevel"), uint(row, "QualityID"))
        for row in read_rows(csv_dir, "SoulbindConduitRankProperties")
    )
    # WM108 §A1: Flags carries the rank-1 (64), end-game (4), rank-limit (8),
    # bonus (16) and tier-scaled (32) masks read by the artifact-power getter.
    artifact_powers = sorted(
        (
            uint(row, "ID"),
            uint(row, "ArtifactID"),
            uint(row, "MaxPurchasableRank"),
            uint(row, "Flags"),
            uint(row, "Tier"),
        )
        for row in read_rows(csv_dir, "ArtifactPower")
    )
    artifact_power_ranks = sorted(
        (
            uint(row, "ArtifactPowerID"),
            uint(row, "RankIndex"),
            uint(row, "SpellID"),
        )
        for row in read_rows(csv_dir, "ArtifactPowerRank")
    )

    out = OUTPUT.open("w")
    out.write("//! Generated power-catalog rows for the `C_TooltipInfo` getters.\n")
    out.write("//! Do not edit manually; regenerate with\n")
    out.write("//! `python3 tools/gen_tooltip_power_catalog.py <pinned-csv-dir>`.\n")
    out.write("//!\n")
    out.write("//! Source: Retail 12.1.5.70391 enUS DB2 CSV export.\n")
    for table in TABLES:
        out.write(f"//! {table}.csv sha256={sha256(csv_dir, table)}\n")
    out.write("\n")
    write_rows(
        out,
        "AZERITE_POWER_SPELLS",
        "`(powerID, spellID)` rows with a nonzero spell ID.",
        azerite,
        "u32, u32",
    )
    write_rows(
        out,
        "CONDUIT_TYPES",
        "`(conduitID, ConduitType)`; types 0, 1 and 2 select the three labels.",
        conduits,
        "u32, u8",
    )
    write_rows(
        out,
        "CONDUIT_RANKS",
        "`(conduitID, RankIndex, spellID)`; RankIndex is zero-based.",
        conduit_ranks,
        "u32, u8, u32",
    )
    write_rows(
        out,
        "CONDUIT_RANK_PROPERTIES",
        "`(Rank 1-based, ItemLevel, QualityID)` used by both conduit getters.",
        rank_properties,
        "u8, u32, u8",
    )
    write_rows(
        out,
        "ARTIFACT_POWERS",
        "`(powerID, ArtifactID, MaxPurchasableRank, Flags, Tier)`.",
        artifact_powers,
        "u32, u32, u32, u32, u32",
    )
    write_rows(
        out,
        "ARTIFACT_POWER_RANKS",
        "`(powerID, RankIndex, spellID)`; RankIndex is zero-based.",
        artifact_power_ranks,
        "u32, u8, u32",
    )
    out.close()
    print(f"Wrote {OUTPUT}")


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    generate(pathlib.Path(sys.argv[1]))


if __name__ == "__main__":
    main()
