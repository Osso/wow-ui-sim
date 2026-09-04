//! Generator for spells.rs, spell_descriptions.rs, and spell_power.rs from WoW CSV exports.
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
//! Generates: data/spells.rs, data/spell_descriptions.rs, data/spell_power.rs

use super::csv_util::{escape_str, parse_csv_line, wow_data_dir};
use super::gen_spells_power::{SpellPowerRow, load_spell_power, write_spell_power};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const SPELL_NAME_COLUMN: usize = 1;
const SPELL_SUBTEXT_COLUMN: usize = 1;
const SPELL_DESCRIPTION_COLUMN: usize = 2;

#[derive(Clone, Copy)]
enum EmptySpellText {
    Keep,
    Skip,
}

const SPELL_LOOKUP_TESTS: &[&str] = &[
    "",
    "#[cfg(test)]",
    "mod tests {",
    "    use super::*;",
    "",
    "    #[test]",
    "    fn test_spell_count() {",
    "        assert!(SPELL_DB.len() > 100);",
    "    }",
    "",
    "    #[test]",
    "    fn test_frostbolt() {",
    "        let spell = get_spell(116).expect(\"spell 116 should exist\");",
    "        assert_eq!(spell.name, \"Frostbolt\");",
    "    }",
    "",
    "    #[test]",
    "    fn test_nonexistent_spell() {",
    "        assert!(get_spell(999_999_999).is_none());",
    "    }",
    "}",
];

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let spell_data = load_spell_data(&wow_data)?;
    std::fs::create_dir_all("data")?;
    let required_ids = collect_required_spell_ids()?;
    generate_spell_table(&spell_data, &required_ids)?;
    generate_spell_descriptions(&spell_data, &required_ids)?;
    generate_spell_power_table(&spell_data, &required_ids)?;
    Ok(())
}

struct SpellData {
    spell_names: HashMap<u32, String>,
    spell_subtexts: HashMap<u32, String>,
    spell_descriptions: HashMap<u32, String>,
    spell_misc: HashMap<u32, (u32, u32)>,
    spell_power: HashMap<u32, Vec<SpellPowerRow>>,
    spell_targets: HashMap<u32, u8>,
}

fn load_spell_data(wow_data: &Path) -> Result<SpellData, Box<dyn std::error::Error>> {
    let spell_names = load_spell_names(&wow_data.join("SpellName.csv"))?;
    println!("SpellName: {} entries", spell_names.len());

    let spell_subtexts = load_spell_subtexts(&wow_data.join("Spell.csv"))?;
    println!("Spell (subtexts): {} entries", spell_subtexts.len());

    let spell_descriptions = load_spell_descriptions(&wow_data.join("Spell.csv"))?;
    println!("Spell (descriptions): {} entries", spell_descriptions.len());

    let spell_misc = load_spell_misc(&wow_data.join("SpellMisc.csv"))?;
    println!("SpellMisc (DifficultyID=0): {} entries", spell_misc.len());

    let spell_power = load_spell_power(&wow_data.join("SpellPower.csv"))?;
    println!("SpellPower: {} spells with costs", spell_power.len());

    let spell_targets = load_spell_effect_targets(&wow_data.join("SpellEffect.csv"))?;
    println!("SpellEffect (targets): {} spells", spell_targets.len());

    Ok(SpellData {
        spell_names,
        spell_subtexts,
        spell_descriptions,
        spell_misc,
        spell_power,
        spell_targets,
    })
}

