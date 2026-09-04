//! Generator for items.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - ItemSparse.csv
//!   - ItemModifiedAppearance.csv
//!   - ItemAppearance.csv
//!
//! Generates: data/items.rs

use super::csv_util::{escape_str, parse_csv_line, read_csv_records, wow_data_dir};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let csv_path = wow_data.join("ItemSparse.csv");
    println!("Loading ItemSparse from {}...", csv_path.display());

    let required_ids = collect_required_item_ids();
    println!("Required item IDs: {} (deduplicated)", required_ids.len());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    let icon_map = build_icon_map(&wow_data, &required_ids)?;

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/items.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;

    let (count, skipped) = build_item_map(&mut out, reader, &icon_map, &required_ids, &wow_data)?;

    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!("Generated {} item entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());
    Ok(())
}

/// Collect item IDs referenced by the simulator from source files.
fn collect_required_item_ids() -> BTreeSet<u32> {
    let mut ids = BTreeSet::new();

    // Equipped items seeded in state defaults: e(211993), e(211995), etc.
    if let Ok(src) = std::fs::read_to_string("src/lua_api/state_defaults.rs") {
        collect_number_literals_after(&src, "e(", &mut ids);
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
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
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
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
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
    for line in reader.lines() {
        let line = line?;
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
fn array_field<'a>(fields: &'a [String], idx: &HashMap<String, usize>, base: &str, index: usize) -> &'a str {
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
