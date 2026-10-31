//! Saved variables management for addon persistence.
//!
//! WoW addons can declare SavedVariables and SavedVariablesPerCharacter in
//! their `.toc` files. These are global Lua tables that persist between
//! sessions.
//!
//! Storage uses WoW-compatible Lua format (`VarName = { ... }`), so files can
//! be shared between the simulator and a real WoW installation.
//!
//! Loading priority:
//! 1. WTF directory (real WoW installation, if configured)
//! 2. Simulator storage (`~/.local/share/wow-sim/SavedVariables/`)

#[cfg(test)]
#[path = "saved_variables_details_tests.rs"]
mod saved_variables_details_tests;
#[cfg(test)]
#[path = "saved_variables_load_tests.rs"]
mod saved_variables_load_tests;
#[cfg(test)]
#[path = "saved_variables_model_tests.rs"]
mod saved_variables_model_tests;
#[path = "saved_variables_parse.rs"]
mod saved_variables_parse;
#[path = "saved_variables_serialize.rs"]
mod saved_variables_serialize;
#[path = "saved_variables_table_size_cache.rs"]
mod saved_variables_table_size_cache;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::lua_api::methods::{call_function_state, create_string, table_get_static};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, Val};

const EDIT_MODE_LAYOUT_ENV: &str = "WOW_SIM_EDIT_MODE_LAYOUT";
use saved_variables_parse::parse_saved_variables_file_with_cache;
use saved_variables_serialize::serialize_assignment;
use saved_variables_table_size_cache::{load_table_size_cache, save_table_size_cache};

const DEFAULT_MAX_SAVED_VARIABLE_BYTES: u64 = 2 * 1024 * 1024;

/// Read-only source for importing WTF saved variables from a real WoW installation.
///
/// Writes always go to `SavedVariablesManager::storage_dir`, never back to this
/// path.
#[derive(Debug, Clone)]
pub struct WtfConfig {
    pub wtf_path: PathBuf,
    pub account: String,
    pub realm: String,
    pub character: String,
}

impl WtfConfig {
    pub fn new(wtf_path: impl Into<PathBuf>, account: &str, realm: &str, character: &str) -> Self {
        Self {
            wtf_path: wtf_path.into(),
            account: account.to_string(),
            realm: realm.to_string(),
            character: character.to_string(),
        }
    }

    pub fn account_saved_vars_path(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join("SavedVariables")
    }

    pub fn character_saved_vars_path(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join(&self.realm)
            .join(&self.character)
            .join("SavedVariables")
    }

    pub fn account_saved_vars_file(&self, addon_name: &str) -> PathBuf {
        self.account_saved_vars_path()
            .join(format!("{}.lua", addon_name))
    }

    pub fn account_bindings_cache_file(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join("bindings-cache.wtf")
    }

    pub fn character_click_bindings_cache_file(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join(&self.realm)
            .join(&self.character)
            .join("click-bindings-cache.txt")
    }

    pub fn character_saved_vars_file(&self, addon_name: &str) -> PathBuf {
        self.character_saved_vars_path()
            .join(format!("{}.lua", addon_name))
    }

    pub fn edit_mode_account_cache_file(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join("edit-mode-cache-account.txt")
    }

    pub fn edit_mode_character_cache_file(&self) -> PathBuf {
        self.wtf_path
            .join("Account")
            .join(&self.account)
            .join(&self.realm)
            .join(&self.character)
            .join("edit-mode-cache-character.txt")
    }
}

/// Result of resolving one addon's account or character input in this VM.
#[derive(Debug)]
enum LoadResult {
    Loaded,
    MissingImport,
    Failed(String),
}

/// Manages saved variables for all loaded addons.
#[derive(Debug)]
pub struct SavedVariablesManager {
    storage_dir: PathBuf,
    character_name: String,
    realm_name: String,
    registered: HashMap<String, Vec<String>>,
    registered_per_char: HashMap<String, Vec<String>>,
    wtf_config: Option<WtfConfig>,
    loaded_values: HashMap<String, Vec<(String, Val)>>,
    reload_saved_addons: std::collections::HashSet<String>,
    max_load_bytes: Option<u64>,
    load_results: HashMap<(String, bool), LoadResult>,
}