fn generate_spell_table(
    spell_data: &SpellData,
    required_ids: &BTreeSet<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new("data/spells.rs");
    let mut out = File::create(output_path)?;
    write_header(&mut out)?;
    let count = build_spell_map(
        &mut out,
        &spell_data.spell_names,
        &spell_data.spell_subtexts,
        &spell_data.spell_misc,
        &spell_data.spell_targets,
        required_ids,
    )?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;
    println!("Generated {} spell entries", count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn generate_spell_descriptions(
    spell_data: &SpellData,
    required_ids: &BTreeSet<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let descriptions_path = Path::new("data/spell_descriptions.rs");
    let mut desc_out = File::create(descriptions_path)?;
    let description_count =
        write_spell_descriptions(&mut desc_out, &spell_data.spell_descriptions, required_ids)?;
    println!("Generated {} compact spell descriptions", description_count);
    println!("Output: {}", descriptions_path.display());
    Ok(())
}

fn generate_spell_power_table(
    spell_data: &SpellData,
    required_ids: &BTreeSet<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let power_path = Path::new("data/spell_power.rs");
    let mut power_out = File::create(power_path)?;
    // Only include power data for required spells
    let filtered_power: HashMap<u32, Vec<SpellPowerRow>> = spell_data
        .spell_power
        .iter()
        .filter(|(id, _)| required_ids.contains(id))
        .map(|(id, rows)| (*id, rows.clone()))
        .collect();
    let power_count = write_spell_power(&mut power_out, &filtered_power)?;
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
    required_ids: &BTreeSet<u32>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;

    for &spell_id in required_ids {
        let Some(name) = spell_names.get(&spell_id) else {
            continue;
        };
        let escaped_name = escape_str(name);
        let subtext = spell_subtexts
            .get(&spell_id)
            .map(|s| escape_str(s))
            .unwrap_or_default();
        let (icon, school) = spell_misc.get(&spell_id).copied().unwrap_or((136243, 0));
        let implicit_target = spell_targets.get(&spell_id).copied().unwrap_or(0);

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
    writeln!(
        out,
        "    /// ImplicitTarget_0 from first SpellEffect (EffectIndex=0, DifficultyID=0)."
    )?;
    writeln!(
        out,
        "    /// Determines valid target type: 1=Self, 6=Enemy, 21=Ally, 25=Any, etc."
    )?;
    writeln!(out, "    pub implicit_target: u8,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_spell_descriptions(
    out: &mut File,
    spell_descriptions: &HashMap<u32, String>,
    include_ids: &BTreeSet<u32>,
) -> Result<u32, Box<dyn std::error::Error>> {
    write_spell_description_header(out)?;
    let (builder, count) = build_spell_description_map(spell_descriptions, include_ids);
    write_spell_description_lookup(out, builder)?;
    Ok(count)
}

fn write_spell_description_header(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "//! Auto-generated compact spell descriptions used by tooltip APIs."
    )?;
    writeln!(
        out,
        "//! Contains only spell IDs referenced by trait data and the minimal spell table."
    )?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate spells"
    )?;
    writeln!(out)?;
    Ok(())
}

fn build_spell_description_map(
    spell_descriptions: &HashMap<u32, String>,
    include_ids: &BTreeSet<u32>,
) -> (phf_codegen::Map<u32>, u32) {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;
    for spell_id in include_ids {
        let Some(description) = spell_descriptions.get(spell_id) else {
            continue;
        };
        builder.entry(*spell_id, &format!("\"{}\"", escape_str(description)));
        count += 1;
    }
    (builder, count)
}

fn write_spell_description_lookup(
    out: &mut File,
    builder: phf_codegen::Map<u32>,
) -> std::io::Result<()> {
    writeln!(
        out,
        "pub static SPELL_DESCRIPTIONS: phf::Map<u32, &'static str> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "pub fn get_spell_description(id: u32) -> Option<&'static str> {{"
    )?;
    writeln!(out, "    SPELL_DESCRIPTIONS.get(&id).copied()")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_spell(id: u32) -> Option<&'static SpellInfo> {{"
    )?;
    writeln!(out, "    SPELL_DB.get(&id)")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn write_tests(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, SPELL_LOOKUP_TESTS)?;
    Ok(())
}

fn write_literal_lines(out: &mut File, lines: &[&str]) -> std::io::Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

fn load_spell_names(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    load_spell_text_column(path, SPELL_NAME_COLUMN, EmptySpellText::Keep)
}

fn load_spell_subtexts(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    load_spell_text_column(path, SPELL_SUBTEXT_COLUMN, EmptySpellText::Skip)
}

fn load_spell_descriptions(
    path: &Path,
) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    load_spell_text_column(path, SPELL_DESCRIPTION_COLUMN, EmptySpellText::Skip)
}

fn load_spell_text_column(
    path: &Path,
    value_column: usize,
    empty_spell_text: EmptySpellText,
) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        let fields = parse_csv_line(&line);
        if fields.len() > value_column
            && let Ok(id) = fields[0].parse::<u32>()
        {
            let value = &fields[value_column];
            if matches!(empty_spell_text, EmptySpellText::Keep) || !value.is_empty() {
                map.insert(id, value.clone());
            }
        }
    }
    Ok(map)
}

