//! Extract and convert textures referenced by addons to WebP format.

use image_blp::{convert::blp_to_image, parser::load_blp};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Extract all texture paths referenced in addon XML and Lua files.
pub fn find_texture_references(addons_path: &Path) -> HashSet<String> {
    let mut textures = HashSet::new();

    // Regex to match texture paths like Interface\Buttons\UI-Panel-Button-Up
    // Requires at least two path components after Interface
    let texture_re =
        Regex::new(r#"(?i)Interface[/\\]+([A-Za-z][A-Za-z0-9_-]*)[/\\]+([A-Za-z0-9_/\\-]+)"#)
            .expect("invalid regex");

    for entry in WalkDir::new(addons_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let ext = e.path().extension().and_then(|s| s.to_str());
            matches!(ext, Some("xml" | "lua" | "XML" | "LUA"))
        })
    {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for cap in texture_re.captures_iter(&content) {
                let folder = cap.get(1).unwrap().as_str().to_lowercase();
                let rest = cap
                    .get(2)
                    .unwrap()
                    .as_str()
                    .replace('\\', "/")
                    .to_lowercase();

                // Skip Interface/AddOns paths (addon-specific textures)
                if folder == "addons" {
                    continue;
                }

                // Skip paths with double slashes (malformed)
                if rest.contains("//") {
                    continue;
                }

                let path = format!("interface/{}/{}", folder, rest);
                textures.insert(path);
            }
        }
    }

    textures
}

/// Build a case-insensitive index of texture files in a directory.
pub fn build_file_index(base: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();

    for entry in WalkDir::new(base)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if matches!(
            ext.to_lowercase().as_str(),
            "blp" | "png" | "tga" | "jpg"
        ) {
            // Create lowercase key without extension
            if let Ok(rel) = path.strip_prefix(base) {
                let key = rel
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_lowercase();
                // Prefer PNG over BLP if both exist
                let dominated = index.get(&key).is_some_and(|existing: &PathBuf| {
                    existing
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
                });
                if !dominated {
                    index.insert(key, path.to_path_buf());
                }
            }
        }
    }

    index
}

/// Convert a texture file to WebP format.
pub fn convert_to_webp(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext.eq_ignore_ascii_case("blp") {
        // BLP uses image 0.24, we use 0.25, so extract raw pixels
        let blp = load_blp(src)?;
        let blp_img = blp_to_image(&blp, 0)?;
        let rgba = blp_img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Create new image with current image crate version
        let img =
            image::RgbaImage::from_raw(width, height, rgba.into_raw()).expect("invalid dimensions");
        img.save(dst)?;
    } else {
        let img = image::open(src)?;
        let rgba = img.to_rgba8();
        rgba.save(dst)?;
    }

    Ok(())
}

/// Collect all texture references from addons and Blizzard base UI.
fn collect_texture_references(addons_path: &Path) -> HashSet<String> {
    println!("Scanning for texture references...");
    let mut textures = find_texture_references(addons_path);

    let wow_ui_source = addons_path.parent().map(|p| p.join("BlizzardUI"));
    if let Some(ref ui_source) = wow_ui_source
        && ui_source.exists() {
            textures.extend(find_texture_references(ui_source));
        }

    println!("Found {} unique texture references", textures.len());
    textures
}

/// Convert a single texture reference, returning true if found/converted, false if missing.
fn convert_texture_reference(
    lookup_key: &str,
    index: &HashMap<String, PathBuf>,
    output_dir: &Path,
) -> bool {
    let out_file = output_dir.join(format!("{}.webp", lookup_key));

    if out_file.exists() {
        println!("SKIP: {}", lookup_key);
        return true;
    }

    let Some(src_file) = index.get(lookup_key) else {
        println!("MISS: {}", lookup_key);
        return false;
    };

    if let Some(parent) = out_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    match convert_to_webp(src_file, &out_file) {
        Ok(()) => {
            println!("CONV: {}", lookup_key);
            true
        }
        Err(e) => {
            println!("FAIL: {} ({})", lookup_key, e);
            false
        }
    }
}

/// Extract textures referenced by addons and convert to WebP.
///
/// Returns (found, missing) counts.
pub fn extract_textures(
    addons_path: &Path,
    interface_path: &Path,
    output_dir: &Path,
) -> (usize, usize) {
    let textures = collect_texture_references(addons_path);

    println!("Building file index...");
    let index = build_file_index(interface_path);
    println!("Indexed {} texture files", index.len());

    std::fs::create_dir_all(output_dir).expect("failed to create output directory");

    let mut found = 0;
    let mut missing = 0;

    for texture_path in &textures {
        let lookup_key = texture_path
            .strip_prefix("interface/")
            .unwrap_or(texture_path);

        if convert_texture_reference(lookup_key, &index, output_dir) {
            found += 1;
        } else {
            missing += 1;
        }
    }

    (found, missing)
}
