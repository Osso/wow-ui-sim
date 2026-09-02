//! Generator for atlas_data.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - UiTextureAtlas.csv
//!   - UiTextureAtlasElement.csv
//!   - UiTextureAtlasMember.csv
//!   - UiTextureAtlasElementSliceData.csv
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
    let atlas_data = load_atlas_data(&wow_data)?;
    let output = generate_output_files(&atlas_data)?;
    println!(
        "Generated {} atlas entries ({} skipped), {} element mappings",
        output.count, output.skipped, output.elem_count
    );
    println!("Output: {}", output.output_path.display());
    Ok(())
}

struct AtlasData {
    listfile: HashMap<u32, String>,
    atlases: HashMap<u32, AtlasEntry>,
    elements: HashMap<u32, String>,
    slices: HashMap<u32, SliceEntry>,
    members: Vec<MemberEntry>,
}

struct GeneratedOutput {
    count: u32,
    skipped: u32,
    elem_count: u32,
    output_path: &'static Path,
}

fn load_atlas_data(wow_data: &Path) -> Result<AtlasData, Box<dyn std::error::Error>> {
    println!("Loading listfile...");
    let listfile = load_listfile(&wow_data.join("listfile.csv"))?;
    println!("  {} entries", listfile.len());

    println!("Loading UiTextureAtlas...");
    let atlases = load_atlas(&wow_data.join("UiTextureAtlas.csv"))?;
    println!("  {} entries", atlases.len());

    println!("Loading UiTextureAtlasElement...");
    let elements = load_elements(&wow_data.join("UiTextureAtlasElement.csv"))?;
    println!("  {} entries", elements.len());

    println!("Loading UiTextureAtlasElementSliceData...");
    let slices = load_slices(&slice_data_path(wow_data), &elements)?;
    println!("  {} entries", slices.len());

    println!("Loading UiTextureAtlasMember...");
    let members = load_members(&wow_data.join("UiTextureAtlasMember.csv"))?;
    println!("  {} entries", members.len());

    Ok(AtlasData {
        listfile,
        atlases,
        elements,
        slices,
        members,
    })
}

fn slice_data_path(wow_data: &Path) -> std::path::PathBuf {
    let local = Path::new("data/db2/UiTextureAtlasElementSliceData.csv");
    if local.exists() {
        local.to_path_buf()
    } else {
        wow_data.join("UiTextureAtlasElementSliceData.csv")
    }
}

fn generate_output_files(
    atlas_data: &AtlasData,
) -> Result<GeneratedOutput, Box<dyn std::error::Error>> {
    println!("Generating atlas_data.rs...");
    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/atlas.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    write_lookup_fn(&mut out)?;
    let (count, skipped) = write_atlas_entries(
        &mut out,
        &atlas_data.members,
        &atlas_data.atlases,
        &atlas_data.listfile,
    )?;
    write_slice_lookup(&mut out, &atlas_data.slices)?;

    let elem_path = Path::new("data/atlas_elements.rs");
    let mut elem_out = File::create(elem_path)?;
    let elem_count = write_element_map(&mut elem_out, &atlas_data.elements)?;

    Ok(GeneratedOutput {
        count,
        skipped,
        elem_count,
        output_path,
    })
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated atlas data from WoW CSV exports.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate atlas"
    )?;
    writeln!(out)?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out)?;
    write_atlas_structs(out)?;
    write_atlas_lookup_struct(out)?;
    Ok(())
}

fn write_atlas_structs(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, ATLAS_INFO_STRUCT)?;
    write_literal_lines(out, ATLAS_SLICE_MODE_ENUM)?;
    write_literal_lines(out, ATLAS_SLICE_INFO_STRUCT)?;
    Ok(())
}

const ATLAS_INFO_STRUCT: &[&str] = &[
    "#[derive(Debug, Clone)]",
    "pub struct AtlasInfo {",
    "    pub file: &'static str,",
    "    pub width: u32,",
    "    pub height: u32,",
    "    pub left_tex_coord: f32,",
    "    pub right_tex_coord: f32,",
    "    pub top_tex_coord: f32,",
    "    pub bottom_tex_coord: f32,",
    "    pub tiles_horizontally: bool,",
    "    pub tiles_vertically: bool,",
    "    /// `width`/`height` come from the member's OverrideWidth/Height, i.e.",
    "    /// they are the logical (display) size already. A `-2x` entry with",
    "    /// this set must not be halved when it stands in for a missing 1x",
    "    /// sibling; without it the entry holds the pixel rect of the 2x art.",
    "    pub size_is_override: bool,",
    "}",
    "",
];

