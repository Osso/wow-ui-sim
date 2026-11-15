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
        if not reader.fieldnames or not set(required_columns).issubset(
            reader.fieldnames
        ):
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
        raise ValueError(
            f"invalid {table}.{column} value: {row.get(column)!r}"
        ) from error


def read_catalog_sources():
    essence = read_table("AzeriteEssence", ESSENCE_COLUMNS)
    power = read_table("AzeriteEssencePower", POWER_COLUMNS)
    return (*essence, *power)


def build_essences(rows):
    essences = sorted(
        (read_integer(row, "AzeriteEssence", "ID"), row["Name_lang"]) for row in rows
    )
    if len({essence_id for essence_id, _ in essences}) != len(essences):
        raise ValueError("AzeriteEssence.csv has duplicate IDs")
    return essences


def build_powers(rows):
    return sorted(
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
        for row in rows
    )


def validate_power_records(essences, powers):
    power_keys = [(essence_id, rank) for essence_id, rank, *_ in powers]
    if len(set(power_keys)) != len(power_keys):
        raise ValueError("AzeriteEssencePower.csv has duplicate essence/rank rows")
    essence_ids = {essence_id for essence_id, _ in essences}
    unknown = sorted({essence_id for essence_id, *_ in powers} - essence_ids)
    if unknown:
        raise ValueError(f"AzeriteEssencePower.csv references unknown IDs: {unknown}")


def render_header(essence_hash, power_hash):
    return [
        "//! Generated Azerite Essence tooltip catalog. Do not edit manually.",
        "//! Regenerate with `python3 tools/gen_tooltip_azerite_catalog.py`.",
        f"//! Source: Wago DB2 exact-build CSV exports for Retail {BUILD} ({LOCALE}).",
        "//! Spell columns keep their DB2 names; selector mappings are separate.",
        "//! Source text columns are preserved but unused; faction selection is UNKNOWN.",
        f"//! AzeriteEssence.csv SHA-256: {essence_hash}.",
        f"//! AzeriteEssencePower.csv SHA-256: {power_hash}.",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct AzeriteEssenceCatalogRecord {",
        "    pub(crate) id: u32,",
        "    pub(crate) name: &'static str,",
        "}",
        "",
        "pub(crate) static AZERITE_ESSENCES: &[AzeriteEssenceCatalogRecord] = &[",
    ]


def render_essence_record(record):
    essence_id, name = record
    return [
        "    AzeriteEssenceCatalogRecord {",
        f"        id: {essence_id},",
        f"        name: {rust_string(name)},",
        "    },",
    ]


def render_essence_section(essences):
    lines = []
    for record in essences:
        lines.extend(render_essence_record(record))
    return lines + ["];"]


def render_power_record(record):
    (
        essence_id,
        rank,
        source_alliance,
        source_horde,
        major_description,
        minor_description,
        major_actual,
        minor_actual,
    ) = record
    return [
        "    AzeriteEssencePowerRecord {",
        f"        essence_id: {essence_id},",
        f"        rank: {rank},",
        f"        _source_alliance: {rust_string(source_alliance)},",
        f"        _source_horde: {rust_string(source_horde)},",
        f"        major_power_description: {major_description},",
        f"        minor_power_description: {minor_description},",
        f"        major_power_actual: {major_actual},",
        f"        minor_power_actual: {minor_actual},",
        "    },",
    ]


def render_power_section(powers):
    lines = [
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct AzeriteEssencePowerRecord {",
        "    pub(crate) essence_id: u32,",
        "    pub(crate) rank: u32,",
        "    pub(crate) _source_alliance: &'static str,",
        "    pub(crate) _source_horde: &'static str,",
        "    pub(crate) major_power_description: u32,",
        "    pub(crate) minor_power_description: u32,",
        "    pub(crate) major_power_actual: u32,",
        "    pub(crate) minor_power_actual: u32,",
        "}",
        "",
        "pub(crate) static AZERITE_ESSENCE_POWERS: &[AzeriteEssencePowerRecord] = &[",
    ]
    for record in powers:
        lines.extend(render_power_record(record))
    return lines + ["];"]


def render_catalog(essences, powers, essence_hash, power_hash):
    lines = render_header(essence_hash, power_hash)
    lines.extend(render_essence_section(essences))
    lines.extend(["", *render_power_section(powers)])
    return "\n".join(lines) + "\n"


def build_table_provenance(table, path, columns, rows):
    return {
        "url": f"https://wago.tools/db2/{table}/csv?build={BUILD}&locale={LOCALE}",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "rows": len(rows),
        "columns": columns,
    }


def build_provenance(essence, power, generated):
    essence_path, essence_columns, essence_rows = essence
    power_path, power_columns, power_rows = power
    return {
        "build": BUILD,
        "locale": LOCALE,
        "source": "Wago DB2 exact-build CSV exports",
        "tables": {
            "AzeriteEssence.csv": build_table_provenance(
                "AzeriteEssence", essence_path, essence_columns, essence_rows
            ),
            "AzeriteEssencePower.csv": build_table_provenance(
                "AzeriteEssencePower", power_path, power_columns, power_rows
            ),
        },
        "generated_file": str(OUTPUT.relative_to(ROOT)),
        "generated_sha256": hashlib.sha256(generated.encode("utf-8")).hexdigest(),
    }


def generate():
    (
        essence_path,
        essence_columns,
        essence_rows,
        power_path,
        power_columns,
        power_rows,
    ) = read_catalog_sources()
    essence = essence_path, essence_columns, essence_rows
    power = power_path, power_columns, power_rows
    essences = build_essences(essence_rows)
    powers = build_powers(power_rows)
    validate_power_records(essences, powers)
    essence_hash = hashlib.sha256(essence_path.read_bytes()).hexdigest()
    power_hash = hashlib.sha256(power_path.read_bytes()).hexdigest()
    generated = render_catalog(essences, powers, essence_hash, power_hash)
    OUTPUT.write_text(generated, encoding="utf-8", newline="\n")
    provenance = build_provenance(essence, power, generated)
    PROVENANCE.write_text(
        json.dumps(provenance, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(
        f"Wrote {OUTPUT.relative_to(ROOT)} ({len(essences)} essences, {len(powers)} ranks)"
    )
    print(f"Wrote {PROVENANCE.relative_to(ROOT)}")


if __name__ == "__main__":
    generate()
