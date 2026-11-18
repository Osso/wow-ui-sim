#!/usr/bin/env python3
"""Generate Retail 12.1.5.70688 secure-publication data from the R88y CSV."""

import argparse
import csv
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "data/secure-publication/retail-12-1-5-70688.csv"
OUTPUT = ROOT / "src/c_api/secure_publication_data.rs"
PROVENANCE = ROOT / "data/secure-publication/provenance.json"
SOURCE_SHA256 = "3a8922d84756171f666fe79b61068d3344ce2db866250dede1de384fa9878f82"
EXPECTED_ROWS = 6_763
EXPECTED_EVIDENCE_COUNTS = {"VERIFIED": 6_554, "INFERRED": 209}
EXPECTED_FLAG_COUNTS = {"1": 122, "2": 42, "3": 6_573, "6": 1, "7": 25}
EXPECTED_CONTEXT_COUNTS = {"Game": 5_136, "Glue": 298, "both": 1_329}
VERIFIED_STRING_LIBRARY_MEMBERS = ("concat", "join", "split")
RUSTFMT_INLINE_TUPLE_LIMIT = 67  # Match this repo's Rustfmt array-tuple layout.


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def read_rows(source):
    raw = source.read_bytes()
    actual_sha256 = sha256(raw)
    if actual_sha256 != SOURCE_SHA256:
        raise ValueError(f"{source}: SHA-256 {actual_sha256}, expected {SOURCE_SHA256}")

    text = raw.decode("utf-8")
    reader = csv.DictReader(text.splitlines())
    expected_columns = ["namespace", "member", "flag", "context", "evidence"]
    if reader.fieldnames != expected_columns:
        raise ValueError(f"{source}: unexpected columns {reader.fieldnames!r}")

    rows = list(reader)
    if len(rows) != EXPECTED_ROWS:
        raise ValueError(f"{source}: {len(rows)} rows, expected {EXPECTED_ROWS}")

    pairs = [(row["namespace"], row["member"]) for row in rows]
    if len(set(pairs)) != len(pairs):
        raise ValueError(f"{source}: duplicate namespace/member pair")

    evidence_counts = Counter(row["evidence"] for row in rows)
    if dict(evidence_counts) != EXPECTED_EVIDENCE_COUNTS:
        raise ValueError(f"{source}: unexpected evidence counts {dict(evidence_counts)}")

    flag_counts = Counter(row["flag"] for row in rows)
    if dict(flag_counts) != EXPECTED_FLAG_COUNTS:
        raise ValueError(f"{source}: unexpected flag counts {dict(flag_counts)}")

    context_counts = Counter(row["context"] for row in rows)
    if dict(context_counts) != EXPECTED_CONTEXT_COUNTS:
        raise ValueError(f"{source}: unexpected context counts {dict(context_counts)}")

    if any(not row["namespace"] or not row["member"] for row in rows):
        raise ValueError(f"{source}: empty namespace or member")
    if any(row["flag"] not in EXPECTED_FLAG_COUNTS for row in rows):
        raise ValueError(f"{source}: unsupported publication flag")
    if any(row["context"] not in EXPECTED_CONTEXT_COUNTS for row in rows):
        raise ValueError(f"{source}: unsupported context")
    if any(row["evidence"] not in EXPECTED_EVIDENCE_COUNTS for row in rows):
        raise ValueError(f"{source}: unsupported evidence label")
    if re.search(r"0x[0-9a-fA-F]{6,}", text):
        raise ValueError(f"{source}: address pattern found")

    return rows


def rust_string(value):
    return json.dumps(value, ensure_ascii=False)


