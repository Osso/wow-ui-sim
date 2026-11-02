#!/usr/bin/env python3
"""Generate profession specialization reminder data from DB2 CSV exports.

Arguments: SkillLine.csv SkillLineXTraitTree.csv TraitTreeXTraitCurrency.csv
TraitCurrency.csv CurrencyTypes.csv.
"""

import csv
import json
import sys
from collections import defaultdict
from pathlib import Path

PROFESSION_CATEGORIES = {"9", "11"}
POINT_CURRENCY_TYPE = "1"
DIVIDE_BY_100_FLAG = 8

skill_source, tab_source, tree_currency_source, currency_source, types_source = (
    Path(arg) for arg in sys.argv[1:6]
)
output = Path(__file__).resolve().parents[1] / "crates/sim-data/data/profession_spec_reminders.rs"


def read_rows(path):
    with path.open(newline="", encoding="utf-8") as file:
        return list(csv.DictReader(file))


def rust_string(text):
    if any(ord(char) < 0x20 for char in text):
        raise ValueError(f"control character in {text!r}")
    return json.dumps(text, ensure_ascii=False)


skill_rows = read_rows(skill_source)
# Profession and secondary-skill lines plus every descendant.
included = {row["ID"] for row in skill_rows if row["CategoryID"] in PROFESSION_CATEGORIES}
while True:
    children = {row["ID"] for row in skill_rows if row["ParentSkillLineID"] in included}
    if children <= included:
        break
    included |= children
# WM55-Q4: children iterate by signed ParentTierIndex descending; equal tiers
# keep CSV row order (INFERRED). The stable sort applies that order globally.
skill_lines = sorted(
    (
        (int(row["ID"]), int(row["ParentSkillLineID"]), int(row["ParentTierIndex"]), row["DisplayName_lang"])
        for row in skill_rows
        if row["ID"] in included
    ),
    key=lambda line: -line[2],
)
tabs = [
    (int(row["SkillLineID"]), int(row["TraitTreeID"]))
    for row in read_rows(tab_source)
    if row["SkillLineID"] in included
]

trait_currencies = {row["ID"]: row for row in read_rows(currency_source)}
currency_flags = {row["ID"]: int(row["Flags_0"]) for row in read_rows(types_source)}
tree_currencies = defaultdict(list)
for row in read_rows(tree_currency_source):
    tree_currencies[int(row["TraitTreeID"])].append((int(row["_Index"]), row["TraitCurrencyID"]))


def first_point_currency(tree):
    """First Type-1 TraitCurrency with a CurrencyTypes row, in _Index order."""
    for _, trait_currency_id in sorted(tree_currencies[tree]):
        row = trait_currencies.get(trait_currency_id)
        if row and row["Type"] == POINT_CURRENCY_TYPE and row["CurrencyTypesID"] in currency_flags:
            currency_id = row["CurrencyTypesID"]
            return int(currency_id), bool(currency_flags[currency_id] & DIVIDE_BY_100_FLAG)
    return None


tab_currencies = sorted(
    {(tree, *currency) for _, tree in tabs if (currency := first_point_currency(tree))}
)

with output.open("w", encoding="utf-8") as file:
    file.write(
        "//! Generated from SkillLine, SkillLineXTraitTree, TraitTreeXTraitCurrency, TraitCurrency\n"
        "//! and CurrencyTypes CSV exports, build 12.1.5.70391.\n"
        "//! Regenerate with `python3 tools/gen_profession_spec_reminders.py <SkillLine.csv>\n"
        "//! <SkillLineXTraitTree.csv> <TraitTreeXTraitCurrency.csv> <TraitCurrency.csv> <CurrencyTypes.csv>`.\n\n"
    )
    file.write("pub struct SkillLine {\n")
    file.write("    pub id: u32,\n    pub parent_id: u32,\n    pub parent_tier_index: i32,\n")
    file.write("    pub name: &'static str,\n}\n\n")
    file.write("/// The first point currency of a specialization tab (TraitTree).\n")
    file.write("pub struct TabCurrency {\n")
    file.write("    pub tab_id: u32,\n    pub currency_id: u32,\n    pub divide_by_100: bool,\n}\n\n")
    file.write(
        "/// Profession and secondary SkillLines with their descendants, by ParentTierIndex\n"
        "/// descending, then CSV row order.\n"
    )
    file.write("pub static SKILL_LINES: &[SkillLine] = &[\n")
    for skill_id, parent_id, tier, name in skill_lines:
        file.write("    SkillLine {\n")
        file.write(f"        id: {skill_id},\n        parent_id: {parent_id},\n")
        file.write(f"        parent_tier_index: {tier},\n")
        file.write(f"        name: {rust_string(name)},\n    }},\n")
    file.write("];\n\n")
    file.write("/// (SkillLineID, TraitTreeID) in SkillLineXTraitTree row order.\n")
    file.write("pub static SKILL_LINE_TABS: &[(u32, u32)] = &[\n")
    for skill_id, tree in tabs:
        file.write(f"    ({skill_id}, {tree}),\n")
    file.write("];\n\n")
    file.write("/// Tabs with a Type-1 TraitCurrency that has a CurrencyTypes row, in tab ID order.\n")
    file.write("pub static TAB_POINT_CURRENCIES: &[TabCurrency] = &[\n")
    for tree, currency_id, divide in tab_currencies:
        file.write("    TabCurrency {\n")
        file.write(f"        tab_id: {tree},\n        currency_id: {currency_id},\n")
        file.write(f"        divide_by_100: {str(divide).lower()},\n    }},\n")
    file.write("];\n")
print(
    f"Generated {len(skill_lines)} skill lines, {len(tabs)} tabs and "
    f"{len(tab_currencies)} tab currencies in {output}"
)
