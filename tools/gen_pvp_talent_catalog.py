#!/usr/bin/env python3
"""Project the Retail PvpTalent DB2 rows used by tooltip construction."""

import argparse
import csv
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data/pvp-talents/retail-12.1.5.70783"
OUTPUT = ROOT / "src/c_api/c_tooltip_info_pvp_talent_data.rs"
TABLES = {
    "PvpTalent": {
        "file_data_id": 1302853,
        "layout": 0xBE69BB5A,
        "fields": {
            "ID": "id",
            "SpellID": 3,
            "OverridesSpellID": 4,
            "PvpTalentCategoryID": 7,
            "LevelRequired": 8,
        },
    },
    "PvpTalentCategory": {
        "file_data_id": 1891534,
        "layout": 0xECB2C5C2,
        "fields": {"ID": "id", "TalentSlotMask": ("u8", 0)},
    },
}
FIELDS = ("ID", "SpellID", "OverridesSpellID", "LevelRequired", "TalentSlotMask")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_exporter(path):
    spec = importlib.util.spec_from_file_location("wow_ui_sim_db2_exporter", path)
    if spec is None or spec.loader is None:
        raise ValueError(f"cannot load DB2 exporter {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def validate_export_schema(exporter):
    for table, expected in TABLES.items():
        layout, columns = exporter.TABLES[table]
        if layout != expected["layout"]:
            raise ValueError(f"{table}: exporter layout {layout:08X} does not match pinned layout")
        actual = dict(columns)
        for name, source in expected["fields"].items():
            if actual.get(name) != source:
                raise ValueError(f"{table}.{name}: exporter field mapping changed")


def read_db2(path, table, exporter):
    data = path.read_bytes()
    if data[:4] != b"WDC5":
        raise ValueError(f"{path}: expected WDC5")
    layout = TABLES[table]["layout"]
    records, unreadable, _, _ = exporter.read_wdc5(
        data, layout, exporter.INLINE_ID_FIELD.get(table, 0)
    )
    if unreadable:
        raise ValueError(f"{path}: DB2 export dropped {unreadable} unreadable records")
    return records, hashlib.sha256(data).hexdigest()


def export_rows(talents_path, categories_path, exporter_path):
    exporter = load_exporter(exporter_path)
    validate_export_schema(exporter)
    talents, talent_hash = read_db2(talents_path, "PvpTalent", exporter)
    categories, category_hash = read_db2(categories_path, "PvpTalentCategory", exporter)
    rows = []
    for talent_id, (values, _, _) in talents.items():
        category_id = values[7]
        category = categories.get(category_id)
        if category is None:
            raise ValueError(f"PvpTalent {talent_id}: missing category {category_id}")
        mask = category[0][0]
        rows.append((talent_id, values[3], values[4], values[8], mask))
    rows.sort()
    validate_rows(rows)
    return rows, {
        "PvpTalent": {
            "file_data_id": TABLES["PvpTalent"]["file_data_id"],
            "layout": f"{TABLES['PvpTalent']['layout']:08X}",
            "sha256": talent_hash,
            "unreadable_records": 0,
        },
        "PvpTalentCategory": {
            "file_data_id": TABLES["PvpTalentCategory"]["file_data_id"],
            "layout": f"{TABLES['PvpTalentCategory']['layout']:08X}",
            "sha256": category_hash,
            "unreadable_records": 0,
        },
    }


def validate_rows(rows):
    if not rows:
        raise ValueError("PvpTalent export has no rows")
    seen = set()
    bounds = ((1, (1 << 31) - 1), (0, (1 << 32) - 1),
              (0, (1 << 32) - 1), (-(1 << 31), (1 << 31) - 1), (0, 255))
    for row in rows:
        if len(row) != len(FIELDS):
            raise ValueError("PvpTalent row has an unexpected field count")
        talent_id = row[0]
        if talent_id in seen:
            raise ValueError(f"duplicate talent ID {talent_id}")
        seen.add(talent_id)
        for name, value, (minimum, maximum) in zip(FIELDS, row, bounds):
            if not isinstance(value, int) or not minimum <= value <= maximum:
                raise ValueError(f"{name} value {value!r} is out of range")


def read_rows(path):
    with path.open(newline="", encoding="utf-8") as source:
        reader = csv.DictReader(source)
        if not set(FIELDS).issubset(reader.fieldnames or ()):
            raise ValueError(f"{path}: missing columns required for PvP talent catalog")
        rows = []
        for line, row in enumerate(reader, start=2):
            try:
                rows.append(tuple(int(row[field]) for field in FIELDS))
            except (TypeError, ValueError) as error:
                raise ValueError(f"{path}:{line}: invalid numeric PvP talent row") from error
    rows.sort()
    validate_rows(rows)
    return rows


def render_rust(rows, build, source_hash):
    validate_rows(rows)
    lines = [
        "// Generated by tools/gen_pvp_talent_catalog.py; do not edit.",
        f"// Local Retail build {build}; projected CSV SHA-256: {source_hash}.",
        "// (talent ID, SpellID, OverridesSpellID, LevelRequired, TalentSlotMask).",
        "pub(super) const ROWS: &[(i32, u32, u32, i32, u8)] = &[",
    ]
    lines.extend(f"    ({', '.join(str(value) for value in row)})," for row in rows)
    lines.extend(["];"])
    return "\n".join(lines) + "\n"


def write_csv(path, rows):
    with path.open("w", newline="", encoding="utf-8") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(FIELDS)
        writer.writerows(rows)


def write_provenance(path, args, records, exporter_path, row_count, csv_hash):
    exporter = load_exporter(exporter_path)
    columns = {
        table: [name for name, _ in exporter.TABLES[table][1]]
        for table in TABLES
    }
    provenance = {
        "profile": "retail",
        "product": "wow",
        "build": args.build,
        "build_key": args.build_key,
        "source": "local Retail CASC via wow-cli casc extract-fdid",
        "db2_tables": records,
        "exporter": {
            "path": "game-engine/scripts/export_db2_csv.py",
            "sha256": sha256(exporter_path),
            "schema_columns": columns,
            "schema_source": "existing exporter TABLES exact-layout mappings; pinned WoWDBDefs clone unavailable",
        },
        "projected_fields": list(FIELDS),
        "row_count": row_count,
        "projected_csv_sha256": csv_hash,
        "granted_by_item_and_eligibility_inputs": "not derived from DB2; host-only inputs remain neutral",
    }
    path.write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")


def generate(args):
    if args.build != "12.1.5.70783" or args.build_key != "bc493ecb2898766c0f8947a99b4b3d65":
        raise ValueError("this local CASC export is pinned to installed Retail 12.1.5.70783")
    rows, records = export_rows(args.talents_db2, args.categories_db2, args.exporter)
    output_dir = args.output_dir or DATA
    output_dir.mkdir(parents=True, exist_ok=True)
    csv_path = output_dir / "pvp-talents.csv"
    write_csv(csv_path, rows)
    csv_hash = sha256(csv_path)
    write_provenance(output_dir / "provenance.json", args, records,
                     args.exporter, len(rows), csv_hash)
    rust_path = args.rust_output or OUTPUT
    rust_path.write_text(render_rust(rows, args.build, csv_hash), encoding="utf-8", newline="\n")
    print(f"Generated {len(rows)} Retail PvP talent rows from {args.build}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--talents-db2", type=Path, required=True)
    parser.add_argument("--categories-db2", type=Path, required=True)
    parser.add_argument("--exporter", type=Path, required=True)
    parser.add_argument("--build", required=True)
    parser.add_argument("--build-key", required=True)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--rust-output", type=Path)
    generate(parser.parse_args())


if __name__ == "__main__":
    main()
