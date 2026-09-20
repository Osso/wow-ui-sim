//! Console Variable (CVar) storage.
//!
//! CVars are configuration values that addons can read/write.
//! Defaults come from WoW's built-in cvars, overrides are persisted to disk.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    RwLock,
    atomic::{AtomicU64, Ordering},
};

static NEXT_TEST_STORAGE_ID: AtomicU64 = AtomicU64::new(1);

/// Default path for persisted CVar overrides.
fn default_storage_path() -> PathBuf {
    let in_test_binary = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.ends_with("deps")))
        .unwrap_or(false);

    if in_test_binary {
        let storage_id = NEXT_TEST_STORAGE_ID.fetch_add(1, Ordering::Relaxed);
        return std::env::temp_dir().join(format!(
            "wow-sim-cvars-{}-{}.json",
            std::process::id(),
            storage_id
        ));
    }

    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wow-sim")
        .join("cvars.json")
}

/// CVar storage with defaults and overrides.
pub struct CVarStorage {
    /// Default values (lowercase key -> value)
    defaults: HashMap<String, String>,
    /// Original-case names (lowercase key -> original name)
    original_names: HashMap<String, String>,
    /// Runtime-registered defaults (lowercase key -> value).
    registered_defaults: RwLock<HashMap<String, String>>,
    /// Original-case names for runtime-registered CVars.
    registered_original_names: RwLock<HashMap<String, String>>,
    /// Runtime overrides (lowercase key -> value), persisted to disk.
    overrides: RwLock<HashMap<String, String>>,
    /// Path to persist overrides.
    storage_path: PathBuf,
}

impl CVarStorage {
    /// Create storage with defaults parsed from YAML, loading persisted overrides from disk.
    pub fn new() -> Self {
        let path = default_storage_path();
        let (defaults, original_names) = parse_default_cvars();
        let overrides = load_overrides(&path);
        Self {
            defaults,
            original_names,
            registered_defaults: RwLock::new(HashMap::new()),
            registered_original_names: RwLock::new(HashMap::new()),
            overrides: RwLock::new(overrides),
            storage_path: path,
        }
    }

    /// Create storage with a custom path (for testing).
    #[cfg(test)]
    fn with_path(path: PathBuf) -> Self {
        let (defaults, original_names) = parse_default_cvars();
        let overrides = load_overrides(&path);
        Self {
            defaults,
            original_names,
            registered_defaults: RwLock::new(HashMap::new()),
            registered_original_names: RwLock::new(HashMap::new()),
            overrides: RwLock::new(overrides),
            storage_path: path,
        }
    }

    /// Get a CVar value (override takes precedence over default).
    pub fn get(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        if is_profile_removed_cvar_key(&key) {
            return None;
        }
        // Check overrides first
        if let Some(value) = self.overrides.read().unwrap().get(&key) {
            return Some(value.clone());
        }
        // Fall back to defaults
        self.defaults
            .get(&key)
            .cloned()
            .or_else(|| self.registered_defaults.read().unwrap().get(&key).cloned())
    }

    /// Get the default value for a CVar.
    pub fn get_default(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        if is_profile_removed_cvar_key(&key) {
            return None;
        }
        self.defaults
            .get(&key)
            .cloned()
            .or_else(|| self.registered_defaults.read().unwrap().get(&key).cloned())
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

    /// Get all known CVar names in original case (defaults + overrides).
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: std::collections::HashSet<String> = self.defaults.keys().cloned().collect();
        for key in self.registered_defaults.read().unwrap().keys() {
            keys.insert(key.clone());
        }
        for key in self.overrides.read().unwrap().keys() {
            keys.insert(key.clone());
        }
        keys.retain(|key| !is_profile_removed_cvar_key(key));
        let mut sorted: Vec<String> = keys
            .into_iter()
            .map(|k| {
                self.original_names
                    .get(&k)
                    .cloned()
                    .or_else(|| {
                        self.registered_original_names
                            .read()
                            .unwrap()
                            .get(&k)
                            .cloned()
                    })
                    .unwrap_or(k)
            })
            .collect();
        sorted.sort_by_key(|name| name.to_lowercase());
        sorted
    }

