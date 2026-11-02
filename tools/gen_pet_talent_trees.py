#!/usr/bin/env python3
"""Generate pet talent tree selection data from CreatureFamily and ChrSpecialization CSV exports."""

import csv
import json
import sys
from pathlib import Path

family_source, specialization_source = (Path(arg) for arg in sys.argv[1:3])
output = Path(__file__).resolve().parents[1] / "crates/sim-data/data/pet_talent_trees.rs"


def read_rows(path):
    with path.open(newline="", encoding="utf-8") as file:
        return list(csv.DictReader(file))


def rust_string(text):
    if any(ord(char) < 0x20 for char in text):
        raise ValueError(f"control character in {text!r}")
    return json.dumps(text, ensure_ascii=False)


families = sorted((int(row["ID"]), int(row["PetTalentType"])) for row in read_rows(family_source))
# Keyed range 0 is ChrSpecialization ClassID 0; rows keep ID order.
specializations = sorted(
    (
        int(row["ID"]),
        int(row["PetTalentType"]),
        int(row["OrderIndex"]),
        int(row["Flags"]),
        row["Name_lang"],
        row["FemaleName_lang"],
    )
    for row in read_rows(specialization_source)
    if int(row["ClassID"]) == 0
)
if len({family for family, _ in families}) != len(families):
    raise ValueError("duplicate CreatureFamily ID")
if any(not -128 <= talent_type <= 127 for _, talent_type in families):
    raise ValueError("PetTalentType outside i8 range")

with output.open("w", encoding="utf-8") as file:
    file.write("//! Generated from CreatureFamily.csv and ChrSpecialization.csv, build 12.1.5.70391.\n")
    file.write(
        "//! Regenerate with `python3 tools/gen_pet_talent_trees.py <CreatureFamily.csv> "
        "<ChrSpecialization.csv>`.\n\n"
    )
    file.write("/// A ChrSpecialization row in keyed range 0 (ClassID 0).\n")
    file.write("pub struct PetSpecialization {\n")
    file.write("    pub id: u32,\n")
    file.write("    pub pet_talent_type: i8,\n")
    file.write("    pub order_index: i32,\n")
    file.write("    pub flags: u32,\n")
    file.write("    pub name: &'static str,\n")
    file.write("    pub female_name: &'static str,\n")
    file.write("}\n\n")
    file.write("/// CreatureFamily ID and its signed PetTalentType, in ID order.\n")
    file.write("pub static FAMILY_TALENT_TYPES: &[(u32, i8)] = &[\n")
    for family, talent_type in families:
        file.write(f"    ({family}, {talent_type}),\n")
    file.write("];\n\n")
    file.write("/// ChrSpecialization rows with ClassID 0, in ID order.\n")
    file.write("pub static PET_SPECIALIZATIONS: &[PetSpecialization] = &[\n")
    for spec_id, talent_type, order_index, flags, name, female_name in specializations:
        file.write("    PetSpecialization {\n")
        file.write(f"        id: {spec_id},\n")
        file.write(f"        pet_talent_type: {talent_type},\n")
        file.write(f"        order_index: {order_index},\n")
        file.write(f"        flags: {flags},\n")
        file.write(f"        name: {rust_string(name)},\n")
        file.write(f"        female_name: {rust_string(female_name)},\n")
        file.write("    },\n")
    file.write("];\n")
print(f"Generated {len(families)} families and {len(specializations)} pet specializations in {output}")
