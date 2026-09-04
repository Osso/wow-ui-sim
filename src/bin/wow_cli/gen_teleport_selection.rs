//! Everything that teleports, as a selection rule shared by the item and
//! spell generators.
//!
//! The compact item and spell tables carry what the simulator's own sources
//! reference. Teleport addons (hearthstone managers, route planners) name
//! items and spells the simulator never mentions, and the client answers
//! them from ItemSparse / SpellName. This rule adds that slice without
//! naming any addon:
//!
//! - every toy (`Toy.csv`), the collection tab every UI can open;
//! - every item whose use effect casts a teleport
//!   (`ItemXItemEffect.csv` -> `ItemEffect.csv` -> `SpellEffect.csv`);
//! - the spells those items cast, and every teleport spell on a class
//!   skill line (`SkillLineAbility.csv` with `SkillLine.csv` category 7).
//!
//! "Teleport" is `SpellEffect.Effect` 15 or 252: Teleport: Stormwind (3561)
//! carries 15, Hearthstone (8690) and Astral Recall (556) carry 252, in the
//! 12.1.0 export. Effect 252 alone matches thousands of NPC and scripted
//! spells ("... (DND)"), which is why spells are taken through items and
//! class lines rather than by effect alone.
//!
//! Reads from ~/Projects/wow/data/. Every table is optional: a missing file
//! contributes nothing and is reported, so an older data directory still
//! regenerates the tables it always did.

use super::csv_util::{parse_csv_line, read_csv_records};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const TELEPORT_EFFECTS: &[&str] = &["15", "252"];
const SKILL_LINE_CATEGORY_CLASS: &str = "7";

#[derive(Debug, Default)]
pub struct TeleportSelection {
    /// Toys plus items with a teleport use effect.
    pub items: BTreeSet<u32>,
    /// The use spells of those items plus class-line teleport spells.
    pub spells: BTreeSet<u32>,
}

pub fn collect(wow_data: &Path) -> TeleportSelection {
    let mut selection = TeleportSelection::default();

    let Some(teleport_spells) = teleport_spells(wow_data) else {
        return selection;
    };

    // Toys.
    for row in rows(wow_data, "Toy.csv") {
        if let Some(item_id) = row.get("ItemID").and_then(|s| s.parse().ok()) {
            selection.items.insert(item_id);
        }
    }
    let toy_count = selection.items.len();

    // Items whose use effect teleports, and the spells they cast.
    let mut effect_spell: HashMap<u32, u32> = HashMap::new();
    for row in rows(wow_data, "ItemEffect.csv") {
        if let (Some(id), Some(spell)) = (parse(row.get("ID")), parse(row.get("SpellID"))) {
            effect_spell.insert(id, spell);
        }
    }
    let mut use_spells: HashSet<u32> = HashSet::new();
    let mut item_spell: Vec<(u32, u32)> = Vec::new();
    for row in rows(wow_data, "ItemXItemEffect.csv") {
        let (Some(item), Some(effect)) = (parse(row.get("ItemID")), parse(row.get("ItemEffectID")))
        else {
            continue;
        };
        if let Some(spell) = effect_spell.get(&effect) {
            item_spell.push((item, *spell));
        }
    }
    for (item, spell) in &item_spell {
        if teleport_spells.contains(spell) {
            selection.items.insert(*item);
        }
    }
    // A selected item's use spells, teleporting or not (a toy's spell is
    // what the tooltip and the cooldown are read from).
    for (item, spell) in &item_spell {
        if selection.items.contains(item) {
            use_spells.insert(*spell);
        }
    }
    selection.spells.extend(use_spells.iter().copied());

    // Teleport spells on a class skill line.
    let class_lines: HashSet<u32> = rows(wow_data, "SkillLine.csv")
        .into_iter()
        .filter(|row| row.get("CategoryID").map(String::as_str) == Some(SKILL_LINE_CATEGORY_CLASS))
        .filter_map(|row| parse(row.get("ID")))
        .collect();
    let mut class_teleports = 0usize;
    for row in rows(wow_data, "SkillLineAbility.csv") {
        let (Some(spell), Some(line)) = (parse(row.get("Spell")), parse(row.get("SkillLine")))
        else {
            continue;
        };
        if class_lines.contains(&line) && teleport_spells.contains(&spell) && selection.spells.insert(spell) {
            class_teleports += 1;
        }
    }

    println!(
        "Teleport selection: {} items ({} toys), {} spells ({} class-line teleports)",
        selection.items.len(),
        toy_count,
        selection.spells.len(),
        class_teleports
    );
    selection
}

/// Spell IDs with a teleport effect, or `None` when SpellEffect.csv is absent.
fn teleport_spells(wow_data: &Path) -> Option<HashSet<u32>> {
    let path = wow_data.join("SpellEffect.csv");
    if !path.exists() {
        println!("Teleport selection: {} missing, skipped", path.display());
        return None;
    }
    let mut spells = HashSet::new();
    for row in rows(wow_data, "SpellEffect.csv") {
        let is_teleport = row
            .get("Effect")
            .map(|e| TELEPORT_EFFECTS.contains(&e.as_str()))
            .unwrap_or(false);
        if is_teleport {
            if let Some(spell) = parse(row.get("SpellID")) {
                spells.insert(spell);
            }
        }
    }
    Some(spells)
}

fn parse(field: Option<&String>) -> Option<u32> {
    field.and_then(|s| s.parse().ok())
}

/// The rows of a CSV as header-keyed maps; an absent file yields no rows.
fn rows(wow_data: &Path, file_name: &str) -> Vec<HashMap<String, String>> {
    let path = wow_data.join(file_name);
    let Ok(file) = File::open(&path) else {
        println!("Teleport selection: {} missing, skipped", path.display());
        return Vec::new();
    };
    let Ok(records) = read_csv_records(BufReader::new(file)) else {
        return Vec::new();
    };
    let mut iter = records.iter();
    let Some(header) = iter.next() else {
        return Vec::new();
    };
    let names = parse_csv_line(header);
    iter.map(|record| {
        parse_csv_line(record)
            .into_iter()
            .enumerate()
            .filter_map(|(i, value)| names.get(i).map(|name| (name.clone(), value)))
            .collect()
    })
    .collect()
}