    /// Register a new CVar with a default value.
    pub fn register(&self, name: &str, default: Option<&str>) {
        let key = name.to_lowercase();
        if self.defaults.contains_key(&key) || is_profile_removed_cvar_key(&key) {
            return;
        }

        self.registered_original_names
            .write()
            .unwrap()
            .entry(key.clone())
            .or_insert_with(|| name.to_string());

        let value = default.unwrap_or("0");
        self.registered_defaults
            .write()
            .unwrap()
            .entry(key)
            .or_insert_with(|| value.to_string());
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

fn parse_default_cvars() -> (HashMap<String, String>, HashMap<String, String>) {
    let (mut defaults, mut original_names) = parse_cvar_yaml(include_str!("cvars.yaml"));
    insert_profile_cvars(&mut defaults, &mut original_names);
    remove_profile_cvars(&mut defaults, &mut original_names);
    (defaults, original_names)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn insert_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    insert_cvar_defaults(defaults, original_names, PATCH_12_0_7_CVARS);
    #[cfg(feature = "retail-12-1-0")]
    insert_cvar_defaults(defaults, original_names, PATCH_12_1_CVARS);
}

#[cfg(feature = "client-wowforever")]
fn insert_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    // Native Forever 1.60.1.69913 probe; see docs/specs/forever-gamepad-cvars.md.
    for (name, value) in [
        ("GamepadPossessBarOverride", "4"),
        ("GamepadStanceBarOverride", "3"),
    ] {
        let key = name.to_lowercase();
        original_names.insert(key.clone(), name.to_string());
        defaults.insert(key, value.to_string());
    }
}

#[cfg(not(any(
    feature = "retail-12-0-7",
    feature = "retail-12-1-0",
    feature = "client-wowforever"
)))]
fn insert_profile_cvars(
    _defaults: &mut HashMap<String, String>,
    _original_names: &mut HashMap<String, String>,
) {
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn insert_cvar_defaults(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
    values: &[(&str, &str)],
) {
    for &(name, value) in values {
        let key = name.to_lowercase();
        original_names.insert(key.clone(), name.to_string());
        defaults.insert(key, value.to_string());
    }
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn remove_profile_cvars(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
) {
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_0_REMOVED_CVARS);
    #[cfg(feature = "retail-12-0-7")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_0_7_REMOVED_CVARS);
    #[cfg(feature = "retail-12-1-0")]
    remove_cvar_defaults(defaults, original_names, PATCH_12_1_REMOVED_CVARS);
}

#[cfg(not(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
)))]
fn remove_profile_cvars(
    _defaults: &mut HashMap<String, String>,
    _original_names: &mut HashMap<String, String>,
) {
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn remove_cvar_defaults(
    defaults: &mut HashMap<String, String>,
    original_names: &mut HashMap<String, String>,
    names: &[&str],
) {
    for key in names.iter().map(|name| name.to_lowercase()) {
        defaults.remove(&key);
        original_names.remove(&key);
    }
}

#[cfg(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
))]
fn is_profile_removed_cvar_key(key: &str) -> bool {
    if PATCH_12_0_0_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-0-7")]
    if PATCH_12_0_7_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    #[cfg(feature = "retail-12-1-0")]
    if PATCH_12_1_REMOVED_CVARS
        .iter()
        .any(|removed| removed.eq_ignore_ascii_case(key))
    {
        return true;
    }

    false
}

#[cfg(not(any(
    feature = "retail-12-0-0",
    feature = "retail-12-0-7",
    feature = "retail-12-1-0"
)))]
fn is_profile_removed_cvar_key(_key: &str) -> bool {
    false
}

#[cfg(feature = "retail-12-0-0")]
const PATCH_12_0_0_REMOVED_CVARS: &[&str] = &[
    "NamePlateHorizontalScale",
    "NamePlateVerticalScale",
    "ShowClassColorInFriendlyNameplate",
];

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
const PATCH_12_0_7_REMOVED_CVARS: &[&str] = &[
    "debugGameEvents",
    "frontendMatchingModes_WowLabs",
    "last_matchmaking_party_size",
    "lastCharacterGuid",
    "skipStartGear",
];

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
const PATCH_12_0_7_CVARS: &[(&str, &str)] = &[
    ("assistedCombatReduceHighlights", "1"),
    ("developerLog", "0"),
    ("developerLogFilterDebug", "0"),
    ("developerLogFilterError", "1"),
    ("developerLogFilterFatal", "1"),
    ("developerLogFilterNormal", "1"),
    ("developerLogFilterSpam", "0"),
    ("developerLogFilterWarning", "1"),
    ("developerLogWriteToFile", "1"),
    ("housingDecorLightRadiusIndicatorsEnabled", "1"),
    ("housingOtherDecorLightRadiusIndicatorType", "1"),
    ("housingSelectedDecorLightRadiusIndicatorType", "1"),
    ("KioskCanSessionExpire", "1"),
    ("KioskCharacterTemplateSet", "0"),
    ("KioskLobbyKickSeconds", "30"),
    ("ThreadPoolPerThreadAllocator", "1"),
    ("useBLEEP", "0"),
    ("gxWindowedResolution", "auto"),
];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_REMOVED_CVARS: &[&str] =
    &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_CVARS: &[(&str, &str)] = &[
    ("accessibilityScreenNarrationEnabled", "0"),
    ("accessibilityScreenNarrationSpeechRate", "1.0"),
    ("accessibilityScreenNarrationSpeechVolume", "1.0"),
    ("accessibilityScreenNarrationVoice", ""),
    ("AftermathShaderDebug", "0"),
    ("discordClientEnabled", "0"),
    ("discordDisplayName", ""),
    ("nameplateCheckDistanceForTarget", "60"),
    ("nameplateForceShowUnitName", "0"),
    ("nameplateNotSelectedAlpha", "0.600000"),
    ("nameplatePlayRemovalAnimation", "1"),
    ("nameplateShowAllPersonalAuras", "1"),
    ("nameplateShowFriendlyRealmName", "0"),
    ("nameplateShowFriends", "0"),
    ("pingTarget", "0"),
    ("raidFramesDispelIndicatorOverlayAnimation", "1"),
    ("showPingsOnRaidFrames", "1"),
    ("showScreenNarrationDialog", "1"),
    ("taintLogObjectSecrets", "0"),
    ("userFontScaleGlue", "1.0"),
];

