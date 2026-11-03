//! Generator for items.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - ItemSparse.csv
//!   - ItemModifiedAppearance.csv
//!   - ItemAppearance.csv
//!
//! Generates: crates/sim-data/data/items.rs

use super::csv_util::{escape_str, parse_csv_line, read_csv_records, wow_data_dir};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let csv_path = wow_data.join("ItemSparse.csv");
    println!("Loading ItemSparse from {}...", csv_path.display());

    let mut required_ids = collect_required_item_ids();
    required_ids.extend(collect_keystone_item_ids(&wow_data)?);
    println!("Required item IDs: {} (deduplicated)", required_ids.len());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    let icon_map = build_icon_map(&wow_data, &required_ids)?;

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("crates/sim-data/data/items.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;

    let (count, skipped) = build_item_map(&mut out, reader, &icon_map, &required_ids, &wow_data)?;

    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;
    write_item_categories(&wow_data, &required_ids)?;
    write_gem_catalog(&wow_data)?;
    write_item_effects(&wow_data)?;
    write_item_children(&wow_data)?;
    write_item_enchant_spells(&wow_data)?;
    write_item_spell_metadata(&wow_data)?;
    write_item_uniqueness(&wow_data)?;
    write_item_sets(&wow_data)?;
    write_item_classifiers(&wow_data)?;
    write_item_spell_ranges(&wow_data)?;
    write_item_cursor_grid(&wow_data)?;

    println!("Generated {} item entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());
    Ok(())
}

/// Inputs for IsAnimaItemByID (ItemCurrencyValue rows) and
/// IsArtifactPowerItem (spells whose difficulty-0 effect index 0 is type 240).
fn write_item_classifiers(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const GRANT_ARTIFACT_POWER: &str = "240";
    let records = open_records(&wow_data.join("ItemCurrencyValue.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemCurrencyValue.csv")?);
    let mut values = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        let item = field(&fields, &idx, "ItemID").parse::<u32>()?;
        let value = field(&fields, &idx, "Value").parse::<i32>()?;
        let mut currencies = [0_u32; 4];
        for (slot, currency) in currencies.iter_mut().enumerate() {
            *currency = field(&fields, &idx, &format!("CurrencyID_{slot}")).parse()?;
        }
        values.push((item, id, value, currencies));
    }
    values.sort_unstable();
    let records = open_records(&wow_data.join("SpellEffect.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty SpellEffect.csv")?);
    let mut spells = BTreeSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let first_effect = field(&fields, &idx, "EffectIndex") == "0";
        let base_difficulty = field(&fields, &idx, "DifficultyID") == "0";
        if first_effect && base_difficulty && field(&fields, &idx, "Effect") == GRANT_ARTIFACT_POWER
        {
            spells.insert(field(&fields, &idx, "SpellID").parse::<u32>()?);
        }
    }
    let mut out = File::create("crates/sim-data/data/item_classifiers.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(
        out,
        "pub static ITEM_CURRENCY_VALUES: &[(u32, i32, [u32; 4])] = &["
    )?;
    for (item, _, value, currencies) in values {
        writeln!(out, "    ({item}, {value}, {currencies:?}),")?;
    }
    writeln!(out, "];\n")?;
    writeln!(out, "pub static ARTIFACT_POWER_SPELLS: &[u32] = &[")?;
    for spell in spells {
        writeln!(out, "    {spell},")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

/// Base Item record inputs for the owned-item cursor grid (WM54 Q4):
/// ID ranges with nonzero InventoryType, and class 17 item IDs.
fn write_item_cursor_grid(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const CLASS_BATTLE_PET: &str = "17";
    let mut equipment_slotted = Vec::new();
    let mut battle_pets = Vec::new();
    for_each_csv_row(wow_data, "Item.csv", |row| {
        let id = row.get("ID").parse::<u32>()?;
        if row.get("InventoryType").parse::<u8>()? != 0 {
            equipment_slotted.push(id);
        }
        if row.get("ClassID") == CLASS_BATTLE_PET {
            battle_pets.push(id);
        }
        Ok(())
    })?;
    equipment_slotted.sort_unstable();
    battle_pets.sort_unstable();
    let mut out = File::create("crates/sim-data/data/item_cursor_grid.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    let ranges = collapse_id_ranges(&equipment_slotted);
    write_rows(
        &mut out,
        "INVENTORY_TYPED_ITEM_RANGES: &[(u32, u32)]",
        &ranges,
    )?;
    write_rows(&mut out, "CLASS_17_ITEMS: &[u32]", &battle_pets)?;
    Ok(())
}

/// Inclusive runs of consecutive IDs from a sorted list.
fn collapse_id_ranges(ids: &[u32]) -> Vec<(u32, u32)> {
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for &id in ids {
        match ranges.last_mut() {
            Some((_, end)) if end.checked_add(1) == Some(id) => *end = id,
            _ => ranges.push((id, id)),
        }
    }
    ranges
}

fn write_item_sets(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("ItemSparse.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemSparse.csv")?);
    let mut members = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let set = field(&fields, &idx, "ItemSet").parse::<u32>()?;
        if set != 0 {
            members.push((field(&fields, &idx, "ID").parse::<u32>()?, set));
        }
    }
    members.sort_unstable();
    let records = open_records(&wow_data.join("ItemSet.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemSet.csv")?);
    let mut names = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        names.push((id, escape_str(field(&fields, &idx, "Name_lang"))));
    }
    names.sort_unstable();
    let mut out = File::create("crates/sim-data/data/item_sets.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(out, "pub static ITEM_SET_MEMBERS: &[(u32, u32)] = &[")?;
    for (item, set) in members {
        writeln!(out, "    ({item}, {set}),")?;
    }
    writeln!(out, "];\n")?;
    writeln!(out, "pub static ITEM_SET_NAMES: &[(u32, &str)] = &[")?;
    for (id, name) in names {
        writeln!(out, "    ({id}, \"{name}\"),")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn write_item_uniqueness(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const BYPASS_LIMIT_CATEGORY: i32 = 524_288;
    let records = open_records(&wow_data.join("ItemSparse.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemSparse.csv")?);
    let mut candidates = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        let category = field(&fields, &idx, "LimitCategory").parse::<u32>()?;
        let flags = field(&fields, &idx, "Flags_0").parse::<i32>()?;
        let bypass = flags & BYPASS_LIMIT_CATEGORY != 0;
        if category != 0 || bypass {
            candidates.push((id, category, bypass));
        }
    }
    candidates.sort_unstable();
    let records = open_records(&wow_data.join("ItemLimitCategory.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemLimitCategory.csv")?);
    let mut categories = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        categories.push(field(&fields, &idx, "ID").parse::<u32>()?);
    }
    categories.sort_unstable();
    let mut out = File::create("crates/sim-data/data/item_uniqueness.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(out, "pub static ITEM_UNIQUENESS: &[(u32, u32, bool)] = &[")?;
    for (item, category, bypass) in candidates {
        writeln!(out, "    ({item}, {category}, {bypass}),")?;
    }
    writeln!(out, "];\n")?;
    writeln!(out, "pub static ITEM_LIMIT_CATEGORIES: &[u32] = &[")?;
    for category in categories {
        writeln!(out, "    {category},")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn write_item_spell_metadata(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = File::create("crates/sim-data/data/item_spell_metadata.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    write_item_spell_power(&mut out, wow_data)?;
    Ok(())
}

fn write_item_spell_power(
    out: &mut File,
    wow_data: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("SpellPower.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty SpellPower.csv")?);
    let mut rows = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let spell = field(&fields, &idx, "SpellID").parse::<u32>()?;
        let order = field(&fields, &idx, "OrderIndex").parse::<u32>()?;
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        let power = field(&fields, &idx, "PowerType").parse::<i32>()?;
        let cost = field(&fields, &idx, "ManaCost").parse::<i32>()?;
        let per_level = field(&fields, &idx, "ManaCostPerLevel").parse::<i32>()?;
        let percent = field(&fields, &idx, "PowerCostPct").parse::<f32>()?;
        let maximum_percent = field(&fields, &idx, "PowerCostMaxPct").parse::<f32>()?;
        let bar = field(&fields, &idx, "AltPowerBarID").parse::<u32>()?;
        let aura = field(&fields, &idx, "RequiredAuraSpellID").parse::<u32>()?;
        rows.push((
            spell,
            order,
            id,
            power,
            cost,
            per_level,
            percent,
            maximum_percent,
            bar,
            aura,
        ));
    }
    rows.sort_unstable_by_key(|row| (row.0, row.1, row.2));
    writeln!(
        out,
        "pub static SPELL_POWER: &[(u32, u32, u32, i32, i32, i32, f32, f32, u32, u32)] = &["
    )?;
    for (spell, order, id, power, cost, per_level, percent, maximum_percent, bar, aura) in rows {
        writeln!(
            out,
            "    ({spell}, {order}, {id}, {power}, {cost}, {per_level}, {percent:?}, {maximum_percent:?}, {bar}, {aura}),"
        )?;
    }
    writeln!(out, "];\n")?;
    Ok(())
}

/// Inputs for ItemHasRange (WM43): range fields, SpellRange arrays, target
/// restrictions in CSV order, Difficulty fallbacks and difficulty-0 effect targets.
fn write_item_spell_ranges(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let spells = collect_item_referenced_spells(wow_data)?;
    let mut misc = Vec::new();
    for_each_csv_row(wow_data, "SpellMisc.csv", |row| {
        let spell = row.get("SpellID").parse::<u32>()?;
        if spells.contains(&spell) && row.get("DifficultyID") == "0" {
            let attributes = row.get("Attributes_0").parse::<i32>()?;
            misc.push((spell, attributes, row.get("RangeIndex").parse::<u32>()?));
        }
        Ok(())
    })?;
    misc.sort_unstable();
    let mut ranges = Vec::new();
    for_each_csv_row(wow_data, "SpellRange.csv", |row| {
        let min: [f32; 2] = [
            row.get("RangeMin_0").parse()?,
            row.get("RangeMin_1").parse()?,
        ];
        let max: [f32; 2] = [
            row.get("RangeMax_0").parse()?,
            row.get("RangeMax_1").parse()?,
        ];
        ranges.push((row.get("ID").parse::<u32>()?, min, max));
        Ok(())
    })?;
    ranges.sort_unstable_by_key(|row| row.0);
    let mut restrictions = Vec::new();
    for_each_csv_row(wow_data, "SpellTargetRestrictions.csv", |row| {
        let spell = row.get("SpellID").parse::<u32>()?;
        if spells.contains(&spell) {
            let difficulty = row.get("DifficultyID").parse::<i32>()?;
            restrictions.push((spell, difficulty, row.get("Targets").parse::<u32>()?));
        }
        Ok(())
    })?;
    // Stable sort: within a spell, the first CSV row for a difficulty wins.
    restrictions.sort_by_key(|row| row.0);
    let mut fallbacks = Vec::new();
    for_each_csv_row(wow_data, "Difficulty.csv", |row| {
        let fallback = row.get("FallbackDifficultyID").parse::<i32>()?;
        fallbacks.push((row.get("ID").parse::<i32>()?, fallback));
        Ok(())
    })?;
    fallbacks.sort_unstable();
    reject_fallback_cycles(&fallbacks)?;
    let effects = collect_effect_targets(wow_data, &spells)?;
    let mut out = File::create("crates/sim-data/data/item_spell_ranges.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    write_rows(&mut out, "SPELL_MISC_RANGES: &[(u32, i32, u32)]", &misc)?;
    write_rows(
        &mut out,
        "SPELL_RANGES: &[(u32, [f32; 2], [f32; 2])]",
        &ranges,
    )?;
    write_rows(
        &mut out,
        "SPELL_TARGET_RESTRICTIONS: &[(u32, i32, u32)]",
        &restrictions,
    )?;
    write_rows(&mut out, "DIFFICULTY_FALLBACKS: &[(i32, i32)]", &fallbacks)?;
    write_rows(
        &mut out,
        "SPELL_EFFECT_TARGETS: &[(u32, i32, u32, u32)]",
        &effects,
    )?;
    Ok(())
}

/// Spells reachable from ItemEffect rows or type-7 enchantment effects.
fn collect_item_referenced_spells(
    wow_data: &Path,
) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
    let mut spells = HashSet::new();
    for_each_csv_row(wow_data, "ItemEffect.csv", |row| {
        spells.insert(row.get("SpellID").parse::<u32>()?);
        Ok(())
    })?;
    for_each_csv_row(wow_data, "SpellItemEnchantment.csv", |row| {
        if let Some(index) = (0..3).find(|index| row.get(&format!("Effect_{index}")) == "7") {
            spells.insert(row.get(&format!("EffectArg_{index}")).parse::<u32>()?);
        }
        Ok(())
    })?;
    Ok(spells)
}

/// Difficulty-0 effect targets, first row per effect index. The node route is
/// not modelled, so a referenced spell with a nonzero Node fails generation.
fn collect_effect_targets(
    wow_data: &Path,
    spells: &HashSet<u32>,
) -> Result<Vec<(u32, i32, u32, u32)>, Box<dyn std::error::Error>> {
    let mut slots = BTreeMap::new();
    for_each_csv_row(wow_data, "SpellEffect.csv", |row| {
        let spell = row.get("SpellID").parse::<u32>()?;
        if !spells.contains(&spell) || row.get("DifficultyID") != "0" {
            return Ok(());
        }
        if row.get("Node__Field_12_0_0_63534_001") != "0" {
            return Err(
                format!("spell {spell} has a node effect; ItemHasRange does not model it").into(),
            );
        }
        let index = row.get("EffectIndex").parse::<u32>()?;
        let targets = (
            row.get("EffectAttributes").parse::<i32>()?,
            row.get("ImplicitTarget_0").parse::<u32>()?,
            row.get("ImplicitTarget_1").parse::<u32>()?,
        );
        slots.entry((spell, index)).or_insert(targets);
        Ok(())
    })?;
    Ok(slots
        .into_iter()
        .map(|((spell, _), (attributes, first, second))| (spell, attributes, first, second))
        .collect())
}

fn reject_fallback_cycles(fallbacks: &[(i32, i32)]) -> Result<(), Box<dyn std::error::Error>> {
    let next: HashMap<i32, i32> = fallbacks.iter().copied().collect();
    for &(start, _) in fallbacks {
        let mut current = start;
        for _ in 0..=fallbacks.len() {
            match next.get(&current) {
                Some(&fallback) => current = fallback,
                None => break,
            }
        }
        if next.contains_key(&current) {
            return Err(format!("Difficulty {start} fallback chain does not terminate").into());
        }
    }
    Ok(())
}

fn write_item_effects(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("ItemEffect.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemEffect.csv")?);
    let mut effects = HashMap::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        let spell = field(&fields, &idx, "SpellID").parse::<u32>()?;
        let trigger = field(&fields, &idx, "TriggerType").parse::<i32>()?;
        let slot = field(&fields, &idx, "LegacySlotIndex").parse::<i32>()?;
        let spec = field(&fields, &idx, "ChrSpecializationID").parse::<u32>()?;
        let condition = field(&fields, &idx, "PlayerConditionID").parse::<u32>()?;
        effects.insert(id, (spell, trigger, slot, spec, condition));
    }
    let records = open_records(&wow_data.join("ItemXItemEffect.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemXItemEffect.csv")?);
    let mut rows = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let item = field(&fields, &idx, "ItemID").parse::<u32>()?;
        let effect = field(&fields, &idx, "ItemEffectID").parse::<u32>()?;
        let &(spell, trigger, slot, spec, condition) = effects
            .get(&effect)
            .ok_or_else(|| format!("missing ItemEffect {effect} for item {item}"))?;
        rows.push((item, effect, spell, trigger, slot, spec, condition));
    }
    rows.sort_unstable();
    let spell_ids: HashSet<u32> = rows.iter().map(|row| row.2).collect();
    let mut out = File::create("crates/sim-data/data/item_effects.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(
        out,
        "pub static ITEM_EFFECTS: &[(u32, u32, u32, i32, i32, u32, u32)] = &["
    )?;
    for (item, effect, spell, trigger, slot, spec, condition) in rows {
        writeln!(
            out,
            "    ({item}, {effect}, {spell}, {trigger}, {slot}, {spec}, {condition}),"
        )?;
    }
    writeln!(out, "];\n")?;
    let records = open_records(&wow_data.join("SpellName.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty SpellName.csv")?);
    let mut names = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        if spell_ids.contains(&id) {
            names.push((id, escape_str(field(&fields, &idx, "Name_lang"))));
        }
    }
    names.sort_unstable_by_key(|row| row.0);
    writeln!(out, "pub static ITEM_SPELL_NAMES: &[(u32, &str)] = &[")?;
    for (id, name) in names {
        writeln!(out, "    ({id}, \"{name}\"),")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn write_item_enchant_spells(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("SpellItemEnchantment.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty SpellItemEnchantment.csv")?);
    let mut spells = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        for index in 0..3 {
            if field(&fields, &idx, &format!("Effect_{index}")) == "7" {
                let id = field(&fields, &idx, "ID").parse::<u32>()?;
                let spell = field(&fields, &idx, &format!("EffectArg_{index}")).parse::<u32>()?;
                spells.push((id, spell));
                break;
            }
        }
    }
    let records = open_records(&wow_data.join("SpellName.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty SpellName.csv")?);
    let wanted: HashSet<u32> = spells.iter().map(|row| row.1).collect();
    let mut names = HashMap::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        if wanted.contains(&id) {
            names.insert(id, escape_str(field(&fields, &idx, "Name_lang")));
        }
    }
    spells.sort_unstable();
    let mut out = File::create("crates/sim-data/data/item_enchants.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(
        out,
        "pub static ENCHANT_SPELLS: &[(u32, u32, Option<&str>)] = &["
    )?;
    for (id, spell) in spells {
        let name = names
            .get(&spell)
            .map_or_else(|| "None".to_owned(), |name| format!("Some(\"{name}\")"));
        writeln!(out, "    ({id}, {spell}, {name}),")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn write_item_children(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("ItemChildEquipment.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemChildEquipment.csv")?);
    let mut out = File::create("crates/sim-data/data/item_children.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV export, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(out, "pub static ITEM_CHILDREN: &[(u32, u32, i32)] = &[")?;
    for record in iter {
        let fields = parse_csv_line(record);
        let parent = field(&fields, &idx, "ParentItemID").parse::<u32>()?;
        let child = field(&fields, &idx, "ChildItemID").parse::<u32>()?;
        let slot = field(&fields, &idx, "ChildItemEquipSlot").parse::<i32>()?;
        writeln!(out, "    ({parent}, {child}, {slot}),")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn collect_keystone_item_ids(wow_data: &Path) -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("Item.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty Item.csv")?);
    let mut ids = BTreeSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        if field(&fields, &idx, "ClassID") == "5" && field(&fields, &idx, "SubclassID") == "1" {
            ids.insert(field(&fields, &idx, "ID").parse()?);
        }
    }
    Ok(ids)
}

fn collect_gem_item_ids(wow_data: &Path) -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("Item.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty Item.csv")?);
    let mut ids = BTreeSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        if field(&fields, &idx, "ClassID") == "3" {
            ids.insert(field(&fields, &idx, "ID").parse()?);
        }
    }
    Ok(ids)
}

