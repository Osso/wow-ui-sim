#!/usr/bin/env python3
"""Generate profile-scoped character-select data from local DB2 files."""
import argparse
import ast
import csv
import hashlib
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile

DEFINITIONS_REVISION = "d6f50d4f351193e84226aa62a36faa0f58528afa"
PROFILES = {
    "retail": {
        "product": "wow",
        "label": "Retail",
        "build": "12.1.5.70783",
        "build_key": "bc493ecb2898766c0f8947a99b4b3d65",
        "simulator_pin": "12.1.5.70575",
        "note": "Installed 70783 data is not a 70575 export.",
    },
    "forever": {
        "product": "wowf",
        "label": "Forever",
        "build": "1.60.1.70888",
        "build_key": "0597e74b01a3f779d849ffe9b3270196",
        "simulator_pin": "1.60.1.70707",
        "note": "70707 values are UNKNOWN; these rows are from installed build 70888.",
    },
}

# Column indices are CSV field positions in the DBD field order. These layout
# hashes and definition hashes pin the index meanings for the exported DB2 files.
# ChrClasses.dbd: WoWDBDefs d6f50d4f351193e84226aa62a36faa0f58528afa,
# SHA-256 3a3d7fe21beef5e07033dde5adb0965de6fd5adb065816ac56ee96fb7b10e7df.
# ChrRaces.dbd: WoWDBDefs d6f50d4f351193e84226aa62a36faa0f58528afa,
# SHA-256 53fd308737d92e7ea2cbffd77add22e82c5a85e0c725545249357e52064b61f6.
TABLES = {
    "ChrClasses": {
        "filedataid": 1361031,
        "layout": "AFC9B0C2",
        "dbd_sha256": "3a3d7fe21beef5e07033dde5adb0965de6fd5adb065816ac56ee96fb7b10e7df",
        "columns": (("ID", 29), ("SelectScreenFileDataID", 11)),
    },
    "ChrRaces": {
        "filedataid": 1305311,
        "layout": "4F44C796",
        "dbd_sha256": "53fd308737d92e7ea2cbffd77add22e82c5a85e0c725545249357e52064b61f6",
        "columns": (
            ("ID", 0),
            ("SelectScreenFileDataID", 22),
            ("LowResScreenFileDataID", 23),
        ),
    },
}


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def parse_export_summary(result, table):
    if result.returncode:
        raise RuntimeError(f"{table}: exporter failed: {result.stderr.strip()}")

    summaries = [
        line.partition(": ")[2]
        for line in result.stderr.splitlines()
        if line.startswith(f"{table}: ")
    ]
    if len(summaries) != 1:
        raise ValueError(f"{table}: exporter did not return one result")
    return ast.literal_eval(summaries[0])


def read_rows(csv_path, table):
    columns = TABLES[table]["columns"]
    with csv_path.open(newline="", encoding="utf-8") as source:
        reader = csv.DictReader(source)
        header = reader.fieldnames or []
        for field, index in columns:
            if index >= len(header) or header[index] != field:
                raise ValueError(f"{table}: CSV column {index} is not {field}")
        rows = [
            (int(row["ID"]), *(int(row[field]) for field, _ in columns[1:]))
            for row in reader
        ]

    if not rows:
        raise ValueError(f"{table}: exporter returned no rows")
    if len({row[0] for row in rows}) != len(rows):
        raise ValueError(f"{table}: duplicate row ID")
    return sorted(rows)


def export_table(args, table, temporary):
    metadata = TABLES[table]
    db2_path = args.db2_dir / f"{table}.db2"
    definition = args.definitions_dir / f"{table}.dbd"
    if not db2_path.is_file() or not definition.is_file():
        missing = db2_path if not db2_path.is_file() else definition
        raise FileNotFoundError(missing)

    data = db2_path.read_bytes()
    if data[:4] != b"WDC5":
        raise ValueError(f"{db2_path}: expected WDC5")
    layout = f"{struct.unpack_from('<I', data, 156)[0]:08X}"
    if layout != metadata["layout"]:
        raise ValueError(f"{table}: layout {layout}, expected {metadata['layout']}")
    if sha256(definition.read_bytes()) != metadata["dbd_sha256"]:
        raise ValueError(f"{table}: DBD does not match the pinned WoWDBDefs revision")

    csv_path = temporary / f"{table}.csv"
    result = subprocess.run(
        [
            sys.executable,
            str(args.exporter),
            table,
            str(db2_path),
            str(csv_path),
            "--definitions",
            str(args.definitions_dir),
        ],
        capture_output=True,
        check=False,
        text=True,
    )
    summary = parse_export_summary(result, table)
    if summary["layout"] != metadata["layout"] or summary["unreadable_records"]:
        raise ValueError(f"{table}: unexpected export result {summary}")
    rows = read_rows(csv_path, table)
    if len(rows) != summary["rows"]:
        raise ValueError(f"{table}: exporter row count differs from CSV")

    return {
        **metadata,
        "db2_sha256": sha256(data),
        "rows": rows,
        "unreadable_records": summary["unreadable_records"],
    }