/// Load persisted overrides from disk.
fn load_overrides(path: &PathBuf) -> HashMap<String, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Parse YAML in format `key: 'value'` or `key: value`.
/// Returns (defaults: lowercase_key->value, original_names: lowercase_key->original_key).
fn parse_cvar_yaml(yaml: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut defaults = HashMap::new();
    let mut original_names = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let original_key = key.trim().to_string();
            let lower_key = original_key.to_lowercase();
            let value = value.trim();
            // Strip surrounding quotes and process escapes for double-quoted strings
            let (value, is_double_quoted) = if let Some(inner) =
                value.strip_prefix('"').and_then(|v| v.strip_suffix('"'))
            {
                (inner, true)
            } else if let Some(inner) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\''))
            {
                (inner, false)
            } else {
                (value, false)
            };
            let value = if is_double_quoted {
                process_yaml_escapes(value)
            } else {
                value.to_string()
            };
            original_names.insert(lower_key.clone(), original_key);
            defaults.insert(lower_key, value);
        }
    }
    (defaults, original_names)
}

/// Process YAML double-quoted string escape sequences (\\xHH hex escapes).
fn process_yaml_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 3 < bytes.len() && bytes[i + 1] == b'x' {
            let hex = &s[i + 2..i + 4];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte as char);
                i += 4;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
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
        let (defaults, original_names) = parse_cvar_yaml(yaml);
        assert_eq!(defaults.get("somevar"), Some(&"1".to_string()));
        assert_eq!(defaults.get("othervar"), Some(&"hello".to_string()));
        assert_eq!(defaults.get("plainvar"), Some(&"50".to_string()));
        // Original case preserved
        assert_eq!(original_names.get("somevar"), Some(&"someVar".to_string()));
        assert_eq!(
            original_names.get("othervar"),
            Some(&"otherVar".to_string())
        );
        assert_eq!(
            original_names.get("plainvar"),
            Some(&"plainVar".to_string())
        );
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

    #[test]
    fn test_register_exposes_runtime_default_without_override() {
        let storage = CVarStorage::new();
        storage.register("PraiseTheSun", Some("1"));

        assert_eq!(storage.get("PraiseTheSun"), Some("1".to_string()));
        assert_eq!(storage.get_default("PraiseTheSun"), Some("1".to_string()));
        assert!(storage.all_keys().iter().any(|key| key == "PraiseTheSun"));
    }

    #[cfg(feature = "retail-12-0-7")]
    #[test]
    fn patch_12_0_7_cvar_defaults_match_retail() {
        let storage = CVarStorage::new();
        for (name, value) in PATCH_12_0_7_CVARS {
            assert_eq!(storage.get(name), Some((*value).to_string()), "{name}");
            assert_eq!(
                storage.get_default(name),
                Some((*value).to_string()),
                "{name}"
            );
        }
        assert!(storage.get_bool("assistedCombatReduceHighlights"));
        assert!(!storage.get_bool("developerLogFilterDebug"));
        assert_eq!(
            storage.get("gxWindowedResolution"),
            Some("auto".to_string())
        );
        for name in PATCH_12_0_7_REMOVED_CVARS {
            assert_eq!(storage.get(name), None, "{name}");
        }
    }
}
