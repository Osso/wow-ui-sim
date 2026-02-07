//! Saved variables management for addon persistence.
//!
//! WoW addons can declare SavedVariables and SavedVariablesPerCharacter in their
//! .toc files. These are global Lua tables that persist between sessions.
//!
//! Storage uses WoW-compatible Lua format (`VarName = { ... }`), so files can be
//! shared between the simulator and a real WoW installation.
//!
//! Loading priority:
//! 1. WTF directory (real WoW installation, if configured)
//! 2. Simulator storage (~/.local/share/wow-sim/SavedVariables/)

use mlua::{Lua, Result, Table, Value};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

/// Configuration for loading WTF saved variables from a real WoW installation.
#[derive(Debug, Clone)]
pub struct WtfConfig {
    /// Base WTF directory path (e.g., /path/to/WoW/WTF)
    pub wtf_path: PathBuf,
    /// Account ID/name (e.g., "50868465#2")
    pub account: String,
    /// Realm name (e.g., "Burning Blade")
    pub realm: String,
    /// Character name (e.g., "Haky")
    pub character: String,
}

impl WtfConfig {
    /// Create a new WTF configuration.
    pub fn new(wtf_path: impl Into<PathBuf>, account: &str, realm: &str, character: &str) -> Self {
        Self {
            wtf_path: wtf_path.into(),
            account: account.to_string(),
            realm: realm.to_string(),
            character: character.to_string(),
        }
    }

    /// Get the path to account-level SavedVariables directory.
    pub fn account_saved_vars_path(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join("SavedVariables")
    }

    /// Get the path to character-level SavedVariables directory.
    pub fn character_saved_vars_path(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join(&self.realm)
            .join(&self.character)
            .join("SavedVariables")
    }

    /// Get the path to account-level SavedVariables file for an addon.
    pub fn account_saved_vars_file(&self, addon_name: &str) -> PathBuf {
        self.account_saved_vars_path().join(format!("{}.lua", addon_name))
    }

    /// Get the path to character-level SavedVariables file for an addon.
    pub fn character_saved_vars_file(&self, addon_name: &str) -> PathBuf {
        self.character_saved_vars_path().join(format!("{}.lua", addon_name))
    }
}

/// Manages saved variables for all loaded addons.
#[derive(Debug)]
pub struct SavedVariablesManager {
    /// Base directory for saved variables storage.
    storage_dir: PathBuf,
    /// Character name for per-character variables.
    character_name: String,
    /// Realm name for per-character variables.
    realm_name: String,
    /// Track which variables have been registered (addon_name -> var_names).
    registered: HashMap<String, Vec<String>>,
    /// Track per-character variables.
    registered_per_char: HashMap<String, Vec<String>>,
    /// Optional WTF configuration for loading real WoW saved variables.
    wtf_config: Option<WtfConfig>,
    /// Track which addons have had WTF variables loaded.
    wtf_loaded: HashMap<String, bool>,
}

impl SavedVariablesManager {
    /// Create a new manager with default storage location.
    pub fn new() -> Self {
        let storage_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wow-sim")
            .join("SavedVariables");

        Self {
            storage_dir,
            character_name: "SimPlayer".to_string(),
            realm_name: "SimRealm".to_string(),
            registered: HashMap::new(),
            registered_per_char: HashMap::new(),
            wtf_config: None,
            wtf_loaded: HashMap::new(),
        }
    }

