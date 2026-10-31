#!/usr/bin/env python3
"""Generate Forever's PHF string table from its committed build-specific CSV."""

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "data/db2/wowforever-1.60.1.70532/GlobalStrings.csv"
OUTPUT = ROOT / "crates/sim-data-profile/data/global_strings_wowforever.rs"


def read_strings(source):
    strings = {}
    with source.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames != ["ID", "BaseTag", "TagText_lang", "Flags"]:
            raise ValueError(f"{source}: unexpected GlobalStrings columns")
        for row in reader:
            tag = row["BaseTag"]
            if not tag or tag in strings:
                raise ValueError(f"{source}: empty or duplicate tag {tag!r}")
            strings[tag] = row["TagText_lang"]
    return strings


def rust_string(value):
    # JSON escapes match Rust for this UTF-8 dataset except control \u escapes.
    return json.dumps(value, ensure_ascii=False).replace("\\u00", "\\x")


def generate(source, output, profile="Forever 1.60.1.70532"):
    strings = read_strings(source)
    lines = [
        f"//! Generated from {profile} GlobalStrings.csv.",
        "//! Regenerate: python3 tools/gen_forever_global_strings.py" if profile == "Forever 1.60.1.70532" else "//! Regenerate: python3 tools/gen_forever_global_strings.py --profile ptr",
        "use phf::phf_map;",
        "pub fn get_global_string(name: &str) -> Option<&'static str> {",
        "    GLOBAL_STRINGS.get(name).copied()",
        "}",
        "pub static GLOBAL_STRINGS: phf::Map<&'static str, &'static str> = phf_map! {",
    ]
    lines.extend(f"    {rust_string(tag)} => {rust_string(value)}," for tag, value in sorted(strings.items()))
    lines.append("};")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Generated {len(strings)} strings: {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--profile", choices=["forever", "ptr"], default="forever")
    args = parser.parse_args()
    if args.profile == "ptr":
        source = ROOT / "data/db2/ptr-12.1.7.70493/GlobalStrings.csv"
        output = ROOT / "crates/sim-data-profile/data/global_strings_ptr.rs"
        generate(source, output, "PTR 12.1.7.70493")
    else:
        generate(args.source, args.output)