fn write_gem_catalog(wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ids = collect_gem_item_ids(wow_data)?;
    let records = open_records(&wow_data.join("ItemSparse.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty ItemSparse.csv")?);
    let mut gems = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        if ids.contains(&id) {
            let name = escape_str(field(&fields, &idx, "Display_lang"));
            let quality = field(&fields, &idx, "OverallQualityID").parse::<u8>()?;
            gems.push((id, name, quality));
        }
    }
    gems.sort_unstable_by_key(|gem| gem.0);
    let mut out = File::create("crates/sim-data/data/gem_items.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(out, "pub static GEMS: &[(u32, &str, u8)] = &[")?;
    for (id, name, quality) in gems {
        writeln!(out, "    ({id}, \"{name}\", {quality}),")?;
    }
    writeln!(out, "];")?;
    Ok(())
}

/// Generate the item classification and localized names used by item-type queries.
fn write_item_categories(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = File::create("crates/sim-data/data/item_categories.rs")?;
    writeln!(
        out,
        "//! Generated from wago.tools DB2 CSV exports, build 12.1.5.70391 (enUS)."
    )?;
    writeln!(out, "//! Regenerate with `wow-cli generate items`.")?;
    writeln!(out, "pub static ITEMS: &[(u32, u8, u8, u8)] = &[")?;
    let records = open_records(&wow_data.join("Item.csv"))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or("empty Item.csv")?);
    let mut items = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = field(&fields, &idx, "ID").parse::<u32>()?;
        if required_ids.contains(&id) {
            let class = field(&fields, &idx, "ClassID").parse::<u8>()?;
            let subclass = field(&fields, &idx, "SubclassID").parse::<u8>()?;
            let inventory = field(&fields, &idx, "InventoryType").parse::<u8>()?;
            items.push((id, class, subclass, inventory));
        }
    }
    items.sort_unstable_by_key(|item| item.0);
    for (id, class, subclass, inventory) in items {
        writeln!(out, "    ({id}, {class}, {subclass}, {inventory}),")?;
    }
    writeln!(out, "];\n")?;
    write_category_names(&mut out, wow_data)?;
    Ok(())
}