impl SavedVariablesManager {
    pub fn new() -> Self {
        let storage_dir = std::env::var_os("WOW_SIM_STORAGE_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("wow-sim")
                    .join("SavedVariables")
            });
        Self::with_storage_dir(storage_dir)
    }

    pub fn with_storage_dir(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            character_name: "SimPlayer".to_string(),
            realm_name: "SimRealm".to_string(),
            registered: HashMap::new(),
            registered_per_char: HashMap::new(),
            wtf_config: None,
            loaded_values: HashMap::new(),
            reload_saved_addons: std::collections::HashSet::new(),
            max_load_bytes: max_load_bytes_from_env(),
            load_results: HashMap::new(),
        }
    }

    /// Restore a simulator-owned model using the SavedVariables data parser only.
    pub(crate) fn load_model(&self, state: &mut LuaState, name: &str) -> crate::Result<Val> {
        let path = self.storage_path(name, false);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                verify_missing_file(&path)?;
                return Ok(Val::Nil);
            }
            Err(error) => {
                return Err(crate::Error::Other(format!(
                    "read model {}: {error}",
                    path.display()
                )));
            }
        };
        saved_variables_parse::parse_model_value(state, &content).map_err(|error| {
            crate::Error::Other(format!("parse model {}: {error}", path.display()))
        })
    }

    /// Persist a private model table in the existing SavedVariables format/directory.
    pub(crate) fn save_model(
        &self,
        state: &mut LuaState,
        name: &str,
        value: Val,
    ) -> crate::Result<()> {
        let path = self.storage_path(name, false);
        let mut output = String::new();
        serialize_assignment(&mut output, state, "preferences", value)?;
        fs::create_dir_all(&self.storage_dir).map_err(|error| {
            crate::Error::Other(format!(
                "create model directory {}: {error}",
                self.storage_dir.display()
            ))
        })?;
        fs::write(&path, output).map_err(|error| {
            crate::Error::Other(format!("write model {}: {error}", path.display()))
        })
    }

    /// Forget VM handles and declarations; the fresh addon graph registers them again.
    pub(crate) fn retire_lua_environment(&mut self) {
        self.load_results.clear();
        self.reload_saved_addons
            .extend(self.registered.keys().cloned());
        self.reload_saved_addons
            .extend(self.registered_per_char.keys().cloned());
        self.registered.clear();
        self.registered_per_char.clear();
        self.loaded_values.clear();
    }

    #[cfg(feature = "client-wrath")]
    pub(crate) fn character_storage_dir(&self) -> PathBuf {
        self.storage_dir
            .join(&self.realm_name)
            .join(&self.character_name)
    }

    pub fn set_character(&mut self, name: &str, realm: &str) {
        self.character_name = name.to_string();
        self.realm_name = realm.to_string();
    }

    pub fn set_wtf_config(&mut self, config: WtfConfig) {
        self.character_name = config.character.clone();
        self.realm_name = config.realm.clone();
        self.wtf_config = Some(config);
    }

    pub fn wtf_config(&self) -> Option<&WtfConfig> {
        self.wtf_config.as_ref()
    }

    pub fn set_max_load_bytes(&mut self, max_load_bytes: Option<u64>) {
        self.max_load_bytes = max_load_bytes;
    }

    /// Load WTF saved variables for an addon from a real WoW installation.
    ///
    /// Returns the number of files loaded (0, 1, or 2 for account + character).
    pub fn load_wtf_for_addon(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
    ) -> crate::Result<usize> {
        if self.reload_saved_addons.contains(addon_name) {
            return Ok(0);
        }
        let Some(config) = self.wtf_config.clone() else {
            return Ok(0);
        };
        let mut loaded = 0;
        let mut failures = Vec::new();
        for (per_character, path) in [
            (false, config.account_saved_vars_file(addon_name)),
            (true, config.character_saved_vars_file(addon_name)),
        ] {
            match self
                .load_results
                .get(&(addon_name.to_owned(), per_character))
            {
                Some(LoadResult::Failed(error)) => {
                    failures.push(error.clone());
                    continue;
                }
                Some(_) => continue,
                None => {}
            }
            match self.load_scope_file(state, addon_name, per_character, &path, true, false) {
                Ok(true) => loaded += 1,
                Ok(false) => {}
                Err(error) => failures.push(error.to_string()),
            }
        }
        if failures.is_empty() {
            Ok(loaded)
        } else {
            Err(crate::Error::Other(failures.join("; ")))
        }
    }

    /// Load the EditMode layout cache and select the active layout.
    ///
    /// The active layout is chosen with this priority:
    /// 1. `WOW_SIM_EDIT_MODE_LAYOUT` env var (explicit manual override)
    /// 2. `snapshot_layout` (the active layout name captured by ServerSnapshot
    ///    from a live client — see `server_snapshot_import`)
    /// 3. the active-layout index recorded in the WTF character cache
    pub fn load_edit_mode_cache(
        &self,
        state: &mut LuaState,
        active_spec_index: i32,
        snapshot_layout: Option<&str>,
    ) -> crate::Result<bool> {
        let Some(config) = self.wtf_config.as_ref() else {
            return Ok(false);
        };

        let account_cache = read_optional_file(&config.edit_mode_account_cache_file())?;
        let character_cache = read_optional_file(&config.edit_mode_character_cache_file())?;
        if account_cache.is_none() && character_cache.is_none() {
            return Ok(false);
        }

        let c_edit_mode = LuaApiMut::get_global_val(state, "C_EditMode");
        let load_cache = table_get_static(state, c_edit_mode, "__LoadCache");
        let account_arg = optional_string_arg(state, account_cache.as_deref());
        let character_arg = optional_string_arg(state, character_cache.as_deref());
        let preferred_layout = std::env::var(EDIT_MODE_LAYOUT_ENV)
            .ok()
            .or_else(|| snapshot_layout.map(str::to_string));
        let preferred_layout_arg = optional_string_arg(state, preferred_layout.as_deref());
        let missing_layouts = call_function_state(
            state,
            load_cache,
            &[
                account_arg,
                character_arg,
                Val::Num(active_spec_index as f64),
                preferred_layout_arg,
            ],
        )?;
        if matches!(missing_layouts, Val::Bool(true)) {
            crate::logging::eprintln_elapsed(
                "Warning: EditMode account cache declares layout rows that are missing",
            );
        }
        Ok(true)
    }

    /// Initialize saved variables for an addon before it loads.
    pub fn init_for_addon(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) -> crate::Result<()> {
        let result = self.load_declared_scopes(state, addon_name, saved_vars, saved_vars_per_char);
        self.remember_loaded_values(state, addon_name, saved_vars, saved_vars_per_char);
        result
    }

    /// Wrath executes each declared scope once after addon code, overwriting defaults.
    pub(crate) fn load_after_addon_code(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) -> crate::Result<()> {
        self.load_declared_scopes(state, addon_name, saved_vars, saved_vars_per_char)
    }

    fn load_declared_scopes(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) -> crate::Result<()> {
        self.remember_registered_vars(addon_name, saved_vars, saved_vars_per_char);
        let mut failures = Vec::new();
        for (per_character, names) in [(false, saved_vars), (true, saved_vars_per_char)] {
            if !names.is_empty()
                && let Err(error) = self.load_registered_scope(state, addon_name, per_character)
            {
                failures.push(error.to_string());
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::Other(failures.join("; ")))
        }
    }

    /// Restore saved-variable globals that addon top-level code clobbered with
    /// an empty table after the persisted value had already loaded.
    pub fn restore_clobbered_globals(&mut self, state: &mut LuaState, addon_name: &str) -> usize {
        let Some(values) = self.loaded_values.get(addon_name).cloned() else {
            return 0;
        };

        let mut restored = 0;
        for (name, loaded_value) in values {
            let current_value = get_global(state, &name);
            if should_restore_clobbered_value(state, loaded_value, current_value) {
                set_global(state, &name, loaded_value);
                restored += 1;
            }
        }
        restored
    }

    /// Seed declared SavedVariables globals with empty tables without touching
    /// on-disk storage. This keeps addon startup behavior deterministic even
    /// when persistence is disabled via `--no-saved-vars`.
    pub fn seed_declared_globals(
        state: &mut LuaState,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) {
        seed_missing_globals(state, saved_vars);
        seed_missing_globals(state, saved_vars_per_char);
    }

    /// Save all registered variables for an addon in WoW-compatible Lua format.
    pub fn save_addon(&self, state: &mut LuaState, addon_name: &str) -> crate::Result<()> {
        self.write_registered_file(state, addon_name, false)?;
        self.write_registered_file(state, addon_name, true)?;
        Ok(())
    }

    /// Save all registered variables for all addons.
    pub fn save_all(&self, state: &mut LuaState) -> crate::Result<()> {
        let addon_names: Vec<String> = self
            .registered
            .keys()
            .chain(self.registered_per_char.keys())
            .cloned()
            .collect();
        for addon_name in addon_names {
            self.save_addon(state, &addon_name)?;
        }
        Ok(())
    }

    pub fn registered_addons(&self) -> Vec<&String> {
        self.registered
            .keys()
            .chain(self.registered_per_char.keys())
            .collect()
    }

    fn remember_registered_vars(
        &mut self,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) {
        if !saved_vars.is_empty() {
            self.registered
                .insert(addon_name.to_string(), saved_vars.to_vec());
        }
        if !saved_vars_per_char.is_empty() {
            self.registered_per_char
                .insert(addon_name.to_string(), saved_vars_per_char.to_vec());
        }
    }

    fn remember_loaded_values(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        saved_vars: &[String],
        saved_vars_per_char: &[String],
    ) {
        let values = saved_vars
            .iter()
            .chain(saved_vars_per_char.iter())
            .map(|name| (name.clone(), get_global(state, name)))
            .collect();
        self.loaded_values.insert(addon_name.to_string(), values);
    }

    fn load_registered_scope(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        per_character: bool,
    ) -> crate::Result<()> {
        match self
            .load_results
            .get(&(addon_name.to_owned(), per_character))
        {
            Some(LoadResult::Loaded) => return Ok(()),
            Some(LoadResult::Failed(error)) => return Err(crate::Error::Other(error.clone())),
            Some(LoadResult::MissingImport) | None => {}
        }
        let reload = self.reload_saved_addons.contains(addon_name);
        let local_path = self.storage_path(addon_name, per_character);
        if reload {
            return self
                .load_scope_file(state, addon_name, per_character, &local_path, false, true)
                .map(|_| ());
        }
        if let Some(config) = self.wtf_config.as_ref()
            && !self
                .load_results
                .contains_key(&(addon_name.to_owned(), per_character))
        {
            let path = if per_character {
                config.character_saved_vars_file(addon_name)
            } else {
                config.account_saved_vars_file(addon_name)
            };
            if self.load_scope_file(state, addon_name, per_character, &path, true, false)? {
                return Ok(());
            }
        }
        self.load_scope_file(state, addon_name, per_character, &local_path, false, false)
            .map(|_| ())
    }

    fn load_scope_file(
        &mut self,
        state: &mut LuaState,
        addon_name: &str,
        per_character: bool,
        path: &Path,
        wtf_input: bool,
        reload: bool,
    ) -> crate::Result<bool> {
        let loaded = (|| -> crate::Result<bool> {
            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    // A dangling link is failed input, not an absent file.
                    Ok(fs::metadata(path).map_err(|error| {
                        crate::Error::Other(format!(
                            "Resolve SavedVariables link {}: {error}",
                            path.display()
                        ))
                    })?)
                }
                result => result,
            };
            match metadata {
                Ok(metadata) if !metadata.file_type().is_file() => {
                    return Err(crate::Error::Other(format!(
                        "SavedVariables input is not a regular file: {}",
                        path.display()
                    )));
                }
                Ok(metadata)
                    if !reload
                        && self
                            .max_load_bytes
                            .is_some_and(|limit| metadata.len() > limit) =>
                {
                    return Err(crate::Error::Other(format!(
                        "SavedVariables input exceeds the configured byte limit: {}",
                        path.display()
                    )));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && !reload => {
                    verify_missing_file(path)?;
                    return Ok(false);
                }
                Err(error) => {
                    return Err(crate::Error::Other(format!(
                        "Read SavedVariables {}: {error}",
                        path.display()
                    )));
                }
            }
            let content = fs::read(path).map_err(|error| {
                crate::Error::Other(format!("Read SavedVariables {}: {error}", path.display()))
            })?;
            if !reload
                && self
                    .max_load_bytes
                    .is_some_and(|limit| content.len() as u64 > limit)
            {
                return Err(crate::Error::Other(format!(
                    "SavedVariables input exceeds the configured byte limit: {}",
                    path.display()
                )));
            }
            self.execute_lua_bytes(
                state,
                path,
                if wtf_input { "@WTF" } else { "@SavedVariables" },
                &content,
            )?;
            Ok(true)
        })();
        let result = match &loaded {
            Ok(false) if wtf_input => LoadResult::MissingImport,
            Ok(_) => LoadResult::Loaded,
            Err(error) => LoadResult::Failed(error.to_string()),
        };
        self.load_results
            .insert((addon_name.to_owned(), per_character), result);
        if wtf_input && matches!(loaded, Ok(true)) {
            crate::lua_api::workarounds::sanitize_imported_wtf_addon_saved_variables(
                state, addon_name,
            );
        }
        loaded
    }

    fn execute_lua_bytes(
        &self,
        state: &mut LuaState,
        path: &Path,
        chunk_prefix: &str,
        content: &[u8],
    ) -> crate::Result<()> {
        let content = content.strip_prefix(b"\xef\xbb\xbf").unwrap_or(content);
        let mut table_size_cache = load_table_size_cache(&self.storage_dir, path);
        // Parse literal data directly when possible; other Lua syntax uses the VM.
        if let Ok(source) = std::str::from_utf8(content)
            && parse_saved_variables_file_with_cache(state, source, Some(&mut table_size_cache))
                .is_ok()
        {
            save_table_size_cache(&self.storage_dir, path, &table_size_cache)?;
            return Ok(());
        }

        let chunk_name = format!(
            "{}/{}",
            chunk_prefix,
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let func = LuaApiMut::load_bytes(state, content, &chunk_name)?;
        call_function_state(state, Val::Function(func.gc_ref()), &[])?;
        Ok(())
    }

    fn write_registered_file(
        &self,
        state: &mut LuaState,
        addon_name: &str,
        per_character: bool,
    ) -> crate::Result<()> {
        let Some(variable_names) = self.registered_vars_for(addon_name, per_character) else {
            return Ok(());
        };
        let path = self.storage_path(addon_name, per_character);
        self.write_vars_file(state, variable_names, &path)
    }

    fn write_vars_file(
        &self,
        state: &mut LuaState,
        vars: &[String],
        path: &Path,
    ) -> crate::Result<()> {
        if vars.is_empty() {
            return Ok(());
        }
        let mut output = String::from("\n");

        for var_name in vars {
            let value = get_global(state, var_name);
            serialize_assignment(&mut output, state, var_name, value)?;
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| crate::Error::Other(e.to_string()))?;
        }
        fs::write(path, output).map_err(|e| crate::Error::Other(e.to_string()))?;
        Ok(())
    }

    fn registered_vars_for(&self, addon_name: &str, per_character: bool) -> Option<&[String]> {
        if per_character {
            return self.registered_per_char.get(addon_name).map(Vec::as_slice);
        }
        self.registered.get(addon_name).map(Vec::as_slice)
    }

    fn storage_path(&self, addon_name: &str, per_character: bool) -> PathBuf {
        if per_character {
            return self
                .storage_dir
                .join(&self.realm_name)
                .join(&self.character_name)
                .join(format!("{}.lua", addon_name));
        }
        self.storage_dir.join(format!("{}.lua", addon_name))
    }
}

