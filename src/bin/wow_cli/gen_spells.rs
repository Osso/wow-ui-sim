//! Generator for spells.rs and spell_power.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - SpellName.csv (ID, Name_lang)
//!   - Spell.csv (ID, NameSubtext_lang, ...)
//!   - SpellMisc.csv (ID, ..., DifficultyID[18], SchoolMask[23],
//!     SpellIconFileDataID[27], SpellID[33])
//!   - SpellPower.csv (ID, OrderIndex, ManaCost, PowerCostPct, PowerType,
//!     RequiredAuraSpellID, OptionalCost, SpellID, ...)
//!   - SpellEffect.csv (ID, ..., DifficultyID[2], EffectIndex[3],
//!     ImplicitTarget_0[34], SpellID[36])
//!
//! Generates: data/spells.rs, data/spell_power.rs

use super::csv_util::{escape_str, parse_csv_line, wow_data_dir};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();

    let spell_names = load_spell_names(&wow_data.join("SpellName.csv"))?;
    println!("SpellName: {} entries", spell_names.len());

    let spell_subtexts = load_spell_subtexts(&wow_data.join("Spell.csv"))?;
    println!("Spell (subtexts): {} entries", spell_subtexts.len());

    let spell_misc = load_spell_misc(&wow_data.join("SpellMisc.csv"))?;
    println!("SpellMisc (DifficultyID=0): {} entries", spell_misc.len());

    let spell_power = load_spell_power(&wow_data.join("SpellPower.csv"))?;
    println!("SpellPower: {} spells with costs", spell_power.len());

    let spell_targets = load_spell_effect_targets(&wow_data.join("SpellEffect.csv"))?;
    println!("SpellEffect (targets): {} spells", spell_targets.len());

    std::fs::create_dir_all("data")?;

    // Generate data/spells.rs
    let output_path = Path::new("data/spells.rs");
    let mut out = File::create(output_path)?;
    write_header(&mut out)?;
    let count = build_spell_map(
        &mut out, &spell_names, &spell_subtexts, &spell_misc, &spell_targets,
    )?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;
    println!("Generated {} spell entries", count);
    println!("Output: {}", output_path.display());

    // Generate data/spell_power.rs
    let power_path = Path::new("data/spell_power.rs");
    let mut power_out = File::create(power_path)?;
    let power_count = write_spell_power(&mut power_out, &spell_power)?;
    println!("Generated {} spell power entries", power_count);
    println!("Output: {}", power_path.display());

    Ok(())
}

