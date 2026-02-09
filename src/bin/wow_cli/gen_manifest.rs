//! Generator for manifest_interface_data.rs from WoW ManifestInterfaceData CSV.
//!
//! Reads: ~/Downloads/ManifestInterfaceData.12.0.1.65769.csv
//!   Columns: ID, FilePath, FileName
//!
//! Generates: data/manifest_interface_data.rs

use super::csv_util::{escape_str, parse_csv_line};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = dirs::home_dir()
        .expect("No home dir")
        .join("Downloads/ManifestInterfaceData.12.0.1.65769.csv");

    let entries = load_manifest(&csv_path)?;
    println!("ManifestInterfaceData: {} entries", entries.len());

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/manifest_interface_data.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    let count = build_manifest_map(&mut out, &entries)?;
    write_lookup_fn(&mut out)?;
    write_tests(&mut out)?;

    println!("Generated {} manifest entries", count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn build_manifest_map(
    out: &mut File,
    entries: &[(u32, String)],
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    let mut count = 0u32;

    for (id, path) in entries {
        let escaped = escape_str(path);
        builder.entry(*id, &format!("\"{}\"", escaped));
        count += 1;
    }

    writeln!(
        out,
        "pub static MANIFEST: phf::Map<u32, &'static str> = {};",
        builder.build()
    )?;
    writeln!(out)?;
    Ok(count)
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated file data ID to texture path mapping.")?;
    writeln!(
        out,
        "//! Do not edit manually - regenerate with: wow-cli generate manifest"
    )?;
    writeln!(out)?;
    Ok(())
}

fn write_lookup_fn(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "pub fn get_texture_path(id: u32) -> Option<&'static str> {{")?;
    writeln!(out, "    MANIFEST.get(&id).copied()")?;
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
    writeln!(out, "    fn test_manifest_count() {{")?;
    writeln!(out, "        assert!(MANIFEST.len() > 100_000);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_default_icon() {{")?;
    writeln!(
        out,
        "        let path = get_texture_path(136243).expect(\"default icon\");"
    )?;
    writeln!(
        out,
        "        assert_eq!(path, \"ICONS/Trade_Engineering\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_paladin_icon() {{")?;
    writeln!(
        out,
        "        let path = get_texture_path(236253).expect(\"paladin icon\");"
    )?;
    writeln!(
        out,
        "        assert_eq!(path, \"ICONS/Ability_Paladin_HammeroftheRighteous\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_id() {{")?;
    writeln!(out, "        assert!(get_texture_path(999_999_999).is_none());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn load_manifest(
    path: &Path,
) -> Result<Vec<(u32, String)>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }
        let fields = parse_csv_line(&line);
        if fields.len() < 3 {
            continue;
        }
        let id: u32 = match fields[0].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let dir = &fields[1];
        let filename = &fields[2];

        let full = format!("{}{}", dir, filename);
        let wow_path = normalize_to_wow_path(&full);
        if !wow_path.is_empty() {
            entries.push((id, wow_path));
        }
    }
    Ok(entries)
}

fn normalize_to_wow_path(raw: &str) -> String {
    let mut path = raw.replace('\\', "/");

    if let Some(rest) = strip_prefix_ci(&path, "Interface/") {
        path = rest.to_string();
    }

    if let Some(dot_pos) = path.rfind('.') {
        path.truncate(dot_pos);
    }

    if path.ends_with('/') {
        path.pop();
    }

    path
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len()
        && s[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}
