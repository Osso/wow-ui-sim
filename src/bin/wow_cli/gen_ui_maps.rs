//! Generator for ui_maps.rs from the WoW UiMap CSV export.
//!
//! Reads from ~/Projects/wow/data/:
//!   - UiMap.csv (Name_lang, ID, ParentUiMapID, Flags, System, Type, ...)
//!
//! Generates: data/ui_maps.rs
//!
//! `C_Map.GetMapInfo` answers from this table for every uiMapID the hand
//! seeded `SimState.maps` does not carry, so an addon that names the
//! destination of a teleport by its map gets "Dornogal", not "Map 2339".

use super::csv_util::{escape_str, parse_csv_line, wow_data_dir};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let csv_path = wow_data.join("UiMap.csv");
    println!("Loading UiMap from {}...", csv_path.display());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/ui_maps.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    let (count, skipped) = build_ui_map_table(&mut out, reader)?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!("Generated {} ui map entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());
    Ok(())
}

struct Columns {
    name: usize,
    id: usize,
    parent: usize,
    flags: usize,
    map_type: usize,
}

fn find_columns(header: &str) -> Result<Columns, Box<dyn std::error::Error>> {
    let fields = parse_csv_line(header);
    let index = |name: &str| {
        fields
            .iter()
            .position(|f| f == name)
            .ok_or_else(|| format!("UiMap.csv has no {name} column"))
    };
    Ok(Columns {
        name: index("Name_lang")?,
        id: index("ID")?,
        parent: index("ParentUiMapID")?,
        flags: index("Flags")?,
        map_type: index("Type")?,
    })
}

fn build_ui_map_table(
    out: &mut File,
    reader: BufReader<File>,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;
    let mut skipped = 0u32;
    let mut columns: Option<Columns> = None;

    for line in reader.lines() {
        let line = line?;
        let Some(cols) = columns.as_ref() else {
            columns = Some(find_columns(&line)?);
            continue;
        };
        match parse_ui_map_row(cols, &line) {
            Some((id, value)) => {
                builder.entry(id, &value);
                count += 1;
            }
            None => {
                skipped += 1;
            }
        }
    }

    writeln!(
        out,
        "pub static UI_MAP_DB: phf::Map<u32, UiMapInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok((count, skipped))
}

fn parse_ui_map_row(cols: &Columns, line: &str) -> Option<(u32, String)> {
    let fields = parse_csv_line(line);
    let needed = [cols.name, cols.id, cols.parent, cols.flags, cols.map_type]
        .into_iter()
        .max()?;
    if fields.len() <= needed {
        return None;
    }
    let id: u32 = fields[cols.id].parse().ok()?;
    let name = &fields[cols.name];
    if name.is_empty() {
        return None;
    }
    let parent_map_id: u32 = fields[cols.parent].parse().unwrap_or(0);
    let flags: u32 = fields[cols.flags].parse().unwrap_or(0);
    let map_type: u32 = fields[cols.map_type].parse().unwrap_or(0);

    let value = format!(
        "UiMapInfo {{ name: \"{}\", map_type: {}, parent_map_id: {}, flags: {} }}",
        escape_str(name),
        map_type,
        parent_map_id,
        flags
    );
    Some((id, value))
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated UiMap data from the WoW UiMap CSV.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate ui-maps"
    )?;
    writeln!(out)?;
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct UiMapInfo {{")?;
    writeln!(out, "    pub name: &'static str,")?;
    writeln!(out, "    /// `Enum.UIMapType`: 0 cosmic, 1 world, 2 continent, 3 zone, 4 dungeon, 5 micro, 6 orphan")?;
    writeln!(out, "    pub map_type: u32,")?;
    writeln!(out, "    pub parent_map_id: u32,")?;
    writeln!(out, "    pub flags: u32,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_ui_map(id: u32) -> Option<&'static UiMapInfo> {{"
    )?;
    writeln!(out, "    UI_MAP_DB.get(&id)")?;
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
    writeln!(out, "    fn test_ui_map_count() {{")?;
    writeln!(out, "        assert!(UI_MAP_DB.len() > 1500);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_dornogal_is_a_zone_of_the_isle_of_dorn() {{")?;
    writeln!(
        out,
        "        let map = get_ui_map(2339).expect(\"Dornogal (2339) should exist\");"
    )?;
    writeln!(out, "        assert_eq!(map.name, \"Dornogal\");")?;
    writeln!(out, "        assert_eq!(map.map_type, 3);")?;
    writeln!(
        out,
        "        assert_eq!(map.parent_map_id, 2248, \"Isle of Dorn, not Khaz Algar 2274\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_cosmic_and_azeroth() {{")?;
    writeln!(out, "        assert_eq!(get_ui_map(946).map(|m| m.name), Some(\"Cosmic\"));")?;
    writeln!(out, "        assert_eq!(get_ui_map(947).map(|m| m.name), Some(\"Azeroth\"));")?;
    writeln!(out, "        assert_eq!(get_ui_map(947).map(|m| m.parent_map_id), Some(946));")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_ui_map() {{")?;
    writeln!(out, "        assert!(get_ui_map(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}
