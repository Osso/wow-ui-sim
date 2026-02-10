//! Generator for atlas_data.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - UiTextureAtlas.csv
//!   - UiTextureAtlasElement.csv
//!   - UiTextureAtlasMember.csv
//!   - listfile.csv
//!
//! Generates: data/atlas.rs

use super::csv_util::{parse_csv_line, wow_data_dir};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();

    println!("Loading listfile...");
    let listfile = load_listfile(&wow_data.join("listfile.csv"))?;
    println!("  {} entries", listfile.len());

    println!("Loading UiTextureAtlas...");
    let atlases = load_atlas(&wow_data.join("UiTextureAtlas.csv"))?;
    println!("  {} entries", atlases.len());

    println!("Loading UiTextureAtlasElement...");
    let _elements = load_elements(&wow_data.join("UiTextureAtlasElement.csv"))?;
    println!("  {} entries", _elements.len());

    println!("Loading UiTextureAtlasMember...");
    let members = load_members(&wow_data.join("UiTextureAtlasMember.csv"))?;
    println!("  {} entries", members.len());

    println!("Generating atlas_data.rs...");
    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/atlas.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    write_lookup_fn(&mut out)?;
    let (count, skipped) = write_atlas_entries(&mut out, &members, &atlases, &listfile)?;

    println!("Generated {} atlas entries ({} skipped)", count, skipped);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated atlas data from WoW CSV exports.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: wow-cli generate atlas")?;
    writeln!(out)?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out)?;
    write_atlas_structs(out)?;
    write_atlas_lookup_struct(out)?;
    Ok(())
}

fn write_atlas_structs(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct AtlasInfo {{")?;
    writeln!(out, "    pub file: &'static str,")?;
    writeln!(out, "    pub width: u32,")?;
    writeln!(out, "    pub height: u32,")?;
    writeln!(out, "    pub left_tex_coord: f32,")?;
    writeln!(out, "    pub right_tex_coord: f32,")?;
    writeln!(out, "    pub top_tex_coord: f32,")?;
    writeln!(out, "    pub bottom_tex_coord: f32,")?;
    writeln!(out, "    pub tiles_horizontally: bool,")?;
    writeln!(out, "    pub tiles_vertically: bool,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_atlas_lookup_struct(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub struct AtlasLookup {{")?;
    writeln!(out, "    pub info: &'static AtlasInfo,")?;
    writeln!(out, "    pub is_2x_fallback: bool,")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "impl AtlasLookup {{")?;
    writeln!(out, "    pub fn width(&self) -> u32 {{")?;
    writeln!(out, "        if self.is_2x_fallback {{ self.info.width / 2 }} else {{ self.info.width }}")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn height(&self) -> u32 {{")?;
    writeln!(out, "        if self.is_2x_fallback {{ self.info.height / 2 }} else {{ self.info.height }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub fn get_atlas_info(name: &str) -> Option<AtlasLookup> {{")?;
    writeln!(out, "    let lower = name.to_lowercase();")?;
    writeln!(out)?;
    writeln!(out, "    if let Some(info) = ATLAS_DB.get(&lower as &str) {{")?;
    writeln!(out, "        return Some(AtlasLookup {{ info, is_2x_fallback: false }});")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    if !lower.ends_with(\"-2x\") {{")?;
    writeln!(out, "        let with_2x = format!(\"{{lower}}-2x\");")?;
    writeln!(out, "        if let Some(info) = ATLAS_DB.get(&with_2x as &str) {{")?;
    writeln!(out, "            return Some(AtlasLookup {{ info, is_2x_fallback: true }});")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    if let Some(base) = lower.strip_suffix(\"-2x\") {{")?;
    writeln!(out, "        if let Some(info) = ATLAS_DB.get(base) {{")?;
    writeln!(out, "            return Some(AtlasLookup {{ info, is_2x_fallback: false }});")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    None")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "pub static ATLAS_DB: phf::Map<&'static str, AtlasInfo> = phf_map! {{")?;
    Ok(())
}

fn write_atlas_entries(
    out: &mut File,
    members: &[MemberEntry],
    atlases: &HashMap<u32, AtlasEntry>,
    listfile: &HashMap<u32, String>,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut count = 0u32;
    let mut skipped = 0u32;
    let mut seen_keys = std::collections::HashSet::new();

    for member in members {
        match format_atlas_entry(member, atlases, listfile, &mut seen_keys) {
            Some(line) => {
                writeln!(out, "    {line}")?;
                count += 1;
            }
            None => {
                skipped += 1;
            }
        }
    }

    writeln!(out, "}};")?;
    Ok((count, skipped))
}

fn format_atlas_entry(
    member: &MemberEntry,
    atlases: &HashMap<u32, AtlasEntry>,
    listfile: &HashMap<u32, String>,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Option<String> {
    let atlas = atlases.get(&member.atlas_id)?;
    let file_path = listfile.get(&atlas.file_data_id)?;

    let wow_path = normalize_atlas_path(file_path);
    let (left, right, top, bottom) = compute_tex_coords(member, atlas);
    let tiles_h = (member.flags & 0x4) != 0;
    let tiles_v = (member.flags & 0x2) != 0;

    let name_lower = member.name.to_lowercase()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    if !seen_keys.insert(name_lower.clone()) {
        return None;
    }

    // Use OverrideWidth/OverrideHeight for display size when non-zero,
    // otherwise fall back to the raw atlas pixel dimensions.
    let display_w = if member.override_width > 0 { member.override_width } else { member.width };
    let display_h = if member.override_height > 0 { member.override_height } else { member.height };

    Some(format!(
        "\"{}\" => AtlasInfo {{ file: r\"{}\", width: {}, height: {}, \
         left_tex_coord: {:.6}, right_tex_coord: {:.6}, \
         top_tex_coord: {:.6}, bottom_tex_coord: {:.6}, \
         tiles_horizontally: {}, tiles_vertically: {} }},",
        name_lower, wow_path, display_w, display_h,
        left, right, top, bottom, tiles_h, tiles_v
    ))
}

fn normalize_atlas_path(file_path: &str) -> String {
    let wow_path = file_path
        .trim_end_matches(".blp")
        .trim_end_matches(".BLP")
        .replace('/', "\\");
    if let Some(rest) = wow_path.strip_prefix("interface") {
        format!("Interface{rest}")
    } else {
        wow_path
    }
}

fn compute_tex_coords(member: &MemberEntry, atlas: &AtlasEntry) -> (f32, f32, f32, f32) {
    let left = member.left as f32 / atlas.width as f32;
    let right = member.right as f32 / atlas.width as f32;
    let top = member.top as f32 / atlas.height as f32;
    let bottom = member.bottom as f32 / atlas.height as f32;
    (left, right, top, bottom)
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
    override_width: u32,
    override_height: u32,
    flags: u32,
}

fn load_listfile(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((id_str, path)) = line.split_once(';')
            && let Ok(id) = id_str.parse::<u32>() {
                map.insert(id, path.to_string());
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
        if i == 0 { continue; }

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
        if i == 0 { continue; }

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
        if i == 0 { continue; }

        let fields = parse_csv_line(&line);
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
                override_width: fields[10].parse().unwrap_or(0),
                override_height: fields[11].parse().unwrap_or(0),
                flags: fields[12].parse().unwrap_or(0),
            });
        }
    }
    Ok(entries)
}