fn build_spell_map(
    out: &mut File,
    spell_names: &HashMap<u32, String>,
    spell_subtexts: &HashMap<u32, String>,
    spell_misc: &HashMap<u32, (u32, u32)>,
    spell_targets: &HashMap<u32, u8>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;

    for (&spell_id, name) in spell_names {
        let escaped_name = escape_str(name);
        let subtext = spell_subtexts
            .get(&spell_id)
            .map(|s| escape_str(s))
            .unwrap_or_default();
        let (icon, school) = spell_misc
            .get(&spell_id)
            .copied()
            .unwrap_or((136243, 0));
        let implicit_target = spell_targets
            .get(&spell_id)
            .copied()
            .unwrap_or(0);

        let value = format!(
            "SpellInfo {{ name: \"{}\", subtext: \"{}\", icon_file_data_id: {}, \
             school_mask: {}, implicit_target: {} }}",
            escaped_name, subtext, icon, school, implicit_target
        );
        builder.entry(spell_id, &value);
        count += 1;
    }

    writeln!(
        out,
        "pub static SPELL_DB: phf::Map<u32, SpellInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok(count)
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated spell data from WoW CSV exports.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate spells"
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct SpellInfo {{")?;
    writeln!(out, "    pub name: &'static str,")?;
    writeln!(out, "    pub subtext: &'static str,")?;
    writeln!(out, "    pub icon_file_data_id: u32,")?;
    writeln!(out, "    pub school_mask: u32,")?;
    writeln!(out, "    /// ImplicitTarget_0 from first SpellEffect (EffectIndex=0, DifficultyID=0).")?;
    writeln!(out, "    /// Determines valid target type: 1=Self, 6=Enemy, 21=Ally, 25=Any, etc.")?;
    writeln!(out, "    pub implicit_target: u8,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub fn get_spell(id: u32) -> Option<&'static SpellInfo> {{")?;
    writeln!(out, "    SPELL_DB.get(&id)")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_spell_count() {{")?;
    writeln!(out, "        assert!(SPELL_DB.len() > 300_000);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_frostbolt() {{")?;
    writeln!(out, "        let spell = get_spell(116).expect(\"spell 116 should exist\");")?;
    writeln!(out, "        assert_eq!(spell.name, \"Frostbolt\");")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_spell() {{")?;
    writeln!(out, "        assert!(get_spell(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn load_spell_names(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() >= 2
            && let Ok(id) = fields[0].parse::<u32>() {
                map.insert(id, fields[1].clone());
            }
    }
    Ok(map)
}

fn load_spell_subtexts(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() >= 2
            && let Ok(id) = fields[0].parse::<u32>() {
                let subtext = &fields[1];
                if !subtext.is_empty() {
                    map.insert(id, subtext.clone());
                }
            }
    }
    Ok(map)
}

/// Parsed SpellPower row.
struct SpellPowerRow {
    power_type: i8,
    mana_cost: i32,
    cost_pct: f32,
    cost_max_pct: f32,
    cost_per_sec: f32,
    required_aura_id: u32,
    optional_cost: i32,
    order_index: u32,
}

/// Load SpellPower.csv grouped by SpellID, sorted by OrderIndex.
///
/// Columns: ID(0), OrderIndex(1), ManaCost(2), ManaCostPerLevel(3),
/// ManaPerSecond(4), PowerDisplayID(5), AltPowerBarID(6), PowerCostPct(7),
/// PowerCostMaxPct(8), OptionalCostPct(9), PowerPctPerSecond(10),
/// PowerType(11), RequiredAuraSpellID(12), OptionalCost(13), SpellID(14)
fn load_spell_power(
    path: &Path,
) -> Result<HashMap<u32, Vec<SpellPowerRow>>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map: HashMap<u32, Vec<SpellPowerRow>> = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 15 {
            continue;
        }
        let spell_id: u32 = match f[14].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let row = SpellPowerRow {
            order_index: f[1].parse().unwrap_or(0),
            mana_cost: f[2].parse().unwrap_or(0),
            cost_per_sec: f[4].parse().unwrap_or(0.0),
            cost_pct: f[7].parse().unwrap_or(0.0),
            cost_max_pct: f[8].parse().unwrap_or(0.0),
            power_type: f[11].parse().unwrap_or(0),
            required_aura_id: f[12].parse().unwrap_or(0),
            optional_cost: f[13].parse().unwrap_or(0),
        };
        map.entry(spell_id).or_default().push(row);
    }

    // Sort each spell's entries by OrderIndex
    for entries in map.values_mut() {
        entries.sort_by_key(|e| e.order_index);
    }
    Ok(map)
}

/// Generate data/spell_power.rs with static arrays + phf map.
fn write_spell_power(
    out: &mut File,
    spell_power: &HashMap<u32, Vec<SpellPowerRow>>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut spell_ids: Vec<u32> = spell_power.keys().copied().collect();
    spell_ids.sort();

    write_spell_power_header(out)?;
    write_spell_power_arrays(out, &spell_ids, spell_power)?;
    write_spell_power_phf_map(out, &spell_ids)?;
    write_spell_power_lookup_fns(out)?;
    write_spell_power_tests(out)?;

    Ok(spell_ids.len() as u32)
}

fn write_spell_power_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated spell power cost data from WoW SpellPower.csv.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: wow-cli generate spells")?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(out, "pub struct SpellPowerCost {{")?;
    writeln!(out, "    pub power_type: i8,")?;
    writeln!(out, "    pub mana_cost: i32,")?;
    writeln!(out, "    pub cost_pct: f32,")?;
    writeln!(out, "    pub cost_max_pct: f32,")?;
    writeln!(out, "    pub cost_per_sec: f32,")?;
    writeln!(out, "    pub required_aura_id: u32,")?;
    writeln!(out, "    pub optional_cost: i32,")?;
    writeln!(out, "}}")?;
    writeln!(out)
}

fn write_spell_power_arrays(
    out: &mut File,
    spell_ids: &[u32],
    spell_power: &HashMap<u32, Vec<SpellPowerRow>>,
) -> std::io::Result<()> {
    for &spell_id in spell_ids {
        let entries = &spell_power[&spell_id];
        writeln!(out, "static SPELL_POWER_{spell_id}: [SpellPowerCost; {}] = [", entries.len())?;
        for e in entries {
            writeln!(
                out,
                "    SpellPowerCost {{ power_type: {}, mana_cost: {}, cost_pct: {:?}_f32, \
                 cost_max_pct: {:?}_f32, cost_per_sec: {:?}_f32, required_aura_id: {}, \
                 optional_cost: {} }},",
                e.power_type, e.mana_cost, e.cost_pct, e.cost_max_pct,
                e.cost_per_sec, e.required_aura_id, e.optional_cost,
            )?;
        }
        writeln!(out, "];")?;
    }
    writeln!(out)
}

fn write_spell_power_phf_map(
    out: &mut File,
    spell_ids: &[u32],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for &spell_id in spell_ids {
        builder.entry(spell_id, &format!("&SPELL_POWER_{spell_id}"));
    }
    writeln!(
        out,
        "pub static SPELL_POWER_DB: phf::Map<u32, &'static [SpellPowerCost]> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok(())
}

fn write_spell_power_lookup_fns(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub fn get_spell_power(id: u32) -> Option<&'static [SpellPowerCost]> {{")?;
    writeln!(out, "    SPELL_POWER_DB.get(&id).copied()")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "pub fn power_type_name(power_type: i8) -> &'static str {{")?;
    writeln!(out, "    match power_type {{")?;
    for (val, name) in [
        ("-2", "HEALTH"), ("0", "MANA"), ("1", "RAGE"), ("2", "FOCUS"),
        ("3", "ENERGY"), ("4", "COMBO_POINTS"), ("5", "RUNES"), ("6", "RUNIC_POWER"),
        ("7", "SOUL_SHARDS"), ("8", "LUNAR_POWER"), ("9", "HOLY_POWER"),
        ("10", "ALTERNATE_POWER"), ("11", "MAELSTROM"), ("12", "CHI"),
        ("13", "INSANITY"), ("17", "FURY"), ("19", "ESSENCE"),
    ] {
        writeln!(out, "        {val} => \"{name}\",")?;
    }
    writeln!(out, "        _ => \"MANA\",")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")
}

fn write_spell_power_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_flash_of_light_power_cost() {{")?;
    writeln!(out, "        let costs = get_spell_power(19750).expect(\"Flash of Light should have power cost\");")?;
    writeln!(out, "        assert!(!costs.is_empty());")?;
    writeln!(out, "        assert_eq!(costs[0].power_type, 0); // MANA")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_spell_power_count() {{")?;
    writeln!(out, "        assert!(SPELL_POWER_DB.len() > 5000);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_no_power_for_unknown() {{")?;
    writeln!(out, "        assert!(get_spell_power(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")
}

/// Load ImplicitTarget_0 for each spell's first effect (EffectIndex=0, DifficultyID=0).
///
/// Columns: DifficultyID[2], EffectIndex[3], ImplicitTarget_0[34], SpellID[36]
fn load_spell_effect_targets(
    path: &Path,
) -> Result<HashMap<u32, u8>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 37 {
            continue;
        }
        let difficulty_id: u32 = f[2].parse().unwrap_or(1);
        if difficulty_id != 0 {
            continue;
        }
        let effect_index: u32 = f[3].parse().unwrap_or(999);
        if effect_index != 0 {
            continue;
        }
        let spell_id: u32 = match f[36].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let target: u8 = f[34].parse().unwrap_or(0);
        map.entry(spell_id).or_insert(target);
    }
    Ok(map)
}

fn load_spell_misc(
    path: &Path,
) -> Result<HashMap<u32, (u32, u32)>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() < 34 {
            continue;
        }
        let difficulty_id: u32 = fields[18].parse().unwrap_or(1);
        if difficulty_id != 0 {
            continue;
        }
        let spell_id: u32 = match fields[33].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let icon: u32 = fields[27].parse().unwrap_or(136243);
        let school: u32 = fields[23].parse().unwrap_or(0);
        map.entry(spell_id).or_insert((icon, school));
    }
    Ok(map)
}
