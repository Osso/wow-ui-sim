#!/usr/bin/env python3
"""Generate the Azerite Essence tooltip catalog from exact-build DB2 CSV exports."""

import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUILD = "12.1.5.70391"
LOCALE = "enUS"
DB2_DIR = ROOT / "data" / "db2" / f"retail-{BUILD}"
OUTPUT = ROOT / "src" / "c_api" / "tooltip_azerite_catalog_data.rs"
PROVENANCE = DB2_DIR / "tooltip-azerite-catalog-provenance.json"
ESSENCE_COLUMNS = ("ID", "Name_lang")
POWER_COLUMNS = (
    "AzeriteEssenceID",
    "Tier",
    "SourceAlliance_lang",
    "SourceHorde_lang",
    "MajorPowerDescription",
    "MinorPowerDescription",
    "MajorPowerActual",
    "MinorPowerActual",
)


def read_table(table, required_columns):
    path = DB2_DIR / f"{table}.csv"
    with path.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        if not reader.fieldnames or not set(required_columns).issubset(reader.fieldnames):
            missing = sorted(set(required_columns) - set(reader.fieldnames or ()))
            raise ValueError(f"{path} is missing columns: {missing}")
        rows = list(reader)
    if not rows:
        raise ValueError(f"{path} contains no records")
    return path, reader.fieldnames, rows


def rust_string(value):
    escaped = []
    for character in value:
        codepoint = ord(character)
        if character == "\\":
            escaped.append("\\\\")
        elif character == '"':
            escaped.append('\\"')
        elif character == "\n":
            escaped.append("\\n")
        elif character == "\r":
            escaped.append("\\r")
        elif character == "\t":
            escaped.append("\\t")
        elif codepoint < 0x20 or codepoint == 0x7F:
            escaped.append(f"\\u{{{codepoint:x}}}")
        else:
            escaped.append(character)
    return '"' + "".join(escaped) + '"'


def read_integer(row, table, column):
    try:
        return int(row[column])
    except (KeyError, ValueError) as error:
        raise ValueError(f"invalid {table}.{column} value: {row.get(column)!r}") from error


