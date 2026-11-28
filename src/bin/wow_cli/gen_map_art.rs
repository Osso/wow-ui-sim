//! Generator for map_art.rs from WoW DB2 CSV exports.
//!
//! Reads from data/db2/:
//!   - UiMapXMapArt.csv    (mapID → artID)
//!   - UiMapArt.csv        (artID → styleID)
//!   - UiMapArtStyleLayer.csv (styleID → layer dimensions)
//!   - UiMapArtTile.csv    (artID + layerIndex → fileDataIDs)
//!
//! Generates: crates/sim-data/data/map_art.rs, and from the WoW Forever build's
//! exports in data/db2/wowforever-1.60.1.70707/ crates/sim-data/data/map_art_forever.rs
//! (that client's own uiMap IDs, which the retail tables do not carry).

use super::csv_util::parse_csv_line;
use super::csv_util::read_csv_records;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

type TileMaps = HashMap<(u32, u32), BTreeMap<(u32, u32), u32>>;

/// One layer's dimension info from UiMapArtStyleLayer.
#[derive(Debug, Clone)]
struct StyleLayer {
    layer_index: u32,
    layer_width: u32,
    layer_height: u32,
    tile_width: u32,
    tile_height: u32,
    min_scale: String,
    max_scale: String,
    additional_zoom_steps: u32,
}

const FOREVER_DB2_DIR: &str = "data/db2/wowforever-1.60.1.70707";

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    generate(
        Path::new("data/db2"),
        Path::new("crates/sim-data/data/map_art.rs"),
        true,
    )?;
    generate(
        Path::new(FOREVER_DB2_DIR),
        Path::new("crates/sim-data/data/map_art_forever.rs"),
        false,
    )
}

/// `own_structs` false: the output reuses `map_art`'s structs so callers see one type.
fn generate(
    db2_dir: &Path,
    output_path: &Path,
    own_structs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading UiMapArtStyleLayer...");
    // styleID → Vec<StyleLayer> (sorted by layer_index)
    let style_layers = load_style_layers(&db2_dir.join("UiMapArtStyleLayer.csv"))?;

    println!("Loading UiMapArt...");
    // artID → styleID
    let art_style: HashMap<u32, u32> = load_art_style(&db2_dir.join("UiMapArt.csv"))?;

    println!("Loading UiMapArtTile...");
    // (artID, layerIndex) → BTreeMap<(row,col), fileDataID>
    let tiles = load_tiles(&db2_dir.join("UiMapArtTile.csv"))?;

    println!("Loading UiMapXMapArt...");
    // mapID → artID (PhaseID=0 only)
    let map_art = load_map_art(&db2_dir.join("UiMapXMapArt.csv"))?;

    let mut out = File::create(output_path)?;

    let map_count = write_map_art(
        &mut out,
        &map_art,
        &art_style,
        &style_layers,
        &tiles,
        own_structs,
    )?;

    println!("Generated {} map entries", map_count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn load_style_layers(
    path: &Path,
) -> Result<HashMap<u32, Vec<StyleLayer>>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map: HashMap<u32, Vec<StyleLayer>> = HashMap::new();

    for (i, line) in read_csv_records(reader)?.into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 10 {
            continue;
        }
        let style_id: u32 = f[9].parse().unwrap_or(0);
        if style_id == 0 {
            continue;
        }
        map.entry(style_id).or_default().push(StyleLayer {
            layer_index: f[1].parse().unwrap_or(0),
            layer_width: f[2].parse().unwrap_or(0),
            layer_height: f[3].parse().unwrap_or(0),
            tile_width: f[4].parse().unwrap_or(0),
            tile_height: f[5].parse().unwrap_or(0),
            min_scale: f[6].clone(),
            max_scale: f[7].clone(),
            additional_zoom_steps: f[8].parse().unwrap_or(0),
        });
    }
    // Sort each style's layers by layer_index
    for layers in map.values_mut() {
        layers.sort_by_key(|l| l.layer_index);
    }
    Ok(map)
}

fn load_art_style(path: &Path) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for (i, line) in read_csv_records(reader)?.into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 4 {
            continue;
        }
        let art_id: u32 = match f[0].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let style_id: u32 = f[3].parse().unwrap_or(0);
        if style_id > 0 {
            map.insert(art_id, style_id);
        }
    }
    Ok(map)
}

