//! Generator for atlas_data.rs from WoW CSV exports.
//!
//! Run with: cargo run --bin gen_atlas
//!
//! Reads from ~/Projects/wow/data/:
//!   - UiTextureAtlas.csv
//!   - UiTextureAtlasElement.csv
//!   - UiTextureAtlasMember.csv
//!   - listfile.csv
//!
//! Generates: data/atlas.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = dirs::home_dir()
        .expect("No home dir")
        .join("Projects/wow/data");

    // Load listfile: FileDataID -> path
    println!("Loading listfile...");
    let listfile = load_listfile(&wow_data.join("listfile.csv"))?;
    println!("  {} entries", listfile.len());

    // Load UiTextureAtlas: ID -> (FileDataID, Width, Height)
    println!("Loading UiTextureAtlas...");
    let atlases = load_atlas(&wow_data.join("UiTextureAtlas.csv"))?;
    println!("  {} entries", atlases.len());

    // Load UiTextureAtlasElement: Name -> ElementID
    println!("Loading UiTextureAtlasElement...");
    let elements = load_elements(&wow_data.join("UiTextureAtlasElement.csv"))?;
    println!("  {} entries", elements.len());

    // Load UiTextureAtlasMember: joins element to atlas with coords
    println!("Loading UiTextureAtlasMember...");
    let members = load_members(&wow_data.join("UiTextureAtlasMember.csv"))?;
    println!("  {} entries", members.len());

    // Join and generate
    println!("Generating atlas_data.rs...");
    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/atlas.rs");
    let mut out = File::create(output_path)?;

    writeln!(out, "//! Auto-generated atlas data from WoW CSV exports.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: cargo run --bin gen_atlas")?;
    writeln!(out, "")?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out, "")?;
    writeln!(out, "/// Information about a texture atlas region.")?;
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct AtlasInfo {{")?;
    writeln!(out, "    /// The texture file path (WoW-style path).")?;
    writeln!(out, "    pub file: &'static str,")?;
    writeln!(out, "    /// Width of the atlas region in pixels.")?;
    writeln!(out, "    pub width: u32,")?;
    writeln!(out, "    /// Height of the atlas region in pixels.")?;
    writeln!(out, "    pub height: u32,")?;
    writeln!(out, "    /// Left texture coordinate (0.0-1.0).")?;
    writeln!(out, "    pub left_tex_coord: f32,")?;
    writeln!(out, "    /// Right texture coordinate (0.0-1.0).")?;
    writeln!(out, "    pub right_tex_coord: f32,")?;
    writeln!(out, "    /// Top texture coordinate (0.0-1.0).")?;
    writeln!(out, "    pub top_tex_coord: f32,")?;
    writeln!(out, "    /// Bottom texture coordinate (0.0-1.0).")?;
    writeln!(out, "    pub bottom_tex_coord: f32,")?;
    writeln!(out, "    /// Whether this atlas tiles horizontally.")?;
    writeln!(out, "    pub tiles_horizontally: bool,")?;
    writeln!(out, "    /// Whether this atlas tiles vertically.")?;
    writeln!(out, "    pub tiles_vertically: bool,")?;
    writeln!(out, "}}")?;
    writeln!(out, "")?;
    writeln!(out, "/// Result of an atlas lookup, including whether a -2x fallback was used.")?;
    writeln!(out, "pub struct AtlasLookup {{")?;
    writeln!(out, "    pub info: &'static AtlasInfo,")?;
    writeln!(out, "    /// True when the caller requested a non-2x name but we resolved to a -2x entry.")?;
    writeln!(out, "    /// Width/height should be halved for logical (1x) dimensions.")?;
    writeln!(out, "    pub is_2x_fallback: bool,")?;
    writeln!(out, "}}")?;
    writeln!(out, "")?;
    writeln!(out, "impl AtlasLookup {{")?;
    writeln!(out, "    /// Logical width (halved when -2x fallback was used).")?;
    writeln!(out, "    pub fn width(&self) -> u32 {{")?;
    writeln!(out, "        if self.is_2x_fallback {{ self.info.width / 2 }} else {{ self.info.width }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    /// Logical height (halved when -2x fallback was used).")?;
    writeln!(out, "    pub fn height(&self) -> u32 {{")?;
    writeln!(out, "        if self.is_2x_fallback {{ self.info.height / 2 }} else {{ self.info.height }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out, "")?;
    writeln!(out, "/// Get atlas info by name (case-insensitive).")?;
    writeln!(out, "/// Falls back to trying with/without -2x suffix for hi-res variants.")?;
    writeln!(out, "pub fn get_atlas_info(name: &str) -> Option<AtlasLookup> {{")?;
    writeln!(out, "    let lower = name.to_lowercase();")?;
    writeln!(out, "")?;
    writeln!(out, "    // Try exact match first")?;
    writeln!(out, "    if let Some(info) = ATLAS_DB.get(&lower as &str) {{")?;
    writeln!(out, "        return Some(AtlasLookup {{ info, is_2x_fallback: false }});")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    // Try with -2x suffix if not present")?;
    writeln!(out, "    if !lower.ends_with(\"-2x\") {{")?;
    writeln!(out, "        let with_2x = format!(\"{{lower}}-2x\");")?;
    writeln!(out, "        if let Some(info) = ATLAS_DB.get(&with_2x as &str) {{")?;
    writeln!(out, "            return Some(AtlasLookup {{ info, is_2x_fallback: true }});")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    // Try without -2x suffix if present")?;
    writeln!(out, "    if let Some(base) = lower.strip_suffix(\"-2x\") {{")?;
    writeln!(out, "        if let Some(info) = ATLAS_DB.get(base) {{")?;
    writeln!(out, "            return Some(AtlasLookup {{ info, is_2x_fallback: false }});")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    None")?;
    writeln!(out, "}}")?;
    writeln!(out, "")?;
    writeln!(out, "/// Atlas database with all known atlas definitions (compile-time perfect hash map).")?;
    writeln!(out, "pub static ATLAS_DB: phf::Map<&'static str, AtlasInfo> = phf_map! {{")?;

    let mut count = 0;
    let mut skipped = 0;
    let mut seen_keys = std::collections::HashSet::new();

    for member in &members {
        // Look up atlas info
        let Some(atlas) = atlases.get(&member.atlas_id) else {
            skipped += 1;
            continue;
        };

        // Look up file path
        let Some(file_path) = listfile.get(&atlas.file_data_id) else {
            skipped += 1;
            continue;
        };

        // Convert path to WoW-style (backslashes, no extension, capitalize Interface)
        let wow_path = file_path
            .trim_end_matches(".blp")
            .trim_end_matches(".BLP")
            .replace('/', "\\");
        let wow_path = if wow_path.starts_with("interface") {
            format!("Interface{}", &wow_path[9..])
        } else {
            wow_path
        };

        // Calculate texture coordinates
        let left = member.left as f32 / atlas.width as f32;
        let right = member.right as f32 / atlas.width as f32;
        let top = member.top as f32 / atlas.height as f32;
        let bottom = member.bottom as f32 / atlas.height as f32;

        // Flags: 0x2 = tiles vertically, 0x4 = tiles horizontally
        let tiles_h = (member.flags & 0x4) != 0;
        let tiles_v = (member.flags & 0x2) != 0;

        // Use lowercase name as key for case-insensitive lookup
        // Escape for Rust string literal
        let name_lower = member.name.to_lowercase()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        // Skip duplicates (keep first occurrence)
        if !seen_keys.insert(name_lower.clone()) {
            skipped += 1;
            continue;
        }

        writeln!(out, "    \"{}\" => AtlasInfo {{ file: r\"{}\", width: {}, height: {}, left_tex_coord: {:.6}, right_tex_coord: {:.6}, top_tex_coord: {:.6}, bottom_tex_coord: {:.6}, tiles_horizontally: {}, tiles_vertically: {} }},",
            name_lower, wow_path, member.width, member.height, left, right, top, bottom, tiles_h, tiles_v)?;

        count += 1;
    }

    writeln!(out, "}};")?;

    println!("Generated {} atlas entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());

    Ok(())
}

struct AtlasEntry {
    file_data_id: u32,
    width: u32,
    height: u32,
}

struct MemberEntry {
    name: String,
    atlas_id: u32,
    width: u32,
    height: u32,
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
    flags: u32,
}

fn load_listfile(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((id_str, path)) = line.split_once(';') {
            if let Ok(id) = id_str.parse::<u32>() {
                map.insert(id, path.to_string());
            }
        }
    }

    Ok(map)
}

fn load_atlas(path: &Path) -> Result<HashMap<u32, AtlasEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; } // Skip header

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 4 {
            let id: u32 = fields[0].parse()?;
            let file_data_id: u32 = fields[1].parse()?;
            let width: u32 = fields[2].parse()?;
            let height: u32 = fields[3].parse()?;

            map.insert(id, AtlasEntry { file_data_id, width, height });
        }
    }

    Ok(map)
}

fn load_elements(path: &Path) -> Result<HashMap<String, u32>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; } // Skip header

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 2 {
            let name = fields[0].to_string();
            let id: u32 = fields[1].parse()?;
            map.insert(name, id);
        }
    }

    Ok(map)
}

fn load_members(path: &Path) -> Result<Vec<MemberEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; } // Skip header

        // Parse CSV properly handling quoted fields
        let fields = parse_csv_line(&line);
        // CommittedName,ID,UiTextureAtlasID,Width,Height,CommittedLeft,CommittedRight,CommittedTop,CommittedBottom,UiTextureAtlasElementID,OverrideWidth,OverrideHeight,CommittedFlags
        if fields.len() >= 13 {
            entries.push(MemberEntry {
                name: fields[0].clone(),
                atlas_id: fields[2].parse()?,
                width: fields[3].parse()?,
                height: fields[4].parse()?,
                left: fields[5].parse()?,
                right: fields[6].parse()?,
                top: fields[7].parse()?,
                bottom: fields[8].parse()?,
                flags: fields[12].parse().unwrap_or(0),
            });
        }
    }

    Ok(entries)
}

/// Parse a CSV line, handling quoted fields properly
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
                // Check for escaped quote ""
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}