def write_csv(path, profile, build, table, rows):
    metadata = PROFILES[profile]
    fields = [field for field, _ in TABLES[table]["columns"]]
    with path.open("w", newline="", encoding="utf-8") as output:
        output.write(
            f"# product={metadata['product']}; build={build}; "
            f"simulator_pin={metadata['simulator_pin']}; {metadata['note']}\n"
        )
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(fields)
        writer.writerows(rows)


def write_tables(path, profile, build, records):
    metadata = PROFILES[profile]
    classes = records["ChrClasses"]["rows"]
    races = records["ChrRaces"]["rows"]
    class_columns = TABLES["ChrClasses"]["columns"]
    race_columns = TABLES["ChrRaces"]["columns"]
    lines = [
        "// Generated by tools/gen_chr_select_screen_data.py; do not edit.",
        f"// Product={metadata['product']}; build={build}; "
        f"simulator_pin={metadata['simulator_pin']}; {metadata['note']}",
        f"// ChrClasses layout={records['ChrClasses']['layout']}; "
        f"column indices={class_columns!r}; DBD sha256={records['ChrClasses']['dbd_sha256']}.",
        "pub(super) const CLASS_ROWS: &[(i32, i64)] = &[",
    ]
    lines.extend(f"    ({class_id}, {fdid})," for class_id, fdid in classes)
    lines.extend(
        [
            "];",
            f"// ChrRaces layout={records['ChrRaces']['layout']}; "
            f"column indices={race_columns!r}; DBD sha256={records['ChrRaces']['dbd_sha256']}.",
            "pub(super) const RACE_ROWS: &[(i32, i64, i64)] = &[",
        ]
    )
    lines.extend(
        f"    ({race_id}, {select_id}, {low_res_id}),"
        for race_id, select_id, low_res_id in races
    )
    lines.append("];")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_provenance(path, profile, args, records):
    metadata = PROFILES[profile]
    lines = [
        f"# Character select data — {metadata['label']} build {args.build}",
        "",
        f"Source product: `{metadata['product']}`; installed build: `{args.build}`; "
        f"build key: `{args.build_key}`.",
        f"Simulator pin: `{metadata['simulator_pin']}`. {metadata['note']}",
        f"WoWDBDefs revision: `{args.definitions_revision}`.",
        "All DB2 files were extracted from the local CASC install. No CDN data was used.",
        "",
        "| Table | FileDataID | Layout | DB2 SHA-256 | DBD SHA-256 | Rows | Unreadable |",
        "|---|---:|---|---|---|---:|---:|",
    ]
    for table, record in records.items():
        lines.append(
            f"| `{table}` | {record['filedataid']} | `{record['layout']}` "
            f"| `{record['db2_sha256']}` | `{record['dbd_sha256']}` "
            f"| {len(record['rows'])} | {record['unreadable_records']} |"
        )
    lines.extend(
        [
            "",
            "The CSV files retain explicit zero values. Missing row IDs are not synthesized.",
            "Regenerate with `tools/gen_chr_select_screen_data.py` and the same build/build key, "
            "local DB2 files, definitions revision, and game-engine `export_db2_csv.py` exporter.",
            "",
        ]
    )
    path.write_text("\n".join(lines), encoding="utf-8")


def generate(args):
    metadata = PROFILES[args.profile]
    if args.build != metadata["build"] or args.build_key != metadata["build_key"]:
        raise ValueError(
            f"{args.profile}: expected installed build {metadata['build']} "
            f"({metadata['build_key']})"
        )
    if args.definitions_revision != DEFINITIONS_REVISION:
        raise ValueError(f"expected WoWDBDefs revision {DEFINITIONS_REVISION}")

    output_dir = args.output_dir or (
        Path(__file__).resolve().parents[1]
        / "data/chr-select-screen"
        / f"{args.profile}-{args.build}"
    )
    output_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="chr-select-screen-") as temp_dir:
        records = {
            table: export_table(args, table, Path(temp_dir))
            for table in TABLES
        }

    write_csv(
        output_dir / "ChrClasses.csv",
        args.profile,
        args.build,
        "ChrClasses",
        records["ChrClasses"]["rows"],
    )
    write_csv(
        output_dir / "ChrRaces.csv",
        args.profile,
        args.build,
        "ChrRaces",
        records["ChrRaces"]["rows"],
    )
    write_tables(output_dir / "tables.rs", args.profile, args.build, records)
    write_provenance(output_dir / "PROVENANCE.md", args.profile, args, records)
    print(f"generated {output_dir}")


def main():
    repository = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=tuple(PROFILES), required=True)
    parser.add_argument("--build", required=True)
    parser.add_argument("--build-key", required=True)
    parser.add_argument("--db2-dir", type=Path, required=True)
    parser.add_argument(
        "--definitions-dir",
        type=Path,
        default=repository / "data/chr-select-screen/definitions",
    )
    parser.add_argument("--definitions-revision", required=True)
    parser.add_argument(
        "--exporter", type=Path, required=True, help="game-engine export_db2_csv.py"
    )
    parser.add_argument("--output-dir", type=Path)
    args = parser.parse_args()
    if not args.definitions_dir.is_dir():
        raise FileNotFoundError(args.definitions_dir)
    if not args.exporter.is_file():
        raise FileNotFoundError(args.exporter)
    generate(args)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError) as error:
        raise SystemExit(str(error)) from error