def generate():
    essence_path, essence_columns, essence_rows = read_table(
        "AzeriteEssence", ESSENCE_COLUMNS
    )
    power_path, power_columns, power_rows = read_table(
        "AzeriteEssencePower", POWER_COLUMNS
    )

    essences = sorted(
        (read_integer(row, "AzeriteEssence", "ID"), row["Name_lang"])
        for row in essence_rows
    )
    if len({essence_id for essence_id, _ in essences}) != len(essences):
        raise ValueError("AzeriteEssence.csv has duplicate IDs")

    powers = sorted(
        (
            read_integer(row, "AzeriteEssencePower", "AzeriteEssenceID"),
            read_integer(row, "AzeriteEssencePower", "Tier"),
            row["SourceAlliance_lang"],
            row["SourceHorde_lang"],
            read_integer(row, "AzeriteEssencePower", "MajorPowerDescription"),
            read_integer(row, "AzeriteEssencePower", "MinorPowerDescription"),
            read_integer(row, "AzeriteEssencePower", "MajorPowerActual"),
            read_integer(row, "AzeriteEssencePower", "MinorPowerActual"),
        )
        for row in power_rows
    )
    power_keys = [(essence_id, rank) for essence_id, rank, *_ in powers]
    if len(set(power_keys)) != len(power_keys):
        raise ValueError("AzeriteEssencePower.csv has duplicate essence/rank rows")
    essence_ids = {essence_id for essence_id, _ in essences}
    unknown_essences = sorted({essence_id for essence_id, *_ in powers} - essence_ids)
    if unknown_essences:
        raise ValueError(f"AzeriteEssencePower.csv references unknown IDs: {unknown_essences}")

    lines = [
        "//! Generated Azerite Essence tooltip catalog. Do not edit manually.",
        "//! Regenerate with `python3 tools/gen_tooltip_azerite_catalog.py`.",
        f"//! Source: Wago DB2 exact-build CSV exports for Retail {BUILD} ({LOCALE}).",
        "//! DB2 spell and source fields retain their names; selector mappings are separate.",
        "//! The source faction selector is INFERRED from modeled player faction.",
        f"//! AzeriteEssence.csv SHA-256: {hashlib.sha256(essence_path.read_bytes()).hexdigest()}.",
        f"//! AzeriteEssencePower.csv SHA-256: {hashlib.sha256(power_path.read_bytes()).hexdigest()}.",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct AzeriteEssenceCatalogRecord {",
        "    pub(crate) id: u32,",
        "    pub(crate) name: &'static str,",
        "}",
        "",
        "pub(crate) static AZERITE_ESSENCES: &[AzeriteEssenceCatalogRecord] = &[",
    ]
    for essence_id, name in essences:
        lines.extend(
            [
                "    AzeriteEssenceCatalogRecord {",
                f"        id: {essence_id},",
                f"        name: {rust_string(name)},",
                "    },",
            ]
        )
    lines.extend(
        [
            "];",
            "",
            "#[derive(Clone, Copy, Debug)]",
            "pub(crate) struct AzeriteEssencePowerRecord {",
            "    pub(crate) essence_id: u32,",
            "    pub(crate) rank: u32,",
            "    pub(crate) source_alliance: &'static str,",
            "    pub(crate) source_horde: &'static str,",
            "    pub(crate) major_power_description: u32,",
            "    pub(crate) minor_power_description: u32,",
            "    pub(crate) major_power_actual: u32,",
            "    pub(crate) minor_power_actual: u32,",
            "}",
            "",
            "pub(crate) static AZERITE_ESSENCE_POWERS: &[AzeriteEssencePowerRecord] = &[",
        ]
    )
    for (
        essence_id,
        rank,
        source_alliance,
        source_horde,
        major_description,
        minor_description,
        major_actual,
        minor_actual,
    ) in powers:
        lines.extend(
            [
                "    AzeriteEssencePowerRecord {",
                f"        essence_id: {essence_id},",
                f"        rank: {rank},",
                f"        source_alliance: {rust_string(source_alliance)},",
                f"        source_horde: {rust_string(source_horde)},",
                f"        major_power_description: {major_description},",
                f"        minor_power_description: {minor_description},",
                f"        major_power_actual: {major_actual},",
                f"        minor_power_actual: {minor_actual},",
                "    },",
            ]
        )
    lines.append("];")
    generated = "\n".join(lines) + "\n"
    OUTPUT.write_text(generated, encoding="utf-8", newline="\n")

    provenance = {
        "build": BUILD,
        "locale": LOCALE,
        "source": "Wago DB2 exact-build CSV exports",
        "tables": {
            "AzeriteEssence.csv": {
                "url": f"https://wago.tools/db2/AzeriteEssence/csv?build={BUILD}&locale={LOCALE}",
                "sha256": hashlib.sha256(essence_path.read_bytes()).hexdigest(),
                "rows": len(essence_rows),
                "columns": essence_columns,
            },
            "AzeriteEssencePower.csv": {
                "url": f"https://wago.tools/db2/AzeriteEssencePower/csv?build={BUILD}&locale={LOCALE}",
                "sha256": hashlib.sha256(power_path.read_bytes()).hexdigest(),
                "rows": len(power_rows),
                "columns": power_columns,
            },
        },
        "generated_file": str(OUTPUT.relative_to(ROOT)),
        "generated_sha256": hashlib.sha256(OUTPUT.read_bytes()).hexdigest(),
    }
    PROVENANCE.write_text(
        json.dumps(provenance, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(f"Wrote {OUTPUT.relative_to(ROOT)} ({len(essences)} essences, {len(powers)} ranks)")
    print(f"Wrote {PROVENANCE.relative_to(ROOT)}")


if __name__ == "__main__":
    generate()
