//! Console Variable (CVar) storage.
//!
//! CVars are configuration values that addons can read/write.
//! Defaults come from WoW's built-in cvars, overrides are stored in memory.

use std::collections::HashMap;
use std::sync::RwLock;

/// CVar storage with defaults and overrides.
pub struct CVarStorage {
    /// Default values (lowercase key -> value)
    defaults: HashMap<String, String>,
    /// Runtime overrides (lowercase key -> value)
    overrides: RwLock<HashMap<String, String>>,
}

impl CVarStorage {
    /// Create storage with defaults parsed from YAML.
    pub fn new() -> Self {
        let defaults = parse_cvar_yaml(include_str!("cvars.yaml"));
        Self {
            defaults,
            overrides: RwLock::new(HashMap::new()),
        }
    }

    /// Get a CVar value (override takes precedence over default).
    pub fn get(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        // Check overrides first
        if let Some(value) = self.overrides.read().unwrap().get(&key) {
            return Some(value.clone());
        }
        // Fall back to defaults
        self.defaults.get(&key).cloned()
    }

    /// Get the default value for a CVar.
    pub fn get_default(&self, name: &str) -> Option<String> {
        self.defaults.get(&name.to_lowercase()).cloned()
    }

    /// Get a CVar as a boolean ("1" = true, anything else = false).
    pub fn get_bool(&self, name: &str) -> bool {
        self.get(name).as_deref() == Some("1")
    }

    /// Set a CVar value.
    pub fn set(&self, name: &str, value: &str) -> bool {
        let key = name.to_lowercase();
        self.overrides
            .write()
            .unwrap()
            .insert(key, value.to_string());
        true
    }

    /// Register a new CVar with a default value.
    pub fn register(&self, name: &str, default: Option<&str>) {
        if let Some(value) = default {
            let key = name.to_lowercase();
            // Only set if not already in defaults
            if !self.defaults.contains_key(&key) {
                self.overrides
                    .write()
                    .unwrap()
                    .insert(key, value.to_string());
            }
        }
    }
}

impl Default for CVarStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse YAML in format `key: 'value'` or `key: value`.
fn parse_cvar_yaml(yaml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            // Strip surrounding quotes if present
            let value = value
                .strip_prefix('\'')
                .and_then(|v| v.strip_suffix('\''))
                .or_else(|| value.strip_prefix('"').and_then(|v| v.strip_suffix('"')))
                .unwrap_or(value);
            map.insert(key, value.to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
            someVar: '1'
            otherVar: "hello"
            plainVar: 50
        "#;
        let map = parse_cvar_yaml(yaml);
        assert_eq!(map.get("somevar"), Some(&"1".to_string()));
        assert_eq!(map.get("othervar"), Some(&"hello".to_string()));
        assert_eq!(map.get("plainvar"), Some(&"50".to_string()));
    }

    #[test]
    fn test_get_set() {
        let storage = CVarStorage::new();
        // Test default
        assert!(storage.get("nameplateShowEnemies").is_some());
        // Test override
        storage.set("nameplateShowEnemies", "0");
        assert_eq!(storage.get("nameplateShowEnemies"), Some("0".to_string()));
        // Test case insensitivity
        assert_eq!(storage.get("NAMEPLATESHOWENEMIES"), storage.get("nameplateShowEnemies"));
    }
}