    /// Create with custom storage directory.
    pub fn with_storage_dir(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            character_name: "SimPlayer".to_string(),
            realm_name: "SimRealm".to_string(),
            registered: HashMap::new(),
            registered_per_char: HashMap::new(),
            wtf_config: None,
            wtf_loaded: HashMap::new(),
        }
    }

    /// Set character info for per-character variables.
    pub fn set_character(&mut self, name: &str, realm: &str) {
        self.character_name = name.to_string();
        self.realm_name = realm.to_string();
    }

    /// Set WTF configuration for loading real WoW saved variables.
    pub fn set_wtf_config(&mut self, config: WtfConfig) {
        // Also update character info from WTF config
        self.character_name = config.character.clone();
        self.realm_name = config.realm.clone();
        self.wtf_config = Some(config);
    }

    /// Get a reference to the WTF configuration.
    pub fn wtf_config(&self) -> Option<&WtfConfig> {
        self.wtf_config.as_ref()
    }

    /// Load WTF saved variables for an addon from the real WoW installation.
    /// This executes the Lua files to set global variables.
    /// Returns the number of files loaded (0, 1, or 2 for account + character).
    pub fn load_wtf_for_addon(&mut self, lua: &Lua, addon_name: &str) -> Result<usize> {
        let config = match &self.wtf_config {
            Some(c) => c.clone(),
            None => return Ok(0), // No WTF config, skip
        };

        // Skip if already loaded
        if self.wtf_loaded.contains_key(addon_name) {
            return Ok(0);
        }

        let mut loaded = 0;

        // Load account-level SavedVariables
        let account_file = config.account_saved_vars_file(addon_name);
        if account_file.exists() {
            match self.load_wtf_lua_file(lua, &account_file) {
                Ok(()) => loaded += 1,
                Err(e) => {
                    tracing::warn!(
                        "Failed to load account SavedVariables for {}: {}",
                        addon_name,
                        e
                    );
                }
            }
        }

        // Load character-level SavedVariables
        let char_file = config.character_saved_vars_file(addon_name);
        if char_file.exists() {
            match self.load_wtf_lua_file(lua, &char_file) {
                Ok(()) => loaded += 1,
                Err(e) => {
                    tracing::warn!(
                        "Failed to load character SavedVariables for {}: {}",
                        addon_name,
                        e
                    );
                }
            }
        }

        self.wtf_loaded.insert(addon_name.to_string(), loaded > 0);
        Ok(loaded)
    }

    /// Load a WTF Lua file, executing it to set global variables.
    fn load_wtf_lua_file(&self, lua: &Lua, path: &std::path::Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(mlua::Error::external)?;
        // Strip UTF-8 BOM if present
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

        // Execute the Lua file - it will set global variables
        let chunk_name = format!("@WTF/{}", path.file_name().unwrap_or_default().to_string_lossy());
        lua.load(content).set_name(&chunk_name).exec()?;
        Ok(())
    }

    /// Get the storage path for account-wide saved variables.
    fn account_path(&self, addon_name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.lua", addon_name))
    }

    /// Get the storage path for per-character saved variables.
    fn character_path(&self, addon_name: &str) -> PathBuf {
        self.storage_dir
            .join(&self.realm_name)
            .join(&self.character_name)
            .join(format!("{}.lua", addon_name))
    }

    /// Initialize saved variables for an addon before it loads.
    /// This creates empty tables in Lua globals for each declared variable,
    /// then loads any existing saved data into them.
    pub fn init_for_addon(
        &mut self,
        lua: &Lua,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) -> Result<()> {
        let globals = lua.globals();

        // Initialize account-wide variables
        for var_name in saved_vars {
            // Only initialize if not already set
            if globals.get::<Value>(var_name.as_str())?.is_nil() {
                // Load from storage or create empty table
                let table = self.load_variable(lua, addon_name, var_name, false)?;
                globals.set(var_name.as_str(), table)?;
            }
        }

        // Initialize per-character variables
        for var_name in saved_vars_per_char {
            if globals.get::<Value>(var_name.as_str())?.is_nil() {
                let table = self.load_variable(lua, addon_name, var_name, true)?;
                globals.set(var_name.as_str(), table)?;
            }
        }

        // Track registered variables
        if !saved_vars.is_empty() {
            self.registered
                .insert(addon_name.to_string(), saved_vars.to_vec());
        }
        if !saved_vars_per_char.is_empty() {
            self.registered_per_char
                .insert(addon_name.to_string(), saved_vars_per_char.to_vec());
        }

        Ok(())
    }

    /// Load a single variable from storage by executing the saved .lua file.
    fn load_variable(
        &self,
        lua: &Lua,
        addon_name: &str,
        var_name: &str,
        per_character: bool,
    ) -> Result<Table> {
        let path = if per_character {
            self.character_path(addon_name)
        } else {
            self.account_path(addon_name)
        };

        if path.exists() {
            // Execute the Lua file to set globals, then retrieve the variable
            self.load_wtf_lua_file(lua, &path)?;
            let val: Value = lua.globals().get(var_name)?;
            if let Value::Table(t) = val {
                return Ok(t);
            }
        }

        // No existing data, return empty table
        lua.create_table()
    }

    /// Save all registered variables for an addon in WoW-compatible Lua format.
    pub fn save_addon(&self, lua: &Lua, addon_name: &str) -> Result<()> {
        let globals = lua.globals();

        // Save account-wide variables
        if let Some(vars) = self.registered.get(addon_name) {
            self.write_vars_file(
                &globals,
                vars,
                &self.account_path(addon_name),
            );
        }

        // Save per-character variables
        if let Some(vars) = self.registered_per_char.get(addon_name) {
            self.write_vars_file(
                &globals,
                vars,
                &self.character_path(addon_name),
            );
        }

        let _ = lua;
        Ok(())
    }

    /// Write variable values to a .lua file in WoW SavedVariables format.
    fn write_vars_file(&self, globals: &Table, vars: &[String], path: &PathBuf) {
        let mut output = String::from("\n");
        let mut has_data = false;

        for var_name in vars {
            let val: Value = match globals.get(var_name.as_str()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            serialize_assignment(&mut output, var_name, &val);
            has_data = true;
        }

        if has_data {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(path, output);
        }
    }

    /// Save all registered variables for all addons.
    pub fn save_all(&self, lua: &Lua) -> Result<()> {
        let addon_names: Vec<String> = self
            .registered
            .keys()
            .chain(self.registered_per_char.keys())
            .cloned()
            .collect();

        for addon_name in addon_names {
            self.save_addon(lua, &addon_name)?;
        }
        Ok(())
    }

    /// Get list of registered addons.
    pub fn registered_addons(&self) -> Vec<&String> {
        self.registered
            .keys()
            .chain(self.registered_per_char.keys())
            .collect()
    }
}