fn load_tiles(path: &Path) -> Result<TileMaps, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // key: (artID, layerIndex), value: BTreeMap<(row, col), fileDataID>
    let mut map: TileMaps = HashMap::new();

    for (i, line) in read_csv_records(reader)?.into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 6 {
            continue;
        }
        let row: u32 = f[1].parse().unwrap_or(0);
        let col: u32 = f[2].parse().unwrap_or(0);
        let layer: u32 = f[3].parse().unwrap_or(0);
        let file_data_id: u32 = f[4].parse().unwrap_or(0);
        let art_id: u32 = f[5].parse().unwrap_or(0);

        if file_data_id == 0 || art_id == 0 {
            continue;
        }
        map.entry((art_id, layer))
            .or_default()
            .insert((row, col), file_data_id);
    }
    Ok(map)
}

fn load_map_art(path: &Path) -> Result<BTreeMap<u32, u32>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = BTreeMap::new();

    for (i, line) in read_csv_records(reader)?.into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        let f = parse_csv_line(&line);
        if f.len() < 4 {
            continue;
        }
        let phase_id: u32 = f[1].parse().unwrap_or(u32::MAX);
        if phase_id != 0 {
            continue;
        }
        let art_id: u32 = f[2].parse().unwrap_or(0);
        let map_id: u32 = f[3].parse().unwrap_or(0);
        if art_id > 0 && map_id > 0 {
            map.insert(map_id, art_id);
        }
    }
    Ok(map)
}

fn write_map_art(
    out: &mut File,
    map_art: &BTreeMap<u32, u32>,
    art_style: &HashMap<u32, u32>,
    style_layers: &HashMap<u32, Vec<StyleLayer>>,
    tiles: &TileMaps,
    own_structs: bool,
) -> Result<u32, Box<dyn std::error::Error>> {
    write_file_header(out)?;
    if own_structs {
        write_structs(out)?;
    } else {
        writeln!(out, "pub use super::map_art::{{MapArtInfo, MapArtLayer}};")?;
        writeln!(out)?;
    }

    let mut art_ids: Vec<u32> = map_art.values().copied().collect();
    art_ids.sort_unstable();
    art_ids.dedup();

    for &art_id in &art_ids {
        let Some(&style_id) = art_style.get(&art_id) else {
            continue;
        };
        let Some(layers) = style_layers.get(&style_id) else {
            continue;
        };
        emit_art_statics(out, art_id, layers, tiles)?;
    }

    writeln!(out)?;
    let count = emit_phf_map(out, map_art, art_style, style_layers)?;
    write_lookup_fn(out)?;
    Ok(count)
}

/// Flatten a tile BTreeMap<(row,col), fileDataID> into a row-major Vec.
fn flatten_tile_map(tm: &BTreeMap<(u32, u32), u32>) -> Vec<u32> {
    let max_row = tm.keys().map(|&(r, _)| r).max().unwrap_or(0);
    let max_col = tm.keys().map(|&(_, c)| c).max().unwrap_or(0);
    let num_cols = max_col + 1;
    let len = ((max_row + 1) * num_cols) as usize;
    let mut v = vec![0u32; len];
    for (&(row, col), &fdid) in tm {
        v[(row * num_cols + col) as usize] = fdid;
    }
    v
}

/// Emit tile arrays, layer info, and tile-slice arrays for one art ID.
fn emit_art_statics(
    out: &mut File,
    art_id: u32,
    layers: &[StyleLayer],
    tiles: &TileMaps,
) -> std::io::Result<()> {
    emit_layer_tile_statics(out, art_id, layers, tiles)?;
    emit_layers_static(out, art_id, layers)?;
    emit_tile_slices_static(out, art_id, layers)?;
    Ok(())
}