fn write_category_names(out: &mut File, wow_data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for (file, name, columns) in [
        (
            "ItemClass.csv",
            "CLASSES",
            &["ClassID", "ClassName_lang"][..],
        ),
        (
            "ItemSubClass.csv",
            "SUBCLASSES",
            &[
                "ClassID",
                "SubClassID",
                "DisplayName_lang",
                "VerboseName_lang",
            ][..],
        ),
        (
            "ItemSubClassMask.csv",
            "MASKS",
            &["ClassID", "Mask", "Name_lang"][..],
        ),
    ] {
        let records = open_records(&wow_data.join(file))?;
        let mut iter = records.iter();
        let idx = header_index(iter.next().ok_or("empty category CSV")?);
        let ty = match name {
            "CLASSES" => "(u8, &str)",
            "SUBCLASSES" => "(u8, u8, &str, &str)",
            _ => "(u8, u32, &str)",
        };
        writeln!(out, "pub static {name}: &[{ty}] = &[")?;
        for record in iter {
            let fields = parse_csv_line(record);
            let values: Vec<String> = columns
                .iter()
                .map(|column| {
                    let value = field(&fields, &idx, column);
                    if column.ends_with("_lang") {
                        format!("\"{}\"", escape_str(value))
                    } else {
                        value.to_owned()
                    }
                })
                .collect();
            writeln!(out, "    ({}),", values.join(", "))?;
        }
        writeln!(out, "];\n")?;
    }
    Ok(())
}