const ATLAS_SLICE_MODE_ENUM: &[&str] = &[
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
    "pub enum AtlasSliceMode {",
    "    Stretch,",
    "    Tile,",
    "}",
    "",
];

const ATLAS_SLICE_INFO_STRUCT: &[&str] = &[
    "#[derive(Debug, Clone, Copy)]",
    "pub struct AtlasSliceInfo {",
    "    pub left: u32,",
    "    pub top: u32,",
    "    pub right: u32,",
    "    pub bottom: u32,",
    "    pub mode: AtlasSliceMode,",
    "}",
    "",
];

const ATLAS_LOOKUP_STRUCT: &[&str] = &[
    "pub struct AtlasLookup {",
    "    pub info: &'static AtlasInfo,",
    "    pub is_2x_fallback: bool,",
    "    /// Exact logical size when the texels come from a 2x entry that has a",
    "    /// 1x sibling; the 2x art is not always exactly twice the 1x size.",
    "    pub logical_size: Option<(u32, u32)>,",
    "}",
    "",
    "impl AtlasLookup {",
    "    pub fn width(&self) -> u32 {",
    "        if let Some((w, _)) = self.logical_size {",
    "            w",
    "        } else if self.is_2x_fallback && !self.info.size_is_override {",
    "            self.info.width / 2",
    "        } else {",
    "            self.info.width",
    "        }",
    "    }",
    "",
    "    pub fn height(&self) -> u32 {",
    "        if let Some((_, h)) = self.logical_size {",
    "            h",
    "        } else if self.is_2x_fallback && !self.info.size_is_override {",
    "            self.info.height / 2",
    "        } else {",
    "            self.info.height",
    "        }",
    "    }",
    "}",
    "",
];

fn write_literal_lines(out: &mut File, lines: &[&str]) -> std::io::Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

fn write_atlas_lookup_struct(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, ATLAS_LOOKUP_STRUCT)
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    write_get_atlas_info_fn(out)?;
    write_get_atlas_slice_info_fn(out)?;
    write_atlas_db_header(out)?;
    Ok(())
}

fn write_get_atlas_info_fn(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, GET_ATLAS_INFO_FN)
}

const GET_ATLAS_INFO_FN: &[&str] = &[
    "pub fn get_atlas_info(name: &str) -> Option<AtlasLookup> {",
    "    let lower = name.to_lowercase();",
    "",
    "    if let Some(info) = ATLAS_DB.get(&lower as &str) {",
    "        return Some(AtlasLookup {",
    "            info,",
    "            is_2x_fallback: false,",
    "            logical_size: None,",
    "        });",
    "    }",
    "",
    "    if !lower.ends_with(\"-2x\") {",
    "        let with_2x = format!(\"{lower}-2x\");",
    "        if let Some(info) = ATLAS_DB.get(&with_2x as &str) {",
    "            return Some(AtlasLookup {",
    "                info,",
    "                is_2x_fallback: true,",
    "                logical_size: None,",
    "            });",
    "        }",
    "    }",
    "",
    "    if let Some(base) = lower.strip_suffix(\"-2x\") {",
    "        if let Some(info) = ATLAS_DB.get(base) {",
    "            return Some(AtlasLookup {",
    "                info,",
    "                is_2x_fallback: false,",
    "                logical_size: None,",
    "            });",
    "        }",
    "    }",
    "",
    "    None",
    "}",
    "",
];

const ATLAS_DB_HEADER: &[&str] =
    &["pub static ATLAS_DB: phf::Map<&'static str, AtlasInfo> = phf_map! {"];

const ATLAS_SLICE_DB_HEADER: &[&str] = &[
    "",
    "pub static ATLAS_SLICE_DB: phf::Map<&'static str, AtlasSliceInfo> = phf_map! {",
];

