//! Generator for items.rs from WoW CSV exports.
//!
//! Run with: cargo run --bin gen_items
//!
//! Reads from ~/Projects/wow/data/:
//!   - ItemSparse.csv
//!
//! Column indices (0-based):
//!   0: ID, 6: Display_lang (name), 7: ExpansionID, 46: Stackable,
//!   50: SellPrice, 83: ItemLevel, 94: Bonding,
//!   99: RequiredLevel, 100: InventoryType, 101: OverallQualityID
//!
//! Generates: data/items.rs
//!
//! Uses phf_codegen (not phf_map! macro) because 171K entries would be
//! extremely slow to compile via macro expansion.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = dirs::home_dir()
        .expect("No home dir")
        .join("Projects/wow/data");

    let csv_path = wow_data.join("ItemSparse.csv");
    println!("Loading ItemSparse from {}...", csv_path.display());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/items.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;

    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;
    let mut skipped = 0u32;

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }

        let fields = parse_csv_line(&line);
        if fields.len() < 102 {
            skipped += 1;
            continue;
        }

        let id: u32 = match fields[0].parse() {
            Ok(v) => v,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let name = &fields[6];
        if name.is_empty() {
            skipped += 1;
            continue;
        }

        let escaped_name = escape_str(name);
        let expansion_id: u8 = fields[7].parse().unwrap_or(0);
        let stackable: u32 = fields[46].parse().unwrap_or(1);
        let sell_price: u32 = fields[50].parse().unwrap_or(0);
        let item_level: u16 = fields[83].parse().unwrap_or(0);
        let bonding: u8 = fields[94].parse().unwrap_or(0);
        let required_level: u16 = fields[99].parse().unwrap_or(0);
        let inventory_type: u8 = fields[100].parse().unwrap_or(0);
        let quality: u8 = fields[101].parse().unwrap_or(0);

        let value = format!(
            "ItemInfo {{ name: \"{}\", quality: {}, item_level: {}, required_level: {}, \
             inventory_type: {}, sell_price: {}, stackable: {}, bonding: {}, expansion_id: {} }}",
            escaped_name, quality, item_level, required_level, inventory_type, sell_price,
            stackable, bonding, expansion_id
        );
        builder.entry(id, &value);
        count += 1;
    }

    writeln!(
        out,
        "pub static ITEM_DB: phf::Map<u32, ItemInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;

    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!(
        "Generated {} item entries ({} skipped)",
        count, skipped
    );
    println!("Output: {}", output_path.display());
    Ok(())
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated item data from WoW CSV exports.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: cargo run --bin gen_items"
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
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub fn get_item(id: u32) -> Option<&'static ItemInfo> {{")?;
    writeln!(out, "    ITEM_DB.get(&id)")?;
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
    writeln!(out, "    fn test_item_count() {{")?;
    writeln!(out, "        assert!(ITEM_DB.len() > 100_000);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_hearthstone() {{")?;
    writeln!(
        out,
        "        let item = get_item(6948).expect(\"Hearthstone (6948) should exist\");"
    )?;
    writeln!(out, "        assert_eq!(item.name, \"Hearthstone\");")?;
    writeln!(out, "        assert_eq!(item.quality, 1);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_item() {{")?;
    writeln!(out, "        assert!(get_item(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}

/// Escape a string for use inside a Rust string literal.
fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Parse a CSV line, handling quoted fields properly.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.clone());
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current);
    fields
}
