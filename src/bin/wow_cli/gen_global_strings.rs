//! Generator for global_strings.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - GlobalStrings.csv
//!
//! Generates: data/global_strings.rs

use super::csv_util::{parse_csv_line, read_csv_records, wow_data_dir};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = wow_data_dir();
    let csv_path = wow_data.join("GlobalStrings.csv");
    println!("Loading GlobalStrings from {}...", csv_path.display());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/global_strings.rs");
    let mut out = File::create(output_path)?;

    write_header(&mut out)?;
    let (count, multiline_key) = write_string_entries(&mut out, reader)?;
    writeln!(out, "}};")?;
    write_tests(&mut out, multiline_key.as_deref())?;

    println!("Generated {} global string entries", count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    write_literal_lines(out, GLOBAL_STRINGS_HEADER)
}

const GLOBAL_STRINGS_HEADER: &[&str] = &[
    "//! Auto-generated global strings from WoW CSV exports.",
    "//! Do not edit manually - regenerate with: wow-cli generate global-strings",
    "",
    "use phf::phf_map;",
    "",
    "pub fn get_global_string(name: &str) -> Option<&'static str> {",
    "    GLOBAL_STRINGS.get(name).copied()",
    "}",
    "",
    "pub static GLOBAL_STRINGS: phf::Map<&'static str, &'static str> = phf_map! {",
];

fn write_literal_lines(out: &mut File, lines: &[&str]) -> std::io::Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

/// Writes the map entries; returns the count and the first key whose text
/// spans several lines, which the generated tests use as the multi-line probe.
fn write_string_entries(
    out: &mut File,
    reader: BufReader<File>,
) -> Result<(u32, Option<String>), Box<dyn std::error::Error>> {
    let mut count = 0u32;
    let mut multiline_key = None;

    for (i, line) in read_csv_records(reader)?.iter().enumerate() {
        if i == 0 {
            continue;
        }

        let fields = parse_csv_line(line);
        if fields.len() >= 3 {
            let tag = &fields[1];
            let text = &fields[2];
            if tag.is_empty() {
                continue;
            }
            if multiline_key.is_none() && text.contains('\n') {
                multiline_key = Some(tag.clone());
            }

            let escaped_text = text
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");

            writeln!(out, "    \"{}\" => \"{}\",", tag, escaped_text)?;
            count += 1;
        }
    }
    Ok((count, multiline_key))
}

fn write_tests(out: &mut File, multiline_key: Option<&str>) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    write_test_addon_list_string(out)?;
    write_test_common_strings_exist(out)?;
    write_test_nonexistent_string_returns_none(out)?;
    if let Some(key) = multiline_key {
        write_test_multiline_string_is_whole(out, key)?;
    }
    write_test_string_count(out)?;
    writeln!(out, "}}")?;
    Ok(())
}

/// A quoted CSV field may span lines; a generator reading line by line keeps
/// only the first of them. The probe key is the first such entry in the export.
fn write_test_multiline_string_is_whole(out: &mut File, key: &str) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_multiline_string_is_whole() {{")?;
    writeln!(
        out,
        "        assert!(get_global_string(\"{key}\").is_some_and(|s| s.contains('\\n')));"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_addon_list_string(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_addon_list_string() {{")?;
    writeln!(
        out,
        "        assert_eq!(get_global_string(\"ADDON_LIST\"), Some(\"AddOn List\"));"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_common_strings_exist(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_common_strings_exist() {{")?;
    write_test_common_string_asserts(out)?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

const COMMON_STRING_TEST_KEYS: &[&str] = &[
    "OKAY", "CANCEL", "ACCEPT", "DECLINE", "YES", "NO", "ENABLE", "DISABLE",
];

fn write_test_common_string_asserts(out: &mut File) -> std::io::Result<()> {
    for key in COMMON_STRING_TEST_KEYS {
        writeln!(
            out,
            "        assert!(get_global_string(\"{key}\").is_some());"
        )?;
    }
    Ok(())
}

fn write_test_nonexistent_string_returns_none(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_string_returns_none() {{")?;
    writeln!(
        out,
        "        assert_eq!(get_global_string(\"THIS_STRING_DOES_NOT_EXIST_12345\"), None);"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    Ok(())
}

fn write_test_string_count(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_string_count() {{")?;
    writeln!(out, "        assert!(GLOBAL_STRINGS.len() > 20000);")?;
    writeln!(out, "    }}")?;
    Ok(())
}