/// Collect item IDs referenced by the simulator from source files.
fn collect_required_item_ids() -> BTreeSet<u32> {
    let mut ids = BTreeSet::new();

    // Equipped items seeded in state defaults: e(211993), e(211995), etc.
    if let Ok(src) = std::fs::read_to_string("src/lua_api/state_defaults.rs") {
        collect_number_literals_after(&src, "e(", &mut ids);
        collect_number_literals_after(&src, "\n        t(", &mut ids);
    }

    // Legacy fallback for older layouts.
    if let Ok(src) = std::fs::read_to_string("src/lua_api/state_types.rs") {
        collect_number_literals_after(&src, "e(", &mut ids);
    }

    // Profession data: item_id, output_item_id, reagent item_ids
    if let Ok(src) = std::fs::read_to_string("src/lua_api/globals/profession_data.rs") {
        collect_number_literals_after(&src, "item_id: ", &mut ids);
        collect_number_literals_after(&src, "output_item_id: ", &mut ids);
    }

    // Legacy fallback for older container stubs.
    if let Ok(src) = std::fs::read_to_string("src/lua_api/globals/c_container_api.rs") {
        collect_number_literals_after(&src, "item_id: ", &mut ids);
    }

    // Store/collection items
    if let Ok(src) = std::fs::read_to_string("src/lua_api/globals/c_stubs_api_store.rs") {
        collect_number_literals_after(&src, "item_id: ", &mut ids);
        collect_number_literals_after(&src, "itemID = ", &mut ids);
    }

    ids.extend(EXISTING_ITEM_FIXTURE_IDS);
    ids.extend(ADDON_COMPAT_ITEM_IDS);

    if let Ok(journal_items) = collect_required_encounter_journal_items(&wow_data_dir()) {
        ids.extend(journal_items);
    }

    // Bag items from state.rs default backpack
    if let Ok(src) = std::fs::read_to_string("src/lua_api/state.rs") {
        collect_number_literals_after(&src, "item_id: ", &mut ids);
    }

    // Everything that teleports: toys and items with a teleport use effect.
    ids.extend(super::gen_teleport_selection::collect(&wow_data_dir()).items);

    // Baseline items always needed
    ids.insert(6948); // Hearthstone (test)

    ids.remove(&0);
    ids
}

