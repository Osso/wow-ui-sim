#!/usr/bin/env python3
"""Generate the WM78 pet-item catalog from pinned Wago CSV exports."""

import argparse
import csv
import hashlib
import json
from collections import defaultdict
from pathlib import Path

TABLES = (
    "ItemSparse", "ItemXItemEffect", "ItemEffect", "SpellName", "SpellEffect",
    "SummonProperties", "BattlePetSpecies", "Creature", "CreatureXDisplayInfo",
)


def rows(directory, name):
    with (directory / f"{name}.csv").open(newline="", encoding="utf-8") as source:
        yield from csv.DictReader(source)


def generate(directory):
    effects = {int(row["ID"]): row for row in rows(directory, "ItemEffect")}
    associations = defaultdict(list)
    for row in rows(directory, "ItemXItemEffect"):
        effect = effects.get(int(row["ItemEffectID"]))
        if effect is not None:
            associations[int(row["ItemID"])].append(effect)
    sparse = {int(row["ID"]): int(row["Flags_0"]) for row in rows(directory, "ItemSparse")}
    spells = {int(row["ID"]) for row in rows(directory, "SpellName")}
    selected = {}
    for item, entries in associations.items():
        slots = [int(entry["LegacySlotIndex"]) for entry in entries]
        if item not in sparse or sparse[item] & 256 or any(slot < 0 or slot > 4 for slot in slots) or len(set(slots)) != len(slots):
            continue
        for entry in sorted(entries, key=lambda entry: int(entry["LegacySlotIndex"])):
            spell = int(entry["SpellID"])
            if int(entry["TriggerType"]) == 6 and spell:
                if spell in spells:
                    selected[item] = spell
                break
    relevant = set(selected.values())
    spell_effects = defaultdict(list)
    for row in rows(directory, "SpellEffect"):
        spell = int(row["SpellID"])
        if spell in relevant and int(row["EffectIndex"]) == 0:
            spell_effects[spell].append(row)
    competing = sorted(spell for spell, entries in spell_effects.items() if len(entries) > 1)
    if competing:
        raise ValueError(f"unknown spell difficulty selection for {competing}")
    summon = {int(row["ID"]): row for row in rows(directory, "SummonProperties")}
    species = {}
    for row in rows(directory, "BattlePetSpecies"):
        creature = int(row["CreatureID"])
        if creature in species:
            raise ValueError(f"duplicate species creature ID disables index: {creature}")
        species[creature] = row
    creatures = {int(row["ID"]): row for row in rows(directory, "Creature")}
    displays = defaultdict(list)
    for row in rows(directory, "CreatureXDisplayInfo"):
        display = int(row["CreatureDisplayInfoID"])
        if display:
            displays[int(row["CreatureID"])].append((int(row["OrderIndex"]), display))
    result = []
    unresolved = []
    for item, spell in sorted(selected.items()):
        entries = spell_effects.get(spell, [])
        if not entries:
            continue
        effect = entries[0]
        if int(effect["Effect"]) not in (28, 51):
            continue
        properties = summon.get(int(effect["EffectMiscValue_1"]))
        if not properties or int(properties["Slot"]) != 5 or not int(properties["Flags_0"]) & 2097152:
            continue
        creature_id = int(effect["EffectMiscValue_0"])
        pet = species.get(creature_id)
        if pet is None:
            continue
        creature = creatures.get(creature_id)
        candidates = displays[creature_id] if creature else []
        first_order = min((order for order, _ in candidates), default=None)
        choices = sorted({display for order, display in candidates if order == first_order})
        if len(choices) > 1:
            unresolved.append(item)
        flags = int(pet["Flags"])
        result.append({
            "item_id": item,
            "name": creature["Name_lang"] if creature else None,
            "icon": int(pet["IconFileDataID"]) or None,
            "pet_type": int(pet["PetTypeEnum"]) + 1,
            "creature_id": creature_id,
            "source": pet["SourceText_lang"],
            "description": pet["Description_lang"],
            "wild": bool(creature and int(creature["CreatureType"]) == 14),
            "can_battle": not bool(flags & 128),
            "tradeable": not bool(flags & 16),
            "unique": bool(flags & 64),
            "obtainable": not bool(flags & 32),
            "display_choices": choices,
            "species_id": int(pet["ID"]),
        })
    return result, unresolved


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--build", required=True, help="client build label of the CSV export")
    parser.add_argument("csv_dir", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("provenance", type=Path)
    args = parser.parse_args()
    result, unresolved = generate(args.csv_dir)
    output = (json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n").encode()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(output)
    provenance = {
        "PROVENANCE": f"Wago DB2 CSV, build {args.build}, enUS; WM78/WM78b/WM78c selection; SpellName presence EMULATED",
        "generator": f"python3 tools/generate_pet_item_catalog.py --build {args.build} <csv_dir> <output> <provenance>",
        "inputs": {f"{table}.csv": hashlib.sha256((args.csv_dir / f"{table}.csv").read_bytes()).hexdigest() for table in TABLES},
        "output_sha256": hashlib.sha256(output).hexdigest(),
        "records": len(result),
        "unresolved_equal_order_displays": unresolved,
        "notes": ["EMULATED: SpellName IDs approximate spell presence; native presence equivalence is unverified.", "Equal lowest OrderIndex with distinct displays has no known winner."],
    }
    args.provenance.write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