impl Default for SavedVariablesManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize a top-level `VarName = value` assignment in WoW SavedVariables format.
fn serialize_assignment(out: &mut String, name: &str, value: &Value) {
    let _ = write!(out, "{} = ", name);
    serialize_value(out, value, 0);
    out.push('\n');
}

/// Serialize a Lua value to WoW SavedVariables format.
fn serialize_value(out: &mut String, value: &Value, depth: usize) {
    match value {
        Value::Nil => out.push_str("nil"),
        Value::Boolean(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Integer(i) => {
            let _ = write!(out, "{}", i);
        }
        Value::Number(n) => {
            // Match WoW's format: use integer notation when possible
            if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
                let _ = write!(out, "{}", *n as i64);
            } else {
                let _ = write!(out, "{}", n);
            }
        }
        Value::String(s) => {
            let Ok(s) = s.to_str() else { out.push_str("\"\""); return };
            out.push('"');
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\0' => out.push_str("\\0"),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        Value::Table(t) => serialize_table(out, t, depth),
        // Functions, userdata, threads etc. are not serializable
        _ => out.push_str("nil"),
    }
}

/// Collect non-array entries from a table, sorted by key for deterministic output.
fn collect_hash_entries(table: &Table, array_len: usize) -> Vec<(String, Value)> {
    let mut entries = Vec::new();
    let Ok(pairs) = table.clone().pairs::<Value, Value>().collect::<std::result::Result<Vec<_>, _>>() else {
        return entries;
    };
    for (k, v) in pairs {
        match &k {
            Value::Integer(i) if *i >= 1 && *i <= array_len as i64 => continue,
            Value::String(s) => {
                if let Ok(key) = s.to_str() {
                    entries.push((key.to_string(), v));
                }
            }
            Value::Integer(i) => entries.push((i.to_string(), v)),
            Value::Number(n) => entries.push((n.to_string(), v)),
            _ => {}
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

/// Write a Lua-escaped key string (for `["key"]` syntax).
fn write_escaped_key(out: &mut String, key: &str) {
    for ch in key.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
}

/// Serialize a Lua table in WoW SavedVariables format.
///
/// WoW uses a specific format:
/// - Array entries (sequential integer keys 1..N) are written without explicit keys
/// - String/other keys use `["key"] = value` syntax
/// - Tables are indented with tabs
fn serialize_table(out: &mut String, table: &Table, depth: usize) {
    out.push_str("{\n");
    let indent = "\t".repeat(depth + 1);
    let array_len = table.raw_len();

    for i in 1..=array_len {
        let val: Value = match table.get(i as i64) {
            Ok(v) => v,
            Err(_) => break,
        };
        if val.is_nil() {
            break;
        }
        let _ = write!(out, "{}", indent);
        serialize_value(out, &val, depth + 1);
        let _ = writeln!(out, ", -- [{}]", i);
    }

    for (key, val) in &collect_hash_entries(table, array_len) {
        let _ = write!(out, "{}[\"", indent);
        write_escaped_key(out, key);
        out.push_str("\"] = ");
        serialize_value(out, val, depth + 1);
        out.push_str(",\n");
    }

    let _ = write!(out, "{}}}", "\t".repeat(depth));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_empty_variables() {
        let lua = Lua::new();
        let dir = tempdir().unwrap();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        // Variable should exist as empty table
        let globals = lua.globals();
        let db: Table = globals.get("TestDB").unwrap();
        assert!(db.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();

        // First session: save some data
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
                .unwrap();

            // Set some values
            lua.load(r#"TestDB.setting1 = "hello"; TestDB.setting2 = 42"#)
                .exec()
                .unwrap();

            mgr.save_addon(&lua, "TestAddon").unwrap();
        }

        // Second session: load the data
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
                .unwrap();

            // Values should be restored
            let val1: String = lua.load("return TestDB.setting1").eval().unwrap();
            let val2: i64 = lua.load("return TestDB.setting2").eval().unwrap();

            assert_eq!(val1, "hello");
            assert_eq!(val2, 42);
        }
    }

    #[test]
    fn test_save_produces_lua_format() {
        let dir = tempdir().unwrap();
        let lua = Lua::new();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        lua.load(r#"TestDB.name = "Haky"; TestDB.level = 70; TestDB.active = true"#)
            .exec()
            .unwrap();

        mgr.save_addon(&lua, "TestAddon").unwrap();

        let content = fs::read_to_string(dir.path().join("TestAddon.lua")).unwrap();
        assert!(content.contains("TestDB = {"), "should have Lua assignment");
        assert!(content.contains("[\"name\"] = \"Haky\""), "should have string value");
        assert!(content.contains("[\"level\"] = 70"), "should have integer value");
        assert!(content.contains("[\"active\"] = true"), "should have boolean value");
    }

    #[test]
    fn test_save_nested_tables() {
        let dir = tempdir().unwrap();
        let lua = Lua::new();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        lua.load(r#"
            TestDB.nested = { a = 1, b = { c = "deep" } }
            TestDB.list = { 10, 20, 30 }
        "#)
            .exec()
            .unwrap();

        mgr.save_addon(&lua, "TestAddon").unwrap();

        // Verify round-trip
        let lua2 = Lua::new();
        let mut mgr2 = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        mgr2.init_for_addon(&lua2, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        let deep: String = lua2.load("return TestDB.nested.b.c").eval().unwrap();
        assert_eq!(deep, "deep");

        let second: i64 = lua2.load("return TestDB.list[2]").eval().unwrap();
        assert_eq!(second, 20);

        let len: i64 = lua2.load("return #TestDB.list").eval().unwrap();
        assert_eq!(len, 3);
    }

    #[test]
    fn test_save_string_escaping() {
        let dir = tempdir().unwrap();
        let lua = Lua::new();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        mgr.init_for_addon(&lua, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        lua.load(r#"TestDB.msg = "line1\nline2"; TestDB.path = "C:\\Users\\test""#)
            .exec()
            .unwrap();

        mgr.save_addon(&lua, "TestAddon").unwrap();

        // Round-trip
        let lua2 = Lua::new();
        let mut mgr2 = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        mgr2.init_for_addon(&lua2, "TestAddon", &["TestDB".to_string()], &[])
            .unwrap();

        let msg: String = lua2.load("return TestDB.msg").eval().unwrap();
        assert_eq!(msg, "line1\nline2");

        let path: String = lua2.load("return TestDB.path").eval().unwrap();
        assert_eq!(path, "C:\\Users\\test");
    }

    #[test]
    fn test_per_character_variables() {
        let dir = tempdir().unwrap();

        // Save for one character
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Thrall", "Hyjal");

            mgr.init_for_addon(&lua, "TestAddon", &[], &["CharDB".to_string()])
                .unwrap();

            lua.load("CharDB.level = 70").exec().unwrap();
            mgr.save_addon(&lua, "TestAddon").unwrap();
        }

        // Load for same character
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Thrall", "Hyjal");

            mgr.init_for_addon(&lua, "TestAddon", &[], &["CharDB".to_string()])
                .unwrap();

            let level: i64 = lua.load("return CharDB.level").eval().unwrap();
            assert_eq!(level, 70);
        }

        // Different character should have empty data
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Jaina", "Hyjal");

            mgr.init_for_addon(&lua, "TestAddon", &[], &["CharDB".to_string()])
                .unwrap();

            let level: Value = lua.load("return CharDB.level").eval().unwrap();
            assert!(level.is_nil());
        }
    }

    #[test]
    fn test_multiple_variables_per_addon() {
        let dir = tempdir().unwrap();

        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            mgr.init_for_addon(
                &lua,
                "Angleur",
                &["AngleurConfig".to_string(), "AngleurMinimapButton".to_string()],
                &["AngleurCharacter".to_string()],
            )
            .unwrap();

            lua.load(r#"
                AngleurConfig.method = "oneKey"
                AngleurMinimapButton.hide = true
                AngleurCharacter.sleeping = false
            "#)
            .exec()
            .unwrap();

            mgr.save_addon(&lua, "Angleur").unwrap();
        }

        // Round-trip
        {
            let lua = Lua::new();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            mgr.init_for_addon(
                &lua,
                "Angleur",
                &["AngleurConfig".to_string(), "AngleurMinimapButton".to_string()],
                &["AngleurCharacter".to_string()],
            )
            .unwrap();

            let method: String = lua.load("return AngleurConfig.method").eval().unwrap();
            assert_eq!(method, "oneKey");

            let hide: bool = lua.load("return AngleurMinimapButton.hide").eval().unwrap();
            assert!(hide);

            let sleeping: bool = lua.load("return AngleurCharacter.sleeping").eval().unwrap();
            assert!(!sleeping);
        }
    }

    #[test]
    fn test_serialize_format_matches_wow() {
        // Verify the output format matches what WoW produces
        let lua = Lua::new();
        lua.load(r#"
            TestVar = {
                ["setting"] = "hello",
                ["items"] = { 10, 20, 30 },
            }
        "#)
        .exec()
        .unwrap();

        let val: Value = lua.globals().get("TestVar").unwrap();
        let mut output = String::new();
        serialize_assignment(&mut output, "TestVar", &val);

        // Should start with assignment
        assert!(output.starts_with("TestVar = {"));
        // Array items should use `-- [N]` comments
        assert!(output.contains("-- [1]"));
        assert!(output.contains("-- [2]"));
        // String keys should use ["key"] syntax
        assert!(output.contains("[\"setting\"] = \"hello\""));
    }
}