fn emit_layer_tile_statics(
    out: &mut File,
    art_id: u32,
    layers: &[StyleLayer],
    tiles: &TileMaps,
) -> std::io::Result<()> {
    for layer in layers {
        let ids = tiles
            .get(&(art_id, layer.layer_index))
            .map(flatten_tile_map)
            .unwrap_or_default();
        write!(
            out,
            "static ART_{}_LAYER_{}_TILES: [u32; {}] = [",
            art_id,
            layer.layer_index,
            ids.len()
        )?;
        for (i, &id) in ids.iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            write!(out, "{}", id)?;
        }
        writeln!(out, "];")?;
    }
    Ok(())
}

fn emit_layers_static(out: &mut File, art_id: u32, layers: &[StyleLayer]) -> std::io::Result<()> {
    writeln!(
        out,
        "static ART_{}_LAYERS: [MapArtLayer; {}] = [",
        art_id,
        layers.len()
    )?;
    for layer in layers {
        writeln!(
            out,
            "    MapArtLayer {{ layer_width: {}, layer_height: {}, tile_width: {}, tile_height: {}, min_scale: {}f32, max_scale: {}f32, additional_zoom_steps: {} }},",
            layer.layer_width,
            layer.layer_height,
            layer.tile_width,
            layer.tile_height,
            layer.min_scale,
            layer.max_scale,
            layer.additional_zoom_steps,
        )?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn emit_tile_slices_static(
    out: &mut File,
    art_id: u32,
    layers: &[StyleLayer],
) -> std::io::Result<()> {
    writeln!(
        out,
        "static ART_{}_TILE_SLICES: [&[u32]; {}] = [",
        art_id,
        layers.len()
    )?;
    for layer in layers {
        writeln!(
            out,
            "    &ART_{}_LAYER_{}_TILES,",
            art_id, layer.layer_index
        )?;
    }
    writeln!(out, "];")?;
    Ok(())
}

/// Build and emit the phf::Map from mapID → MapArtInfo.
fn emit_phf_map(
    out: &mut File,
    map_art: &BTreeMap<u32, u32>,
    art_style: &HashMap<u32, u32>,
    style_layers: &HashMap<u32, Vec<StyleLayer>>,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;

    for (&map_id, &art_id) in map_art {
        let Some(&style_id) = art_style.get(&art_id) else {
            continue;
        };
        if style_layers.get(&style_id).is_none() {
            continue;
        }
        let value = format!(
            "MapArtInfo {{ art_id: {}, layers: &ART_{}_LAYERS, tiles: &ART_{}_TILE_SLICES }}",
            art_id, art_id, art_id
        );
        builder.entry(map_id, &value);
        count += 1;
    }

    writeln!(
        out,
        "pub static MAP_ART_DB: phf::Map<u32, MapArtInfo> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok(count)
}

fn write_file_header(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "//! Auto-generated map art tile data from WoW DB2 exports."
    )?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate map-art"
    )?;
    writeln!(out)?;
    Ok(())
}

fn write_structs(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, MAP_ART_LAYER_STRUCT)?;
    write_literal_lines(out, MAP_ART_INFO_STRUCT)?;
    Ok(())
}

const MAP_ART_LAYER_STRUCT: &[&str] = &[
    "/// Map art layer info (dimensions and scale).",
    "#[derive(Debug, Clone, Copy)]",
    "pub struct MapArtLayer {",
    "    pub layer_width: u32,",
    "    pub layer_height: u32,",
    "    pub tile_width: u32,",
    "    pub tile_height: u32,",
    "    pub min_scale: f32,",
    "    pub max_scale: f32,",
    "    pub additional_zoom_steps: u32,",
    "}",
    "",
];

const MAP_ART_INFO_STRUCT: &[&str] = &[
    "/// Map art info (art ID, style layers, and tile fileDataIDs per layer).",
    "#[derive(Debug, Clone)]",
    "pub struct MapArtInfo {",
    "    pub art_id: u32,",
    "    pub layers: &'static [MapArtLayer],",
    "    pub tiles: &'static [&'static [u32]], // per layer, ordered row-major",
    "}",
    "",
];

fn write_literal_lines(out: &mut File, lines: &[&str]) -> std::io::Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(
        out,
        "pub fn get_map_art(map_id: u32) -> Option<&'static MapArtInfo> {{"
    )?;
    writeln!(out, "    MAP_ART_DB.get(&map_id)")?;
    writeln!(out, "}}")?;
    Ok(())
}