impl Default for SavedVariablesManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Some platforms report a non-directory path ancestor as `NotFound`. That is
/// an unreadable input, not verified absence eligible for initializing new data.
fn verify_missing_file(path: &Path) -> crate::Result<()> {
    for ancestor in path.ancestors().skip(1) {
        let ancestor = if ancestor.as_os_str().is_empty() {
            Path::new(".")
        } else {
            ancestor
        };
        match fs::metadata(ancestor) {
            Ok(metadata) if metadata.is_dir() => return Ok(()),
            Ok(_) => {
                return Err(crate::Error::Other(format!(
                    "SavedVariables ancestor is not a directory: {}",
                    ancestor.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(crate::Error::Other(format!(
                    "Inspect SavedVariables ancestor {}: {error}",
                    ancestor.display()
                )));
            }
        }
    }
    Err(crate::Error::Other(format!(
        "Could not verify SavedVariables absence: {}",
        path.display()
    )))
}

fn read_optional_file(path: &Path) -> crate::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| crate::Error::Other(e.to_string()))?;
    Ok(Some(content))
}

fn optional_string_arg(state: &mut LuaState, value: Option<&str>) -> Val {
    value
        .map(|text| create_string(state, text))
        .unwrap_or(Val::Nil)
}

fn get_global(state: &mut LuaState, name: &str) -> Val {
    let key = state.gc.intern_string(name.as_bytes());
    state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn set_global(state: &mut LuaState, name: &str, value: Val) {
    let key = state.gc.intern_string(name.as_bytes());
    if let Some(globals) = state.gc.tables.get_mut(state.global) {
        let _ = globals.raw_set(Val::Str(key), value, &state.gc.string_arena);
    }
}

fn create_empty_table(state: &mut LuaState) -> Val {
    Val::Table(state.gc.alloc_table(Table::new()))
}

fn table_has_entries(state: &LuaState, value: Val) -> bool {
    let Val::Table(table_ref) = value else {
        return false;
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return false;
    };

    table.array_slice().iter().any(|value| !value.is_nil()) || !table.hash_entries().is_empty()
}

fn should_restore_clobbered_value(state: &LuaState, loaded_value: Val, current_value: Val) -> bool {
    // GTFO-style saved variables carry a schema marker and are sometimes
    // reset to `{}` by top-level addon code before VARIABLES_LOADED.
    // Require the marker so huge addon-managed caches are not rehydrated
    // after an intentional empty-table reset.
    table_has_entries(state, loaded_value)
        && table_has_string_key(state, loaded_value, b"DataCode")
        && table_is_empty(state, current_value)
}

fn table_has_string_key(state: &LuaState, value: Val, key: &[u8]) -> bool {
    let Val::Table(table_ref) = value else {
        return false;
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return false;
    };

    table.hash_entries().iter().any(|(entry_key, _)| {
        let Val::Str(string_ref) = entry_key else {
            return false;
        };
        state
            .gc
            .string_arena
            .get(*string_ref)
            .is_some_and(|string| string.data() == key)
    })
}

fn table_is_empty(state: &LuaState, value: Val) -> bool {
    matches!(value, Val::Table(_)) && !table_has_entries(state, value)
}

fn seed_missing_globals(state: &mut LuaState, variable_names: &[String]) {
    for variable_name in variable_names {
        let already_present = !matches!(get_global(state, variable_name), Val::Nil);
        if already_present {
            continue;
        }
        let empty_table = create_empty_table(state);
        set_global(state, variable_name, empty_table);
    }
}

fn max_load_bytes_from_env() -> Option<u64> {
    match std::env::var("WOW_SIM_MAX_SAVED_VARIABLE_BYTES") {
        Ok(value) if value == "0" => None,
        Ok(value) => value.parse().ok(),
        Err(_) => Some(DEFAULT_MAX_SAVED_VARIABLE_BYTES),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::lua_api::WowLuaEnv;
    use crate::lua_api::methods::table_get_static;
    use rilua::LuaApiMut;
    use rilua::Val;

    use super::{SavedVariablesManager, WtfConfig, get_global};

    #[test]
    fn oversized_local_saved_variables_fail_instead_of_counting_as_loaded() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("HugeAddon.lua"),
            "\nHugeDB = { value = 'should not load' }\n",
        )
        .expect("write saved variables");

        let env = WowLuaEnv::new().expect("env");
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        mgr.set_max_load_bytes(Some(8));

        {
            let mut lua = env.rilua_mut();
            let error = mgr
                .init_for_addon(lua.state_mut(), "HugeAddon", &["HugeDB".to_string()], &[])
                .expect_err("oversized input is a load failure");
            assert!(error.to_string().contains("byte limit"));
        }

        let value_type: String = env.eval("return type(HugeDB)").expect("probe should run");
        assert_eq!(value_type, "nil");
    }

    #[test]
    fn wtf_saved_variables_win_over_simulator_storage() {
        let local_dir = tempfile::tempdir().expect("local tempdir");
        let wtf_dir = tempfile::tempdir().expect("wtf tempdir");
        let wtf_account_dir = wtf_dir.path().join("Account/TestAccount/SavedVariables");
        fs::create_dir_all(&wtf_account_dir).expect("create wtf saved variables dir");
        fs::write(
            wtf_account_dir.join("BlizzMove.lua"),
            "\nBlizzMoveDB = { source = 'wtf' }\n",
        )
        .expect("write wtf saved variables");
        fs::write(
            local_dir.path().join("BlizzMove.lua"),
            "\nBlizzMoveDB = { source = 'local' }\n",
        )
        .expect("write local saved variables");

        let env = WowLuaEnv::new().expect("env");
        let mut mgr = SavedVariablesManager::with_storage_dir(local_dir.path().to_path_buf());
        mgr.set_wtf_config(WtfConfig::new(
            wtf_dir.path(),
            "TestAccount",
            "TestRealm",
            "TestCharacter",
        ));
        let saved_vars = ["BlizzMoveDB".to_string()];

        {
            let mut lua = env.rilua_mut();
            let loaded = mgr
                .load_wtf_for_addon(lua.state_mut(), "BlizzMove")
                .expect("wtf load should succeed");
            assert_eq!(loaded, 1);
            mgr.init_for_addon(lua.state_mut(), "BlizzMove", &saved_vars, &[])
                .expect("saved variable init should not fail");
        }

        let source: String = env
            .eval("return BlizzMoveDB.source")
            .expect("probe should run");
        assert_eq!(source, "wtf");
    }

    #[test]
    fn saved_variables_restore_after_addon_top_level_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("GTFO.lua"),
            "\nGTFOData = { [\"DataCode\"] = \"4\", [\"source\"] = \"saved\" }\n",
        )
        .expect("write saved variables");

        let env = WowLuaEnv::new().expect("env");
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        let saved_vars = ["GTFOData".to_string()];

        {
            let mut lua = env.rilua_mut();
            mgr.init_for_addon(lua.state_mut(), "GTFO", &saved_vars, &[])
                .expect("saved variable init should not fail");
        }

        env.exec("GTFOData = {}")
            .expect("addon top-level defaults should run");

        {
            let mut lua = env.rilua_mut();
            let restored = mgr.restore_clobbered_globals(lua.state_mut(), "GTFO");
            assert_eq!(restored, 1);
        }

        let data_code: String = env
            .eval("return GTFOData.DataCode")
            .expect("probe should run");
        let source: String = env
            .eval("return GTFOData.source")
            .expect("probe should run");
        assert_eq!(data_code, "4");
        assert_eq!(source, "saved");
    }

    #[test]
    fn saved_variables_table_size_cache_persists_between_manager_instances() {
        let dir = tempfile::tempdir().expect("tempdir");
        let saved_vars_file = dir.path().join("Large.lua");
        fs::write(
            &saved_vars_file,
            r#"
                TEST_SV = {
                    child = {
                        k1 = true, k2 = true, k3 = true,
                        k4 = true, k5 = true, k6 = true,
                        k7 = true, k8 = true, k9 = true,
                    },
                }
            "#,
        )
        .expect("write saved variables");

        let env = WowLuaEnv::new().expect("env");
        {
            let mut lua = env.rilua_mut();
            let mut manager = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            manager
                .init_for_addon(lua.state_mut(), "Large", &["TEST_SV".to_string()], &[])
                .expect("initial parse should persist table sizes");
        }

        fs::write(
            &saved_vars_file,
            r#"
                TEST_SV = {
                    child = {
                        k1 = true,
                    },
                }
            "#,
        )
        .expect("rewrite saved variables");

        {
            let mut lua = env.rilua_mut();
            let mut manager = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            manager
                .init_for_addon(lua.state_mut(), "Large", &["TEST_SV".to_string()], &[])
                .expect("second parse should use persisted table sizes");

            let state = lua.state_mut();
            let root = get_global(state, "TEST_SV");
            let child = table_get_static(state, root, "child");
            let Val::Table(child_ref) = child else {
                panic!("TEST_SV.child should be a table");
            };
            let child_table = state.gc.tables.get(child_ref).unwrap();
            assert_eq!(child_table.hash_size(), 16);
        }
    }
}
