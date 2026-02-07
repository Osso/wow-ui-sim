//! Console Variable (CVar) storage.
//!
//! CVars are configuration values that addons can read/write.
//! Defaults come from WoW's built-in cvars, overrides are persisted to disk.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Default path for persisted CVar overrides.
fn default_storage_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wow-sim")
        .join("cvars.json")
}

/// CVar storage with defaults and overrides.
pub struct CVarStorage {
    /// Default values (lowercase key -> value)
    defaults: HashMap<String, String>,
    /// Runtime overrides (lowercase key -> value), persisted to disk.
    overrides: RwLock<HashMap<String, String>>,
    /// Path to persist overrides.
    storage_path: PathBuf,
}

impl CVarStorage {
    /// Create storage with defaults parsed from YAML, loading persisted overrides from disk.
    pub fn new() -> Self {
        let path = default_storage_path();
        let defaults = parse_cvar_yaml(include_str!("cvars.yaml"));
        let overrides = load_overrides(&path);
        Self {
            defaults,
            overrides: RwLock::new(overrides),
            storage_path: path,
        }
    }

    /// Create storage with a custom path (for testing).
    #[cfg(test)]
    fn with_path(path: PathBuf) -> Self {
        let defaults = parse_cvar_yaml(include_str!("cvars.yaml"));
        let overrides = load_overrides(&path);
        Self {
            defaults,
            overrides: RwLock::new(overrides),
            storage_path: path,
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

    /// Set a CVar value and persist to disk.
    pub fn set(&self, name: &str, value: &str) -> bool {
        let key = name.to_lowercase();
        self.overrides
            .write()
            .unwrap()
            .insert(key, value.to_string());
        self.save();
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

    /// Persist current overrides to disk.
    fn save(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let overrides = self.overrides.read().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*overrides) {
            let _ = std::fs::write(&self.storage_path, json);
        }
    }
}

impl Default for CVarStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Load persisted overrides from disk.
fn load_overrides(path: &PathBuf) -> HashMap<String, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
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
    use std::fs;

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
        assert_eq!(
            storage.get("NAMEPLATESHOWENEMIES"),
            storage.get("nameplateShowEnemies")
        );
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cvars.json");

        // Set overrides and verify they're written to disk
        {
            let storage = CVarStorage::with_path(path.clone());
            storage.set("checkaddonversion", "0");
            storage.set("someCustomVar", "hello");
            assert!(path.exists(), "cvars.json should exist after set()");
        }

        // Load from same path — overrides should survive
        {
            let storage = CVarStorage::with_path(path.clone());
            assert_eq!(
                storage.get("checkaddonversion"),
                Some("0".to_string()),
                "persisted override should take precedence over default"
            );
            // Keys are stored lowercase, so any casing resolves the same value
            assert_eq!(
                storage.get("someCustomVar"),
                Some("hello".to_string()),
                "case-insensitive lookup should find persisted value"
            );
            assert_eq!(
                storage.get("somecustomvar"),
                Some("hello".to_string()),
                "lowercase lookup should also work"
            );
        }
    }

    #[test]
    fn test_persistence_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent").join("cvars.json");

        // No file — should fall back to defaults without error
        let storage = CVarStorage::with_path(path.clone());
        assert_eq!(storage.get("checkaddonversion"), Some("1".to_string()));

        // set() should create parent dirs
        storage.set("checkaddonversion", "0");
        assert!(path.exists());
    }

    #[test]
    fn test_persistence_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cvars.json");
        fs::write(&path, "not valid json {{{").unwrap();

        // Corrupt file — should fall back to defaults
        let storage = CVarStorage::with_path(path);
        assert_eq!(storage.get("checkaddonversion"), Some("1".to_string()));
    }
}
