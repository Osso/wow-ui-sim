//! Read enum expectations from the active profile's pinned Blizzard documentation.

use std::collections::BTreeMap;

pub struct PinnedEnum {
    pub values: BTreeMap<String, i64>,
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let value = line.split_once(&format!("{key} = \""))?.1;
    Some(value.split_once('"')?.0.to_owned())
}

fn numeric_value(line: &str, key: &str) -> Option<i64> {
    let value = line.split_once(&format!("{key} = "))?.1;
    value
        .split(|c: char| !(c.is_ascii_digit() || c == '-'))
        .next()?
        .parse()
        .ok()
}

pub fn read_pinned_enum(source: &str, name: &str) -> PinnedEnum {
    let file = source.split(':').next().expect("documentation filename");
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("resolve pinned Blizzard UI")
        .join("Blizzard_APIDocumentationGenerated");
    let path = root.join(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let lines: Vec<_> = text.lines().collect();
    let start = lines
        .windows(2)
        .position(|pair| quoted_value(pair[0], "Name").as_deref() == Some(name)
            && quoted_value(pair[1], "Type").as_deref() == Some("Enumeration"))
        .unwrap_or_else(|| panic!("enum {name} not found in {}", path.display()));
    let mut meta = BTreeMap::new();
    let mut values = BTreeMap::new();
    for line in &lines[start + 2..] {
        for key in ["NumValues", "MinValue", "MaxValue"] {
            if let Some(value) = numeric_value(line, key) {
                meta.insert(key.to_owned(), value);
            }
        }
        if let (Some(key), Some(value)) = (quoted_value(line, "Name"), numeric_value(line, "EnumValue")) {
            assert!(values.insert(key, value).is_none(), "duplicate enum member in {}", path.display());
        }
        if !values.is_empty() && line.trim() == "}," {
            break;
        }
    }
    assert_eq!(meta.len(), 3, "incomplete metadata for {name} in {}", path.display());
    assert!(!values.is_empty(), "empty enum {name} in {}", path.display());
    PinnedEnum { values }
}
