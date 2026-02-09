//! Generator for global_strings.rs from WoW CSV exports.
//!
//! Reads from ~/Projects/wow/data/:
//!   - GlobalStrings.csv
//!
//! Generates: data/global_strings.rs

use super::csv_util::{parse_csv_line, wow_data_dir};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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
    let count = write_string_entries(&mut out, reader)?;
    writeln!(out, "}};")?;
    write_tests(&mut out)?;

    println!("Generated {} global string entries", count);
    println!("Output: {}", output_path.display());
    Ok(())
}

fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated global strings from WoW CSV exports.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: wow-cli generate global-strings")?;
    writeln!(out)?;
    writeln!(out, "use phf::phf_map;")?;
    writeln!(out)?;
    writeln!(out, "pub fn get_global_string(name: &str) -> Option<&'static str> {{")?;
    writeln!(out, "    GLOBAL_STRINGS.get(name).copied()")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    writeln!(out, "pub static GLOBAL_STRINGS: phf::Map<&'static str, &'static str> = phf_map! {{")?;
    Ok(())
}

fn write_string_entries(out: &mut File, reader: BufReader<File>) -> Result<u32, Box<dyn std::error::Error>> {
    let mut count = 0u32;

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue;
        }

        let fields = parse_csv_line(&line);
        if fields.len() >= 3 {
            let tag = &fields[1];
            let text = &fields[2];
            if tag.is_empty() {
                continue;
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
    Ok(count)
}

fn write_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_addon_list_string() {{")?;
    writeln!(out, "        assert_eq!(get_global_string(\"ADDON_LIST\"), Some(\"AddOn List\"));")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_common_strings_exist() {{")?;
    writeln!(out, "        assert!(get_global_string(\"OKAY\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"CANCEL\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"ACCEPT\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"DECLINE\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"YES\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"NO\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"ENABLE\").is_some());")?;
    writeln!(out, "        assert!(get_global_string(\"DISABLE\").is_some());")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_nonexistent_string_returns_none() {{")?;
    writeln!(out, "        assert_eq!(get_global_string(\"THIS_STRING_DOES_NOT_EXIST_12345\"), None);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_string_count() {{")?;
    writeln!(out, "        assert!(GLOBAL_STRINGS.len() > 20000);")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}