fn write_get_atlas_slice_info_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_atlas_slice_info(name: &str) -> Option<AtlasSliceInfo> {{"
    )?;
    writeln!(out, "    let lower = name.to_lowercase();")?;
    writeln!(out, "    ATLAS_SLICE_DB.get(&lower as &str).copied()")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

fn write_atlas_db_header(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, ATLAS_DB_HEADER)
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

fn write_slice_lookup(
    out: &mut File,
    slices: &HashMap<u32, SliceEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_literal_lines(out, ATLAS_SLICE_DB_HEADER)?;

    for slice in sorted_slices(slices) {
        write_slice_entry(out, slice)?;
    }

    writeln!(out, "}};")?;
    Ok(())
}

fn sorted_slices(slices: &HashMap<u32, SliceEntry>) -> Vec<&SliceEntry> {
    let mut sorted: Vec<_> = slices.iter().collect();
    sorted.sort_by_key(|(id, _)| *id);
    sorted.into_iter().map(|(_id, slice)| slice).collect()
}

fn write_slice_entry(out: &mut File, slice: &SliceEntry) -> std::io::Result<()> {
    let Some(mode) = atlas_slice_mode_name(slice.mode) else {
        return Ok(());
    };

    writeln!(
        out,
        "    \"{}\" => AtlasSliceInfo {{ left: {}u32, top: {}u32, right: {}u32, bottom: {}u32, mode: {} }},",
        slice.name, slice.left, slice.top, slice.right, slice.bottom, mode
    )
}

fn atlas_slice_mode_name(mode: u8) -> Option<&'static str> {
    match mode {
        0 => Some("AtlasSliceMode::Stretch"),
        1 => Some("AtlasSliceMode::Tile"),
        _ => None,
    }
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

    let name_lower = member
        .name
        .to_lowercase()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    if !seen_keys.insert(name_lower.clone()) {
        return None;
    }

    // Use OverrideWidth/OverrideHeight for display size when non-zero,
    // otherwise fall back to the raw atlas pixel dimensions.
    let display_w = if member.override_width > 0 {
        member.override_width
    } else {
        member.width
    };
    let display_h = if member.override_height > 0 {
        member.override_height
    } else {
        member.height
    };

    let size_is_override = member.override_width > 0 || member.override_height > 0;

    Some(format!(
        "\"{}\" => AtlasInfo {{ file: r\"{}\", width: {}, height: {}, \
         left_tex_coord: {:.6}, right_tex_coord: {:.6}, \
         top_tex_coord: {:.6}, bottom_tex_coord: {:.6}, \
         tiles_horizontally: {}, tiles_vertically: {}, size_is_override: {} }},",
        name_lower,
        wow_path,
        display_w,
        display_h,
        left,
        right,
        top,
        bottom,
        tiles_h,
        tiles_v,
        size_is_override
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

struct SliceEntry {
    name: String,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    mode: u8,
}

fn load_listfile(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((id_str, path)) = line.split_once(';')
            && let Ok(id) = id_str.parse::<u32>()
        {
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
        if i == 0 {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 4 {
            let id: u32 = fields[0].parse()?;
            let file_data_id: u32 = fields[1].parse()?;
            let width: u32 = fields[2].parse()?;
            let height: u32 = fields[3].parse()?;
            map.insert(
                id,
                AtlasEntry {
                    file_data_id,
                    width,
                    height,
                },
            );
        }
    }
    Ok(map)
}

fn load_elements(path: &Path) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 2 {
            let name = fields[0].to_string();
            let id: u32 = fields[1].parse()?;
            map.insert(id, name);
        }
    }
    Ok(map)
}

/// Write element ID → atlas name map (for numeric atlas lookups like iconElementID).
fn write_element_map(
    out: &mut File,
    elements: &HashMap<u32, String>,
) -> Result<u32, Box<dyn std::error::Error>> {
    write_element_map_header(out)?;
    let count = write_element_entries(out, elements)?;
    writeln!(out, "}};")?;
    Ok(count)
}

fn write_element_map_header(out: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "//! Auto-generated atlas element ID → name map.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate atlas"
    )?;
    writeln!(out)?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out)?;
    writeln!(
        out,
        "pub fn get_atlas_name_by_element_id(id: u32) -> Option<&'static str> {{"
    )?;
    writeln!(out, "    ATLAS_ELEMENT_DB.get(&id).copied()")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(
        out,
        "static ATLAS_ELEMENT_DB: phf::Map<u32, &'static str> = phf_map! {{"
    )?;
    Ok(())
}

fn write_element_entries(
    out: &mut File,
    elements: &HashMap<u32, String>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut sorted: Vec<_> = elements.iter().collect();
    sorted.sort_by_key(|(id, _)| *id);

    let mut count = 0u32;
    for (id, name) in &sorted {
        let name_lower = name
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        writeln!(out, "    {}u32 => \"{}\",", id, name_lower)?;
        count += 1;
    }
    Ok(count)
}

fn load_members(path: &Path) -> Result<Vec<MemberEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }

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

fn load_slices(
    path: &Path,
    elements: &HashMap<u32, String>,
) -> Result<HashMap<u32, SliceEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }

        let fields = parse_csv_line(&line);
        if fields.len() >= 7 {
            let element_id: u32 = fields[1].parse()?;
            let Some(name) = elements.get(&element_id) else {
                continue;
            };
            entries.insert(
                element_id,
                SliceEntry {
                    name: name.to_lowercase(),
                    left: fields[2].parse()?,
                    top: fields[3].parse()?,
                    right: fields[4].parse()?,
                    bottom: fields[5].parse()?,
                    mode: fields[6].parse()?,
                },
            );
        }
    }
    Ok(entries)
}
