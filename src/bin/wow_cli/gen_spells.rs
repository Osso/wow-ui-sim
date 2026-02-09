//! Generator for spells.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - SpellName.csv (ID, Name_lang)
//!   - Spell.csv (ID, NameSubtext_lang, ...)
//!   - SpellMisc.csv (ID, ..., DifficultyID[18], SchoolMask[23],
//!     SpellIconFileDataID[27], SpellID[33])
//!
//! Generates: data/spells.rs

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

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/spells.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    let count = build_spell_map(&mut out, &spell_names, &spell_subtexts, &spell_misc)?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!("Generated {} spell entries", count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn build_spell_map(
    out: &mut File,
    spell_names: &HashMap<u32, String>,
    spell_subtexts: &HashMap<u32, String>,
    spell_misc: &HashMap<u32, (u32, u32)>,
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

        let value = format!(
            "SpellInfo {{ name: \"{}\", subtext: \"{}\", icon_file_data_id: {}, school_mask: {} }}",
            escaped_name, subtext, icon, school
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
        if fields.len() >= 2 {
            if let Ok(id) = fields[0].parse::<u32>() {
                map.insert(id, fields[1].clone());
            }
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
        if fields.len() >= 2 {
            if let Ok(id) = fields[0].parse::<u32>() {
                let subtext = &fields[1];
                if !subtext.is_empty() {
                    map.insert(id, subtext.clone());
                }
            }
        }
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
