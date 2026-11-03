#!/usr/bin/env python3
"""Generate GarrAutoSpell data from the pinned 70391 enUS CSV."""

import argparse
import csv
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path.home() / "Projects/wow/data/GarrAutoSpell.csv")
    parser.add_argument(
        "--output", type=Path,
        default=Path(__file__).resolve().parents[1] / "crates/sim-data/data/garrison_auto_spell.rs",
    )
    parser.add_argument("--effects", type=Path, default=Path.home() / "Projects/wow/data/GarrAutoSpellEffect.csv")
    args = parser.parse_args()
    required = ("ID", "Name_lang", "Description_lang", "Cooldown", "Duration", "Flags", "SchoolMask", "IconFileDataID")
    with args.input.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        if not reader.fieldnames or not set(required).issubset(reader.fieldnames):
            raise ValueError(f"GarrAutoSpell CSV missing required columns: {required}")
        rows = sorted(reader, key=lambda row: int(row["ID"]))
    ids = [int(row["ID"]) for row in rows]
    if not rows or len(ids) != len(set(ids)):
        raise ValueError("GarrAutoSpell CSV is empty or has duplicate IDs")
    with args.effects.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source)
        required_effects = {"SpellID", "Effect", "Points", "TargetType"}
        if not reader.fieldnames or not required_effects.issubset(reader.fieldnames):
            raise ValueError(f"GarrAutoSpellEffect CSV missing required columns: {required_effects}")
        effects = list(reader)
    if not effects:
        raise ValueError("GarrAutoSpellEffect CSV is empty")
    derived = {}
    for row in effects:
        spell_id, effect, target = int(row["SpellID"]), int(row["Effect"]), int(row["TargetType"])
        points = float(row["Points"])
        mask, tutorial, thorns = derived.get(spell_id, (0, 0, False))
        if effect in (2, 4, 6, 8):
            mask |= 2
        if effect in (10, 15, 16, 17, 18) or (effect in (11, 12, 19) and points > 0) or (effect in (13, 14, 20) and not points >= 0):
            mask |= 4
        tutorial = max(tutorial, {3: 1, 5: 1, 21: 1, 13: 2, 15: 3, 17: 3, 7: 4}.get(target, 0))
        derived[spell_id] = mask, tutorial, thorns or effect in (15, 16)
    lines = [
        "//! Generated from GarrAutoSpell and GarrAutoSpellEffect build 12.1.5.70391; run tools/gen_garrison_auto_spell.py.",
        "", "#[rustfmt::skip]",
        "pub static GARR_AUTO_SPELLS: &[(u32, &str, u32, u32, u32, u32, u32, u32, bool)] = &[", 
    ]
    for row in rows:
        name = json.dumps(row["Name_lang"], ensure_ascii=False)
        numbers = [int(row[key]) for key in ("Cooldown", "Duration", "SchoolMask", "IconFileDataID")]
        mask, tutorial, thorns = derived.get(int(row["ID"]), (0, 0, False))
        lines.append(f"    ({int(row['ID'])}, {name}, {', '.join(map(str, numbers))}, {mask}, {tutorial}, {str(thorns).lower()}),")
    lines.extend(["];", ""])
    args.output.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {len(rows)} GarrAutoSpell rows in {args.output}")


if __name__ == "__main__":
    main()
