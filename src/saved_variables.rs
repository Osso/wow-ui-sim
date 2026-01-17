//! Saved variables management for addon persistence.
//!
//! WoW addons can declare SavedVariables and SavedVariablesPerCharacter in their
//! .toc files. These are global Lua tables that persist between sessions.
//!
//! This module supports two modes:
//! 1. JSON storage (default) - for simulator-generated saved variables
//! 2. WTF loading - loads actual WoW SavedVariables from WTF directory

use mlua::{Lua, Result, Table, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
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
            .join("wow-ui-sim")
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
        let content = fs::read_to_string(path).map_err(|e| mlua::Error::external(e))?;
        // Strip UTF-8 BOM if present
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

        // Execute the Lua file - it will set global variables
        let chunk_name = format!("@WTF/{}", path.file_name().unwrap_or_default().to_string_lossy());
        lua.load(content).set_name(&chunk_name).exec()?;
        Ok(())
    }

    /// Get the storage path for account-wide saved variables.
    fn account_path(&self, addon_name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", addon_name))
    }

    /// Get the storage path for per-character saved variables.
    fn character_path(&self, addon_name: &str) -> PathBuf {
        self.storage_dir
            .join(&self.realm_name)
            .join(&self.character_name)
            .join(format!("{}.json", addon_name))
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

    /// Load a single variable from storage.
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

        // Try to load existing data
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<HashMap<String, JsonValue>>(&content) {
                if let Some(var_data) = json.get(var_name) {
                    return json_to_lua_table(lua, var_data);
                }
            }
        }

        // No existing data, return empty table
        lua.create_table()
    }

    /// Save all registered variables for an addon.
    pub fn save_addon(&self, lua: &Lua, addon_name: &str) -> Result<()> {
        let globals = lua.globals();

        // Save account-wide variables
        if let Some(vars) = self.registered.get(addon_name) {
            let mut data: HashMap<String, JsonValue> = HashMap::new();
            for var_name in vars {
                if let Ok(table) = globals.get::<Table>(var_name.as_str()) {
                    if let Ok(json) = lua_table_to_json(&table) {
                        data.insert(var_name.clone(), json);
                    }
                }
            }

            if !data.is_empty() {
                let path = self.account_path(addon_name);
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(json) = serde_json::to_string_pretty(&data) {
                    let _ = fs::write(&path, json);
                }
            }
        }

        // Save per-character variables
        if let Some(vars) = self.registered_per_char.get(addon_name) {
            let mut data: HashMap<String, JsonValue> = HashMap::new();
            for var_name in vars {
                if let Ok(table) = globals.get::<Table>(var_name.as_str()) {
                    if let Ok(json) = lua_table_to_json(&table) {
                        data.insert(var_name.clone(), json);
                    }
                }
            }

            if !data.is_empty() {
                let path = self.character_path(addon_name);
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(json) = serde_json::to_string_pretty(&data) {
                    let _ = fs::write(&path, json);
                }
            }
        }

        Ok(())
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

/// Convert a JSON value to a Lua table.
fn json_to_lua_table(lua: &Lua, json: &JsonValue) -> Result<Table> {
    let table = lua.create_table()?;

    match json {
        JsonValue::Object(obj) => {
            for (k, v) in obj {
                let lua_value = json_to_lua_value(lua, v)?;
                table.set(k.as_str(), lua_value)?;
            }
        }
        JsonValue::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let lua_value = json_to_lua_value(lua, v)?;
                table.set(i + 1, lua_value)?;
            }
        }
        _ => {}
    }

    Ok(table)
}

/// Convert a JSON value to a Lua value.
fn json_to_lua_value(lua: &Lua, json: &JsonValue) -> Result<Value> {
    Ok(match json {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Nil
            }
        }
        JsonValue::String(s) => Value::String(lua.create_string(s)?),
        JsonValue::Array(_) | JsonValue::Object(_) => Value::Table(json_to_lua_table(lua, json)?),
    })
}

/// Convert a Lua table to JSON.
fn lua_table_to_json(table: &Table) -> std::result::Result<JsonValue, String> {
    let mut is_array = true;
    let mut max_index: i64 = 0;

    // First pass: determine if it's an array
    for pair in table.clone().pairs::<Value, Value>() {
        let (k, _) = pair.map_err(|e| e.to_string())?;
        match k {
            Value::Integer(i) if i > 0 => {
                if i > max_index {
                    max_index = i;
                }
            }
            _ => {
                is_array = false;
                break;
            }
        }
    }

    if is_array && max_index > 0 {
        // Convert as array
        let mut arr = Vec::with_capacity(max_index as usize);
        for i in 1..=max_index {
            let v: Value = table.get(i).map_err(|e| e.to_string())?;
            arr.push(lua_value_to_json(&v)?);
        }
        Ok(JsonValue::Array(arr))
    } else {
        // Convert as object
        let mut obj = serde_json::Map::new();
        for pair in table.clone().pairs::<Value, Value>() {
            let (k, v) = pair.map_err(|e| e.to_string())?;
            let key = match k {
                Value::String(s) => s.to_str().map_err(|e| e.to_string())?.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Number(n) => n.to_string(),
                _ => continue, // Skip non-string/number keys
            };
            obj.insert(key, lua_value_to_json(&v)?);
        }
        Ok(JsonValue::Object(obj))
    }
}

/// Convert a Lua value to JSON.
fn lua_value_to_json(value: &Value) -> std::result::Result<JsonValue, String> {
    Ok(match value {
        Value::Nil => JsonValue::Null,
        Value::Boolean(b) => JsonValue::Bool(*b),
        Value::Integer(i) => JsonValue::Number((*i).into()),
        Value::Number(n) => serde_json::Number::from_f64(*n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(s) => JsonValue::String(s.to_str().map_err(|e| e.to_string())?.to_string()),
        Value::Table(t) => lua_table_to_json(t)?,
        // Skip functions, userdata, threads, etc.
        _ => JsonValue::Null,
    })
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
}