def render_rust(rows):
    verified = sorted(
        (row for row in rows if row["evidence"] == "VERIFIED"),
        key=lambda row: (row["namespace"], row["member"]),
    )
    inferred = [row for row in rows if row["evidence"] == "INFERRED"]
    string_copy = sorted(
        row["member"]
        for row in verified
        if row["namespace"] == "string" and row["flag"] == "1"
    )
    if tuple(string_copy) != VERIFIED_STRING_LIBRARY_MEMBERS:
        raise ValueError(f"unexpected verified flag-1 string members: {string_copy}")

    lines = [
        "//! Generated from data/secure-publication/retail-12-1-5-70688.csv.",
        f"//! Source SHA-256: {SOURCE_SHA256}.",
        f"//! The CSV has {len(rows)} rows: {len(verified)} VERIFIED and {len(inferred)} INFERRED. Evidence: pr46-49-decomp-answers.md, section R88y.y1.",
        "//! Runtime flag routing uses VERIFIED rows only; the source CSV keeps both labels. Evidence: pr46-49-decomp-answers.md, section R88y.y1.",
        "//! Row context codes are 0=Game, 1=Glue, 2=both.",
        "",
        "pub(crate) const SOURCE_SHA256: &str =",
        f"    \"{SOURCE_SHA256}\";",
        f"pub(crate) const SOURCE_ROW_COUNT: usize = {len(rows)};",
        f"pub(crate) const INFERRED_ROW_COUNT: usize = {len(inferred)};",
        "",
        "pub(crate) const CONTEXT_GAME: u8 = 0;",
        "pub(crate) const CONTEXT_GLUE: u8 = 1;",
        "pub(crate) const CONTEXT_BOTH: u8 = 2;",
        "",
        "pub(crate) const VERIFIED_MEMBERS: &[(&str, &str, u8, u8)] = &[",
    ]
    context_codes = {"Game": "0", "Glue": "1", "both": "2"}
    for row in verified:
        fields = (
            rust_string(row["namespace"]),
            rust_string(row["member"]),
            row["flag"],
            context_codes[row["context"]],
        )
        inline_row = "    (%s, %s, %s, %s)," % fields
        if len(inline_row) <= RUSTFMT_INLINE_TUPLE_LIMIT:
            lines.append(inline_row)
        else:
            lines.extend(
                [
                    "    (",
                    f"        {fields[0]},",
                    f"        {fields[1]},",
                    f"        {fields[2]},",
                    f"        {fields[3]},",
                    "    ),",
                ]
            )
    lines.extend(["];"])
    lines.extend(["", "pub(crate) const VERIFIED_STRING_LIBRARY_COPY_ALIASES: &[(&str, &str)] = &["])
    lines.extend(
        f'    ({rust_string(member)}, {rust_string("str" + member)}),'
        for member in string_copy
    )
    lines.extend(["];", ""])
    return "\n".join(lines)


def build_provenance(rows, rust_output):
    return {
        "source": "R88y-secure-publication-70688.csv",
        "source_sha256": SOURCE_SHA256,
        "build": "Retail 12.1.5.70688",
        "rows": len(rows),
        "evidence_counts": dict(Counter(row["evidence"] for row in rows)),
        "flag_counts": dict(Counter(row["flag"] for row in rows)),
        "context_counts": dict(Counter(row["context"] for row in rows)),
        "runtime_rows": "VERIFIED only; INFERRED rows remain in the source CSV",
        "generator": "tools/gen_r88y_secure_publication.py",
        "generated_table": "src/c_api/secure_publication_data.rs",
        "generated_table_sha256": sha256(rust_output.encode("utf-8")),
    }


def write_or_check(path, content, check):
    if check:
        if not path.is_file() or path.read_text(encoding="utf-8") != content:
            raise ValueError(f"{path}: generated content is stale; rerun this generator")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=SOURCE)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--provenance", type=Path, default=PROVENANCE)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    rows = read_rows(args.input)
    rust_output = render_rust(rows)
    provenance_output = json.dumps(build_provenance(rows, rust_output), indent=2) + "\n"
    write_or_check(args.output, rust_output, args.check)
    write_or_check(args.provenance, provenance_output, args.check)
    print(f"R88y: {len(rows)} source rows; {len(rust_output.splitlines())} generated lines")


if __name__ == "__main__":
    main()