fn collect_number_literals_after(src: &str, marker: &str, out: &mut BTreeSet<u32>) {
    let mut rest = src;
    while let Some(idx) = rest.find(marker) {
        rest = &rest[idx + marker.len()..];
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len == 0 {
            continue;
        }
        if let Ok(id) = rest[..digits_len].parse::<u32>() {
            if id != 0 {
                out.insert(id);
            }
        }
        rest = &rest[digits_len..];
    }
}

const EXISTING_ITEM_FIXTURE_IDS: &[u32] = &[
    // Existing compact item fixture rows. Keep these explicit so regenerating
    // the file adds targeted journal coverage without pruning older fixtures.
    159, 4540, 6948, 7005, 122245, 210934, 210935, 211988, 211989, 211990, 211991, 211992, 211993,
    211994, 211995, 211996, 215135, 218715, 225748, 229181, 230637, 236914,
    // tests/wrath_inventory_slot_candidates.rs equips and bags these two rings
    // and head item and asserts their equip locations through GetItemInfo.
    // The 12.1.5.70391 journal tiers no longer list them.
    49803, 49812, 49819,
    // tests/item_spec_info_* read these through GetItemInfo (WM48 examples).
    892, 80773,
];

/// Items addons name at file load that the teleport rule cannot see: their
/// use effect opens a portal object or runs a script instead of teleporting.
const ADDON_COMPAT_ITEM_IDS: &[u32] = &[
    // QuickRoute/Data/TeleportItems.lua
    37863,  // Direbrew's Remote (portal object)
    52251,  // Jaina's Locket (portal object)
    128353, // Admiral's Compass (scripted shipyard teleport)
    129276, // Beginner's Guide to Dimensional Rifting (dummy effect)
    140493, // Adept's Guide to Dimensional Rifting (dummy effect)
];

const REQUIRED_ENCOUNTER_JOURNAL_TIER_IDS: &[u32] = &[
    516, // Midnight dungeons and raids
    514, // The War Within; left "Current Season" in 12.1.5.70391
    505, // Suggested/current Adventure Guide dungeons and raids
];

fn collect_required_encounter_journal_items(
    wow_data: &Path,
) -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let instance_ids = collect_journal_instance_ids_for_required_tiers(wow_data)?;
    let encounter_ids = collect_journal_encounter_ids_for_instances(wow_data, &instance_ids)?;
    collect_journal_item_ids_for_encounters(wow_data, &encounter_ids)
}

fn collect_journal_instance_ids_for_required_tiers(
    wow_data: &Path,
) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
    let required_tiers: HashSet<u32> = REQUIRED_ENCOUNTER_JOURNAL_TIER_IDS
        .iter()
        .copied()
        .collect();
    let records = open_records(&wow_data.join("JournalTierXInstance.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty JournalTierXInstance.csv")?;
    let idx = header_index(header);
    let mut instance_ids = HashSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let tier_id = parse_u32(field(&fields, &idx, "JournalTierID"));
        if required_tiers.contains(&tier_id) {
            instance_ids.insert(parse_u32(field(&fields, &idx, "JournalInstanceID")));
        }
    }
    Ok(instance_ids)
}

fn collect_journal_encounter_ids_for_instances(
    wow_data: &Path,
    instance_ids: &HashSet<u32>,
) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("JournalEncounter.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty JournalEncounter.csv")?;
    let idx = header_index(header);
    let mut encounter_ids = HashSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let instance_id = parse_u32(field(&fields, &idx, "JournalInstanceID"));
        if instance_ids.contains(&instance_id) {
            encounter_ids.insert(parse_u32(field(&fields, &idx, "ID")));
        }
    }
    Ok(encounter_ids)
}

fn collect_journal_item_ids_for_encounters(
    wow_data: &Path,
    encounter_ids: &HashSet<u32>,
) -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("JournalEncounterItem.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty JournalEncounterItem.csv")?;
    let idx = header_index(header);
    let mut item_ids = BTreeSet::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let encounter_id = parse_u32(field(&fields, &idx, "JournalEncounterID"));
        if encounter_ids.contains(&encounter_id) {
            item_ids.insert(parse_u32(field(&fields, &idx, "ItemID")));
        }
    }
    item_ids.remove(&0);
    Ok(item_ids)
}

/// One CSV record addressed by header name.
struct CsvRow<'a> {
    fields: Vec<String>,
    idx: &'a HashMap<String, usize>,
}

impl CsvRow<'_> {
    fn get(&self, key: &str) -> &str {
        field(&self.fields, self.idx, key)
    }
}

fn for_each_csv_row(
    wow_data: &Path,
    name: &str,
    mut visit: impl FnMut(&CsvRow) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join(name))?;
    let mut iter = records.iter();
    let idx = header_index(iter.next().ok_or_else(|| format!("empty {name}"))?);
    for record in iter {
        visit(&CsvRow {
            fields: parse_csv_line(record),
            idx: &idx,
        })?;
    }
    Ok(())
}