/// Collect all spell IDs the simulator actually needs.
///
/// Sources:
/// - `data/traits.rs`: spell_id, overrides_spell_id, visible_spell_id, override_icon fields
/// - `src/lua_api/globals/spellbook_data.rs`: spell(N) and passive(N) calls
/// - Hardcoded baseline: action bar spells, trinket/item procs not covered elsewhere
/// - Addon compatibility: spell names that installed addons index directly during load
fn collect_required_spell_ids() -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let trait_ids = collect_ids_from_file(
        "data/traits.rs",
        &[
            "spell_id: ",
            "overrides_spell_id: ",
            "visible_spell_id: ",
            "override_icon: ",
        ],
    )?;
    let spellbook_ids = collect_ids_from_file(
        "src/lua_api/globals/spellbook_data.rs",
        &["spell(", "passive("],
    )?;
    let baseline_ids: BTreeSet<u32> = BASELINE_SPELL_IDS.iter().copied().collect();
    let addon_compat_ids: BTreeSet<u32> = ADDON_COMPAT_SPELL_IDS.iter().copied().collect();

    println!(
        "Required spell IDs: {} (traits: {}, spellbook: {}, baseline: {}, addon compat: {})",
        trait_ids.len() + spellbook_ids.len() + baseline_ids.len() + addon_compat_ids.len(),
        trait_ids.len(),
        spellbook_ids.len(),
        baseline_ids.len(),
        addon_compat_ids.len(),
    );

    let mut all = trait_ids;
    all.extend(&spellbook_ids);
    all.extend(&baseline_ids);
    all.extend(&addon_compat_ids);
    // Everything that teleports: the use spells of toys and teleport items,
    // and the teleport spells on class skill lines.
    all.extend(super::gen_teleport_selection::collect(&wow_data_dir()).spells);
    println!("Required spell IDs (deduplicated): {}", all.len());
    Ok(all)
}

const BASELINE_SPELL_IDS: &[u32] = &[
    100, 116, 2018, 2575, 2576, 2657, 395296, 1230084, 1232418, 1232421, 1234430, 1242031, 1247534,
    1272143, 1279510,
];

/// Retail addons sometimes build lookup tables keyed by localized spell names
/// during file load, so `C_Spell.GetSpellInfo` must know these IDs before any
/// character spellbook or trait data references them.
const ADDON_COMPAT_SPELL_IDS: &[u32] = &[
    // Cell/Defaults/Indicator_DefaultSpells.lua
    430, 1064, 73920, 108280, 52042, 197995, 114911, 382311, 207778, 114083, 377509, 322118, 170906,
    167152, 43182, 172786, 308433, 369162, 456574, 461063, 195181, 203819, 192081, 215479, 132403,
    132404, // Cell/Defaults/ClickCasting_DefaultSpells.lua
    61999, 20484, 50769, 212040, 361227, 361178, 115178, 212051, 391054, 7328, 212056, 2006,
    212036, 2008, 212048, 20707,
    // QuickRoute/Data/TeleportItems.lua: teleports the effect rule cannot
    // see -- a housing teleport off any class line, summons and scripted
    // returns (Cantrips, Mole Machine, Return to Camp) and the Death Gate,
    // which opens a portal object rather than teleporting the caster.
    1233637, 255661, 265225, 312372, 50977,
];

fn collect_ids_from_file(
    path: &str,
    markers: &[&str],
) -> Result<BTreeSet<u32>, Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(path)?;
    let mut ids = BTreeSet::new();
    for marker in markers {
        collect_number_literals_after(&src, marker, &mut ids);
    }
    Ok(ids)
}

fn collect_number_literals_after(src: &str, marker: &str, out: &mut BTreeSet<u32>) {
    let mut rest = src;
    while let Some(idx) = rest.find(marker) {
        rest = &rest[idx + marker.len()..];
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len == 0 {
            continue;
        }
        if let Ok(id) = rest[..digits_len].parse::<u32>()
            && id != 0
        {
            out.insert(id);
        }
        rest = &rest[digits_len..];
    }
}

/// Load ImplicitTarget_0 for each spell's first effect (EffectIndex=0, DifficultyID=0).
///
/// Columns: DifficultyID[2], EffectIndex[3], ImplicitTarget_0[34], SpellID[36]
fn load_spell_effect_targets(path: &Path) -> Result<HashMap<u32, u8>, Box<dyn std::error::Error>> {
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

fn load_spell_misc(path: &Path) -> Result<HashMap<u32, (u32, u32)>, Box<dyn std::error::Error>> {
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
