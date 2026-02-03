//! Generator for global_strings.rs from WoW CSV exports.
//!
//! Run with: cargo run --bin gen_global_strings
//!
//! Reads from ~/Projects/wow/data/:
//!   - GlobalStrings.csv
//!
//! Generates: data/global_strings.rs

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wow_data = dirs::home_dir()
        .expect("No home dir")
        .join("Projects/wow/data");

    let csv_path = wow_data.join("GlobalStrings.csv");
    println!("Loading GlobalStrings from {}...", csv_path.display());

    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);

    std::fs::create_dir_all("data")?;
    let output_path = Path::new("data/global_strings.rs");
    let mut out = File::create(output_path)?;

    writeln!(out, "//! Auto-generated global strings from WoW CSV exports.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: cargo run --bin gen_global_strings")?;
    writeln!(out, "")?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out, "")?;
    writeln!(out, "/// Get a global string by name.")?;
    writeln!(out, "/// Returns the string value or None if not found.")?;
    writeln!(out, "pub fn get_global_string(name: &str) -> Option<&'static str> {{")?;
    writeln!(out, "    GLOBAL_STRINGS.get(name).copied()")?;
    writeln!(out, "}}")?;
    writeln!(out, "")?;
    writeln!(out, "/// Global strings database (compile-time perfect hash map).")?;
    writeln!(out, "pub static GLOBAL_STRINGS: phf::Map<&'static str, &'static str> = phf_map! {{")?;

    let mut count = 0;

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        } // Skip header: ID,BaseTag,TagText_lang,Flags

        let fields = parse_csv_line(&line);
        // ID, BaseTag, TagText_lang, Flags
        if fields.len() >= 3 {
            let tag = &fields[1];
            let text = &fields[2];

            // Skip empty tags
            if tag.is_empty() {
                continue;
            }

            // Escape for Rust string literal
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

    writeln!(out, "}};")?;

    // Generate tests
    writeln!(out, "")?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out, "")?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_addon_list_string() {{")?;
    writeln!(out, "        assert_eq!(get_global_string(\"ADDON_LIST\"), Some(\"AddOn List\"));")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_common_strings_exist() {{")?;
    writeln!(out, "        // UI buttons")?;
    writeln!(out, "        assert!(get_global_string(\"OKAY\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"CANCEL\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"ACCEPT\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"DECLINE\").is_some());")?;
    writeln!(out, "        // Common labels")?;
    writeln!(out, "        assert!(get_global_string(\"YES\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"NO\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"ENABLE\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"DISABLE\").is_some());")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_string_returns_none() {{")?;
    writeln!(out, "        assert_eq!(get_global_string(\"THIS_STRING_DOES_NOT_EXIST_12345\"), None);")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_string_count() {{")?;
    writeln!(out, "        // Sanity check: should have thousands of strings")?;
    writeln!(out, "        assert!(GLOBAL_STRINGS.len() > 20000);")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    println!("Generated {} global string entries", count);
    println!("Output: {}", output_path.display());

    Ok(())
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