fn write_rows<T: std::fmt::Debug>(
    out: &mut File,
    declaration: &str,
    rows: &[T],
) -> std::io::Result<()> {
    writeln!(out, "\npub static {declaration} = &[")?;
    for row in rows {
        writeln!(out, "    {row:?},")?;
    }
    writeln!(out, "];")
}

fn open_records(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(read_csv_records(reader)?)
}

fn header_index(header: &str) -> HashMap<String, usize> {
    parse_csv_line(header)
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect()
}

fn field<'a>(fields: &'a [String], idx: &HashMap<String, usize>, key: &str) -> &'a str {
    idx.get(key)
        .and_then(|i| fields.get(*i))
        .map(String::as_str)
        .unwrap_or("")
}

fn parse_u32(s: &str) -> u32 {
    s.parse().unwrap_or(0)
}

/// Build a HashMap<item_id, icon_file_data_id> from ItemModifiedAppearance + ItemAppearance CSVs.
/// Only loads appearances for items in `required_ids`.
fn build_icon_map(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
    let appearance_map = parse_appearance_icons(wow_data)?;
    let mut icon_map = resolve_item_icons(wow_data, required_ids, &appearance_map)?;

    // Non-equippable items (consumables, utility items) can be missing from
    // ItemModifiedAppearance. Seed explicit icon fileDataIDs for baseline items
    // the simulator always places in the backpack.
    for (item_id, icon_file_data_id) in required_item_icon_overrides() {
        if required_ids.contains(item_id) {
            icon_map.entry(*item_id).or_insert(*icon_file_data_id);
        }
    }

    // Item.csv carries every item's icon, equippable or not; it fills what
    // the appearance tables (equippables only) and the overrides left open.
    add_item_table_icons(wow_data, required_ids, &mut icon_map)?;

    add_spell_name_icon_fallbacks(wow_data, required_ids, &mut icon_map)?;

    Ok(icon_map)
}

/// Item.csv: `ID` -> `IconFileDataID`, for the required items still without an icon.
fn add_item_table_icons(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
    icon_map: &mut HashMap<u32, u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = wow_data.join("Item.csv");
    if !path.exists() {
        println!("Item.csv missing, icons from Item.csv skipped");
        return Ok(());
    }
    let records = open_records(&path)?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty Item.csv")?;
    let idx = header_index(header);
    for record in iter {
        let fields = parse_csv_line(record);
        let item_id = parse_u32(field(&fields, &idx, "ID"));
        let icon = parse_u32(field(&fields, &idx, "IconFileDataID"));
        if icon != 0 && required_ids.contains(&item_id) {
            icon_map.entry(item_id).or_insert(icon);
        }
    }
    Ok(())
}

fn required_item_icon_overrides() -> &'static [(u32, u32)] {
    &[
        // Hearthstone
        (6948, 134414), // ICONS/INV_MISC_RUNE_01
        // Refreshing Spring Water
        (159, 132788), // ICONS/INV_Drink_01
        // Tough Hunk of Bread
        (4540, 133964), // ICONS/INV_MISC_FOOD_11
    ]
}

const GENERIC_SPELL_ICON_FILE_DATA_ID: u32 = 136243;

fn add_spell_name_icon_fallbacks(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
    icon_map: &mut HashMap<u32, u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let item_names = required_item_names(wow_data, required_ids)?;
    let spell_icons = spell_icon_file_data_ids(wow_data)?;
    let spell_name_icons = spell_name_icon_file_data_ids(wow_data, &spell_icons)?;

    for (item_id, name) in item_names {
        if icon_map.contains_key(&item_id) {
            continue;
        }
        if let Some(icon_file_data_id) = spell_name_icons.get(&name) {
            icon_map.insert(item_id, *icon_file_data_id);
        }
    }

    Ok(())
}

fn required_item_names(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let mut item_names = required_item_names_from_csv(wow_data, "ItemSparse.csv", required_ids)?;
    for (item_id, name) in
        required_item_names_from_csv(wow_data, "ItemSearchName.csv", required_ids)?
    {
        item_names.entry(item_id).or_insert(name);
    }
    Ok(item_names)
}

fn required_item_names_from_csv(
    wow_data: &Path,
    file_name: &str,
    required_ids: &BTreeSet<u32>,
) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join(file_name))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or_else(|| format!("empty {file_name}"))?;
    let idx = header_index(header);
    let mut item_names = HashMap::new();

    for record in iter {
        let fields = parse_csv_line(record);
        let item_id = parse_u32(field(&fields, &idx, "ID"));
        let name = field(&fields, &idx, "Display_lang");
        if required_ids.contains(&item_id) && !name.is_empty() {
            item_names.insert(item_id, name.to_string());
        }
    }

    Ok(item_names)
}

fn spell_icon_file_data_ids(
    wow_data: &Path,
) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("SpellMisc.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty SpellMisc.csv")?;
    let idx = header_index(header);
    let mut spell_icons = HashMap::new();

    for record in iter {
        let fields = parse_csv_line(record);
        let spell_id = parse_u32(field(&fields, &idx, "SpellID"));
        let icon_file_data_id = parse_u32(field(&fields, &idx, "SpellIconFileDataID"));
        if spell_id != 0
            && icon_file_data_id != 0
            && icon_file_data_id != GENERIC_SPELL_ICON_FILE_DATA_ID
        {
            spell_icons.insert(spell_id, icon_file_data_id);
        }
    }

    Ok(spell_icons)
}

fn spell_name_icon_file_data_ids(
    wow_data: &Path,
    spell_icons: &HashMap<u32, u32>,
) -> Result<HashMap<String, u32>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("SpellName.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty SpellName.csv")?;
    let idx = header_index(header);
    let mut spell_name_icons = HashMap::new();

    for record in iter {
        let fields = parse_csv_line(record);
        let spell_id = parse_u32(field(&fields, &idx, "ID"));
        let name = field(&fields, &idx, "Name_lang");
        if name.is_empty() {
            continue;
        }
        if let Some(icon_file_data_id) = spell_icons.get(&spell_id) {
            spell_name_icons
                .entry(name.to_string())
                .or_insert(*icon_file_data_id);
        }
    }

    Ok(spell_name_icons)
}

/// Parse ItemAppearance.csv: appearance_id → icon fileDataID.
fn parse_appearance_icons(
    wow_data: &Path,
) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
    let file = File::open(wow_data.join("ItemAppearance.csv"))?;
    let mut map: HashMap<u32, u32> = HashMap::new();
    for (i, line) in read_csv_records(BufReader::new(file))?
        .into_iter()
        .enumerate()
    {
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() < 4 {
            continue;
        }
        let Ok(appearance_id) = fields[0].parse::<u32>() else {
            continue;
        };
        let icon: u32 = fields[3].parse().unwrap_or(0);
        map.insert(appearance_id, icon);
    }
    Ok(map)
}

/// Parse ItemModifiedAppearance.csv: item_id → icon fileDataID (first match per item).
fn resolve_item_icons(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
    appearance_map: &HashMap<u32, u32>,
) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
    let file = File::open(wow_data.join("ItemModifiedAppearance.csv"))?;
    let mut icon_map: HashMap<u32, u32> = HashMap::new();
    for (i, line) in read_csv_records(BufReader::new(file))?
        .into_iter()
        .enumerate()
    {
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() < 4 {
            continue;
        }
        let Ok(item_id) = fields[1].parse::<u32>() else {
            continue;
        };
        if !required_ids.contains(&item_id) || icon_map.contains_key(&item_id) {
            continue;
        }
        let appearance_id: u32 = fields[3].parse().unwrap_or(0);
        let icon = appearance_map.get(&appearance_id).copied().unwrap_or(0);
        if icon != 0 {
            icon_map.insert(item_id, icon);
        }
    }
    Ok(icon_map)
}

/// The ItemSparse columns `parse_item_row` and `format_item_info` read. The
/// array columns are exempt: `array_field` accepts either spelling and a
/// missing one is indistinguishable from an empty value.
const ITEM_SPARSE_REQUIRED_COLUMNS: &[&str] = &[
    "ID",
    "Display_lang",
    "ExpansionID",
    "Stackable",
    "SellPrice",
    "ItemLevel",
    "Bonding",
    "RequiredLevel",
    "InventoryType",
    "OverallQualityID",
];

fn build_item_map(
    out: &mut File,
    reader: BufReader<File>,
    icon_map: &HashMap<u32, u32>,
    required_ids: &BTreeSet<u32>,
    wow_data: &Path,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut emitted_ids = BTreeSet::new();
    let mut count = 0u32;
    let mut skipped = 0u32;

    let mut columns: Option<HashMap<String, usize>> = None;
    for line in read_csv_records(reader)? {
        let Some(idx) = columns.as_ref() else {
            // Columns by name: wago exports reorder them between builds, and
            // positional indices then read the wrong field without a sound.
            let idx = header_index(&line);
            // Every scalar column a row is read through, not just the ones
            // that decide whether the row is kept: `field()` answers "" for a
            // column that is not there, so a rename would quietly emit the
            // default for every item -- the same silent wrong value that
            // positional indices produced.
            for required in ITEM_SPARSE_REQUIRED_COLUMNS {
                if !idx.contains_key(*required) {
                    return Err(format!("ItemSparse.csv has no {required} column").into());
                }
            }
            columns = Some(idx);
            continue;
        };
        match parse_item_row(&line, idx, icon_map) {
            Some((id, value)) if required_ids.contains(&id) => {
                builder.entry(id, &value);
                emitted_ids.insert(id);
                count += 1;
            }
            _ => {
                skipped += 1;
            }
        }
    }

    for (id, value) in fallback_item_search_rows(wow_data, required_ids, &emitted_ids)? {
        builder.entry(id, &value);
        count += 1;
    }

    writeln!(
        out,
        "pub static ITEM_DB: phf::Map<u32, ItemInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok((count, skipped))
}

fn fallback_item_search_rows(
    wow_data: &Path,
    required_ids: &BTreeSet<u32>,
    emitted_ids: &BTreeSet<u32>,
) -> Result<Vec<(u32, String)>, Box<dyn std::error::Error>> {
    let records = open_records(&wow_data.join("ItemSearchName.csv"))?;
    let mut iter = records.iter();
    let header = iter.next().ok_or("empty ItemSearchName.csv")?;
    let idx = header_index(header);
    let mut missing_ids: BTreeSet<u32> = required_ids.difference(emitted_ids).copied().collect();
    let mut rows = Vec::new();
    for record in iter {
        let fields = parse_csv_line(record);
        let id = parse_u32(field(&fields, &idx, "ID"));
        if missing_ids.remove(&id) {
            rows.push((id, format_search_item_info(&fields, &idx)));
        }
    }
    Ok(rows)
}

fn parse_item_row(
    line: &str,
    idx: &HashMap<String, usize>,
    icon_map: &HashMap<u32, u32>,
) -> Option<(u32, String)> {
    let fields = parse_csv_line(line);
    let id: u32 = field(&fields, idx, "ID").parse().ok()?;
    let name = field(&fields, idx, "Display_lang");
    if name.is_empty() {
        return None;
    }
    let icon_file_data_id = icon_map.get(&id).copied().unwrap_or(0);
    Some((id, format_item_info(&fields, idx, name, icon_file_data_id)))
}

fn format_item_info(
    fields: &[String],
    idx: &HashMap<String, usize>,
    name: &str,
    icon_file_data_id: u32,
) -> String {
    let escaped_name = escape_str(name);
    let expansion_id: u8 = field(fields, idx, "ExpansionID").parse().unwrap_or(0);
    let stackable: u32 = field(fields, idx, "Stackable").parse().unwrap_or(1);
    let sell_price: u32 = field(fields, idx, "SellPrice").parse().unwrap_or(0);
    let item_level: u16 = field(fields, idx, "ItemLevel").parse().unwrap_or(0);
    let bonding: u8 = field(fields, idx, "Bonding").parse().unwrap_or(0);
    let required_level: u16 = field(fields, idx, "RequiredLevel").parse().unwrap_or(0);
    let inventory_type: u8 = field(fields, idx, "InventoryType").parse().unwrap_or(0);
    let quality: u8 = field(fields, idx, "OverallQualityID").parse().unwrap_or(0);
    let stat_percent_editor = parse_stat_percent_editor(fields, idx);
    let stat_modifier_bonus_stat = parse_stat_modifier_bonus_stat(fields, idx);

    let stat_percent_editor = format_u16_array(&stat_percent_editor);
    let stat_modifier_bonus_stat = format_i16_array(&stat_modifier_bonus_stat);
    format!(
        "ItemInfo {{ name: \"{escaped_name}\", quality: {quality}, item_level: {item_level}, \
         required_level: {required_level}, inventory_type: {inventory_type}, \
         sell_price: {sell_price}, stackable: {stackable}, bonding: {bonding}, \
         expansion_id: {expansion_id}, icon_file_data_id: {icon_file_data_id}, \
         stat_percent_editor: {stat_percent_editor}, \
         stat_modifier_bonus_stat: {stat_modifier_bonus_stat} }}"
    )
}

fn format_search_item_info(fields: &[String], idx: &HashMap<String, usize>) -> String {
    let name = escape_str(field(fields, idx, "Display_lang"));
    let quality: u8 = field(fields, idx, "OverallQualityID").parse().unwrap_or(0);
    let item_level: u16 = field(fields, idx, "ItemLevel").parse().unwrap_or(0);
    let required_level: u16 = field(fields, idx, "RequiredLevel").parse().unwrap_or(0);
    let expansion_id: u8 = field(fields, idx, "ExpansionID").parse().unwrap_or(0);
    format!(
        "ItemInfo {{ name: \"{name}\", quality: {quality}, item_level: {item_level}, \
         required_level: {required_level}, inventory_type: 0, sell_price: 0, \
         stackable: 1, bonding: 1, expansion_id: {expansion_id}, icon_file_data_id: 0, \
         stat_percent_editor: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], \
         stat_modifier_bonus_stat: [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1] }}"
    )
}

/// An array column, spelled `Name_0` in current wago exports and `Name[0]` in older ones.
fn array_field<'a>(
    fields: &'a [String],
    idx: &HashMap<String, usize>,
    base: &str,
    index: usize,
) -> &'a str {
    let underscore = format!("{base}_{index}");
    let bracket = format!("{base}[{index}]");
    let value = field(fields, idx, &underscore);
    if value.is_empty() {
        field(fields, idx, &bracket)
    } else {
        value
    }
}

fn parse_stat_percent_editor(fields: &[String], idx: &HashMap<String, usize>) -> [u16; 10] {
    std::array::from_fn(|index| {
        array_field(fields, idx, "StatPercentEditor", index)
            .parse::<u16>()
            .unwrap_or(0)
    })
}

fn parse_stat_modifier_bonus_stat(fields: &[String], idx: &HashMap<String, usize>) -> [i16; 10] {
    std::array::from_fn(|index| {
        array_field(fields, idx, "StatModifier_bonusStat", index)
            .parse::<i16>()
            .unwrap_or(-1)
    })
}

fn format_array<T: std::fmt::Display>(values: &[T]) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&value.to_string());
    }
    out.push(']');
    out
}

fn format_u16_array(values: &[u16; 10]) -> String {
    format_array(values)
}

fn format_i16_array(values: &[i16; 10]) -> String {
    format_array(values)
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated item data from WoW CSV exports.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate items"
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct ItemInfo {{")?;
    writeln!(out, "    pub name: &'static str,")?;
    writeln!(out, "    pub quality: u8,")?;
    writeln!(out, "    pub item_level: u16,")?;
    writeln!(out, "    pub required_level: u16,")?;
    writeln!(out, "    pub inventory_type: u8,")?;
    writeln!(out, "    pub sell_price: u32,")?;
    writeln!(out, "    pub stackable: u32,")?;
    writeln!(out, "    pub bonding: u8,")?;
    writeln!(out, "    pub expansion_id: u8,")?;
    writeln!(out, "    pub icon_file_data_id: u32,")?;
    writeln!(out, "    pub stat_percent_editor: [u16; 10],")?;
    writeln!(out, "    pub stat_modifier_bonus_stat: [i16; 10],")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_item(id: u32) -> Option<&'static ItemInfo> {{"
    )?;
    writeln!(
        out,
        "    crate::profession_item_overrides::get_item(id).or_else(|| ITEM_DB.get(&id))"
    )?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    write_test_item_count(out)?;
    write_test_hearthstone(out)?;
    write_test_default_backpack_consumable_icons(out)?;
    write_test_nonexistent_item(out)?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_test_item_count(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_item_count() {{")?;
    writeln!(out, "        assert!(ITEM_DB.len() > 10);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_hearthstone(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_hearthstone() {{")?;
    writeln!(
        out,
        "        let item = get_item(6948).expect(\"Hearthstone (6948) should exist\");"
    )?;
    writeln!(out, "        assert_eq!(item.name, \"Hearthstone\");")?;
    writeln!(out, "        assert_eq!(item.quality, 1);")?;
    writeln!(out, "        assert_eq!(item.icon_file_data_id, 134414);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_default_backpack_consumable_icons(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_default_backpack_consumable_icons() {{")?;
    writeln!(
        out,
        "        let water = get_item(159).expect(\"Refreshing Spring Water (159) should exist\");"
    )?;
    writeln!(
        out,
        "        let bread = get_item(4540).expect(\"Tough Hunk of Bread (4540) should exist\");"
    )?;
    writeln!(out, "        assert_eq!(water.icon_file_data_id, 132788);")?;
    writeln!(out, "        assert_eq!(bread.icon_file_data_id, 133964);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_nonexistent_item(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_item() {{")?;
    writeln!(out, "        assert!(get_item(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    Ok(())
}
