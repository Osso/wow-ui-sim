use super::helpers::set_global_val;
use crate::addon_enable_state::save_local_addon_enable_overrides;
use crate::lua_api::AddonEnableSnapshot;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, registry_get, registry_set,
    val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::{LuaResult, Val, vm::state::LuaState};
use std::path::{Path, PathBuf};

#[path = "c_addons_missing.rs"]
mod c_addons_missing;
#[path = "c_addons_registration.rs"]
mod c_addons_registration;
#[path = "c_addons_runtime.rs"]
mod c_addons_runtime;

const ADDON_VERSION_CHECK_KEY: &str = "__addon_version_check_enabled";
pub(super) const GAME_RUNTIME_FOUNDATIONS: &[&str] = &[
    // Blizzard_SharedXMLBase's TOC depends on Blizzard_ScriptErrors; unless it
    // loads first, the recursive dependency walk re-enters the foundation
    // chain mid-SharedXMLBase and runs Blizzard_SharedXML before
    // SharedXMLBase's own files (FlagsUtil, MathUtil, CallbackRegistry).
    "Blizzard_ScriptErrors",
    "Blizzard_SharedXMLBase",
    "Blizzard_Menu",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_FrameXMLBase",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLUtil",
];
pub(super) const GLUE_RUNTIME_FOUNDATIONS: &[&str] = &[
    "Blizzard_GlueXMLBase",
    "Blizzard_GlueParent",
    "Blizzard_GlueMenuFrame",
    "Blizzard_GlueXML",
];

// ── C_AddOns registration ─────────────────────────────────────────────────────

pub fn register_c_addons(state: &mut LuaState) -> LuaResult<()> {
    let c_addons = create_table(state);
    let Val::Table(c_addons_ref) = c_addons else {
        unreachable!("create_table must return a table");
    };
    c_addons_registration::register_c_addons_methods(state, c_addons_ref)?;
    set_global_val(state, "C_AddOns", c_addons);
    Ok(())
}

pub fn register_legacy_addon_globals(state: &mut LuaState) -> LuaResult<()> {
    register_legacy_addon_fns(state)?;
    register_addon_actions_blocked(state);
    Ok(())
}

fn register_legacy_addon_fns(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetNumAddOns", c_addons_get_num_addons)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "IsAddOnLoaded",
        c_addons_is_addon_loaded,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetAddOnMetadata",
        c_addons_get_addon_metadata,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetAddOnEnableState",
        legacy_get_addon_enable_state,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "IsAddOnLoadOnDemand",
        c_addons_is_addon_load_on_demand,
    )?;
    table_set_rust_fn_static(state, state.global, "LoadAddOn", c_addons_load_addon)?;
    Ok(())
}

fn register_addon_actions_blocked(state: &mut LuaState) {
    let blocked = create_table(state);
    let key_ref = state.gc.intern_string(b"ADDON_ACTIONS_BLOCKED");
    let global_ref = state.global;
    if let Some(global) = state.gc.tables.get_mut(global_ref) {
        let _ = global.raw_set(Val::Str(key_ref), blocked, &state.gc.string_arena);
    }
    state.gc.barrier_back(global_ref);
}

// ── Addon query helpers ───────────────────────────────────────────────────────

fn addon_index_from_value(state: &LuaState, addon: Val) -> Option<usize> {
    match addon {
        Val::Num(index) if index.is_finite() && index.fract() == 0.0 && index >= 1.0 => {
            Some(index as usize - 1)
        }
        Val::Str(_) => {
            let name = val_to_string(state, addon)?;
            let sim = borrow_state(state).ok()?;
            sim.addons.iter().position(|a| a.folder_name == name)
        }
        _ => None,
    }
}

fn addon_name_from_value(state: &LuaState, addon: Val) -> Option<String> {
    match addon {
        Val::Str(_) => val_to_string(state, addon),
        other => {
            let index = addon_index_from_value(state, other)?;
            let sim = borrow_state(state).ok()?;
            sim.addons.get(index).map(|a| a.folder_name.clone())
        }
    }
}

fn with_addon<R>(
    state: &LuaState,
    addon: Val,
    f: impl FnOnce(&crate::lua_api::AddonInfo) -> R,
) -> Option<R> {
    let index = addon_index_from_value(state, addon)?;
    let sim = borrow_state(state).ok()?;
    sim.addons.get(index).map(f)
}

fn addon_metadata(addon: &crate::lua_api::AddonInfo, field: &str) -> Option<String> {
    if field.eq_ignore_ascii_case("Group") {
        return Some(addon.folder_name.clone());
    }
    if field.eq_ignore_ascii_case("Title") {
        return Some(addon.title.clone());
    }
    if field.eq_ignore_ascii_case("Notes") {
        return (!addon.notes.is_empty()).then(|| addon.notes.clone());
    }
    if field.eq_ignore_ascii_case("Version") {
        let version = find_addon_metadata(addon, field);
        return version.or_else(|| Some("@project-version@".to_string()));
    }
    find_addon_metadata(addon, field)
}

fn find_addon_metadata(addon: &crate::lua_api::AddonInfo, field: &str) -> Option<String> {
    addon
        .metadata
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(field))
        .map(|(_, value)| value.clone())
}

fn push_addon_info(state: &mut LuaState, addon: &crate::lua_api::AddonInfo) -> u32 {
    let folder_name = create_string(state, &addon.folder_name);
    let title = create_string(state, &addon.title);
    let notes = (!addon.notes.is_empty()).then(|| create_string(state, &addon.notes));
    let reason = (!addon.enabled).then(|| create_string(state, "DISABLED"));
    let security = addon_security_status(addon);
    let security = create_string(state, &security);
    state.push(folder_name);
    state.push(title);
    if addon.notes.is_empty() {
        state.push(Val::Nil);
    } else {
        state.push(notes.unwrap_or(Val::Nil));
    }
    state.push(Val::Bool(addon.enabled));
    state.push(reason.unwrap_or(Val::Nil));
    state.push(security);
    6
}

fn registry_bool(state: &mut LuaState, key: &'static str) -> bool {
    matches!(registry_get(state, key), Val::Bool(true))
}

fn set_registry_bool(state: &mut LuaState, key: &'static str, value: bool) {
    registry_set(state, key, Val::Bool(value));
}

// ── Addon existence / TOC ─────────────────────────────────────────────────────

fn default_runtime_addon_bases() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![
        crate::client_profile::blizzard_ui_addons_dir_under(&root),
        root.join("Interface/AddOns"),
        root.join("Interface/TestAddOns"),
    ]
}

pub(super) fn find_runtime_addon_toc(state: &LuaState, addon_name: &str) -> Option<PathBuf> {
    if crate::loader::is_addon_excluded_for_active_profile(addon_name) {
        return None;
    }

    let (bases, screen_kind) = {
        let sim = borrow_state(state).ok()?;
        (
            if sim.addon_base_paths.is_empty() {
                default_runtime_addon_bases()
            } else {
                sim.addon_base_paths.clone()
            },
            sim.screen_kind,
        )
    };
    for base in bases {
        let addon_dir = base.join(addon_name);
        if let Some(toc_path) = crate::loader::find_toc_file(&addon_dir)
            && runtime_toc_allowed_on_current_screen(&toc_path, screen_kind)
        {
            return Some(toc_path);
        }
    }
    None
}

fn runtime_toc_allowed_on_current_screen(
    toc_path: &Path,
    screen_kind: crate::screen::ScreenKind,
) -> bool {
    let Ok(toc) = crate::toc::TocFile::from_file(toc_path) else {
        return false;
    };
    toc.allows_screen(screen_kind) && !toc.is_ptr_only() && !toc.is_game_type_restricted()
}

fn addon_exists(state: &LuaState, addon_name: &str) -> bool {
    let registered = {
        let sim = match borrow_state(state) {
            Ok(sim) => sim,
            Err(_) => return false,
        };
        sim.addons.iter().any(|a| a.folder_name == addon_name)
    };
    registered || find_runtime_addon_toc(state, addon_name).is_some()
}

// ── C_AddOns function implementations ────────────────────────────────────────

fn c_addons_get_num_addons(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.addons.len() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

fn c_addons_get_addon_info(state: &mut LuaState) -> LuaResult<u32> {
    let addon = stack_val(state, 1);
    let Some(index) = addon_index_from_value(state, addon) else {
        if let Val::Str(_) = addon
            && let Some(addon_name) = val_to_string(state, addon)
            && addon_name.starts_with("Blizzard_")
            && !addon_exists(state, &addon_name)
        {
            return Ok(c_addons_missing::push_missing_addon_info(
                state,
                &addon_name,
            ));
        }
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = {
        let sim = borrow_state(state)?;
        sim.addons.get(index).cloned()
    };
    let Some(info) = info else {
        state.push(Val::Nil);
        return Ok(1);
    };
    Ok(push_addon_info(state, &info))
}

fn c_addons_is_addon_loaded(state: &mut LuaState) -> LuaResult<u32> {
    let addon_name = addon_name_from_value(state, stack_val(state, 1)).unwrap_or_default();
    let (loaded_or_loading, loaded) = if addon_name.is_empty() {
        (false, false)
    } else {
        let sim = borrow_state(state)?;
        let loaded = sim
            .addons
            .iter()
            .find(|addon| addon.folder_name == addon_name)
            .map(|addon| addon.loaded)
            .unwrap_or(false);
        let loading = sim.is_addon_loading(&addon_name);
        (loaded || loading, loaded)
    };
    state.push(Val::Bool(loaded_or_loading));
    state.push(Val::Bool(loaded));
    Ok(2)
}

fn c_addons_is_addon_loadable(state: &mut LuaState) -> LuaResult<u32> {
    let addon = stack_val(state, 1);
    let Some(addon_name) = addon_name_from_value(state, addon) else {
        state.push(Val::Bool(false));
        state.push(Val::Nil);
        return Ok(2);
    };

    let reason = if !addon_exists(state, &addon_name) {
        Some("MISSING")
    } else if addon_is_disabled(state, &addon_name) {
        Some("DISABLED")
    } else {
        None
    };

    state.push(Val::Bool(reason.is_none()));
    match reason {
        Some(reason) => {
            let reason = create_string(state, reason);
            state.push(reason);
        }
        None => state.push(Val::Nil),
    }
    Ok(2)
}

fn c_addons_is_addon_load_on_demand(state: &mut LuaState) -> LuaResult<u32> {
    let lod = with_addon(state, stack_val(state, 1), |a| a.load_on_demand).unwrap_or(false);
    state.push(Val::Bool(lod));
    Ok(1)
}

fn c_addons_get_addon_enable_state(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = with_addon(state, stack_val(state, 1), |a| a.enabled).unwrap_or(false);
    state.push(Val::Num(if enabled { 2.0 } else { 0.0 }));
    Ok(1)
}

fn c_addons_enable_addon(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(index) = addon_index_from_value(state, stack_val(state, 1))
        && let Some(addon) = borrow_state_mut(state)?.addons.get_mut(index)
    {
        addon.enabled = true;
    }
    Ok(0)
}

fn c_addons_disable_addon(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(index) = addon_index_from_value(state, stack_val(state, 1))
        && let Some(addon) = borrow_state_mut(state)?.addons.get_mut(index)
        && addon.folder_name != "__BuiltIn"
    {
        addon.enabled = false;
    }
    Ok(0)
}

fn c_addons_enable_all_addons(state: &mut LuaState) -> LuaResult<u32> {
    for addon in &mut borrow_state_mut(state)?.addons {
        addon.enabled = true;
    }
    Ok(0)
}

fn c_addons_disable_all_addons(state: &mut LuaState) -> LuaResult<u32> {
    for addon in &mut borrow_state_mut(state)?.addons {
        if addon.folder_name != "__BuiltIn" {
            addon.enabled = false;
        }
    }
    Ok(0)
}

fn c_addons_save_addons(state: &mut LuaState) -> LuaResult<u32> {
    let version_check_enabled = registry_bool(state, ADDON_VERSION_CHECK_KEY);
    let mut sim = borrow_state_mut(state)?;
    sim.addon_saved_enable_state = Some(AddonEnableSnapshot::from_addons(&sim.addons));
    sim.addon_saved_version_check_enabled = Some(version_check_enabled);
    if let Err(error) = save_local_addon_enable_overrides(&sim.addons) {
        eprintln!("[C_AddOns] failed to save local AddOns.txt: {error}");
    }
    Ok(0)
}

fn c_addons_reset_addons(state: &mut LuaState) -> LuaResult<u32> {
    let saved_version_check_enabled = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(snapshot) = sim.addon_saved_enable_state.clone() {
            for addon in &mut sim.addons {
                if let Some(saved_enabled) = snapshot.saved_enabled(&addon.folder_name) {
                    addon.enabled = saved_enabled;
                }
            }
        }
        sim.addon_saved_version_check_enabled
    };
    if let Some(enabled) = saved_version_check_enabled {
        set_registry_bool(state, ADDON_VERSION_CHECK_KEY, enabled);
    }
    Ok(0)
}

fn c_addons_get_addon_metadata(state: &mut LuaState) -> LuaResult<u32> {
    let addon_name = addon_name_from_value(state, stack_val(state, 1)).unwrap_or_default();
    let field = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let value = with_addon(state, stack_val(state, 1), |a| addon_metadata(a, &field))
        .flatten()
        .or_else(|| (field == "Title").then_some(addon_name));
    match value {
        Some(v) => {
            let v = create_string(state, &v);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_addons_does_addon_exist(state: &mut LuaState) -> LuaResult<u32> {
    let addon_name = addon_name_from_value(state, stack_val(state, 1)).unwrap_or_default();
    let exists = !addon_name.is_empty() && addon_exists(state, &addon_name);
    state.push(Val::Bool(exists));
    Ok(1)
}

fn c_addons_get_addon_name(state: &mut LuaState) -> LuaResult<u32> {
    match with_addon(state, stack_val(state, 1), |a| a.folder_name.clone()) {
        Some(name) => {
            let v = create_string(state, &name);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_addons_get_addon_title(state: &mut LuaState) -> LuaResult<u32> {
    match with_addon(state, stack_val(state, 1), |a| a.title.clone()) {
        Some(title) => {
            let v = create_string(state, &title);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_addons_get_addon_notes(state: &mut LuaState) -> LuaResult<u32> {
    match with_addon(state, stack_val(state, 1), |a| a.notes.clone()).filter(|n| !n.is_empty()) {
        Some(notes) => {
            let v = create_string(state, &notes);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_addons_get_addon_security(state: &mut LuaState) -> LuaResult<u32> {
    let security = with_addon(state, stack_val(state, 1), addon_security_status)
        .unwrap_or_else(|| "INSECURE".to_string());
    let v = create_string(state, &security);
    state.push(v);
    Ok(1)
}

fn addon_security_status(addon: &crate::lua_api::AddonInfo) -> String {
    if let Some(security) = &addon.security {
        return security.clone();
    }

    if addon_is_secure_by_default(addon) {
        "SECURE".to_string()
    } else {
        "INSECURE".to_string()
    }
}

fn addon_is_secure_by_default(addon: &crate::lua_api::AddonInfo) -> bool {
    addon.folder_name == "__BuiltIn"
        || crate::blizzard_ui_sync::is_builtin_addon_folder(&addon.folder_name)
}

/// `C_AddOns.GetAddOnDependencies(indexOrName) → ...string`
///
/// Returns one return value per declared dependency from the addon's TOC
/// (variadic, NOT a table). Used by the addon list to walk LOD readiness
/// and to render the dependency line in the addon tooltip. Unknown addon
/// or empty deps → no return values (nothing pushed).
fn c_addons_get_addon_dependencies(state: &mut LuaState) -> LuaResult<u32> {
    let deps =
        with_addon(state, stack_val(state, 1), |a| a.dependencies.clone()).unwrap_or_default();
    for dep in &deps {
        let val = create_string(state, dep);
        state.push(val);
    }
    Ok(deps.len() as u32)
}

/// `C_AddOns.IsAddOnDefaultEnabled(indexOrName) → bool`
///
/// Returns the addon's factory-default enabled state — `true` unless its TOC
/// declares `## DefaultState: disabled`. Used by the addon list's right-click
/// "Reset to Default" path to decide whether to enable or disable on reset.
/// Unknown addon → `false`.
fn c_addons_is_addon_default_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let default_enabled =
        with_addon(state, stack_val(state, 1), |a| a.default_enabled).unwrap_or(false);
    state.push(Val::Bool(default_enabled));
    Ok(1)
}

fn c_addons_get_scripts_disallowed_for_beta(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_addons_is_addon_version_check_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = registry_bool(state, ADDON_VERSION_CHECK_KEY);
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn c_addons_set_addon_version_check(state: &mut LuaState) -> LuaResult<u32> {
    set_registry_bool(
        state,
        ADDON_VERSION_CHECK_KEY,
        !matches!(stack_val(state, 1), Val::Nil | Val::Bool(false)),
    );
    Ok(0)
}

pub fn c_addons_load_addon(state: &mut LuaState) -> LuaResult<u32> {
    let Some(addon_name) = addon_name_from_value(state, stack_val(state, 1)) else {
        return push_load_result(state, false, Some("MISSING"));
    };
    if !addon_exists(state, &addon_name) {
        return push_load_result(state, false, Some("MISSING"));
    }
    if with_addon(state, stack_val(state, 1), |a| a.loaded).unwrap_or(false) {
        return push_load_result(state, true, None);
    }
    if borrow_state(state)
        .map(|sim| sim.is_addon_loading(&addon_name))
        .unwrap_or(false)
    {
        return push_load_result(state, true, None);
    }
    if addon_is_disabled(state, &addon_name) {
        return push_load_result(state, false, Some("DISABLED"));
    }
    let saved_top = state.top;
    let saved_base = state.base;
    let saved_ci = state.ci;
    let result = c_addons_runtime::load_runtime_addon(state, &addon_name);
    state.top = saved_top;
    state.base = saved_base;
    state.ci = saved_ci;
    match result {
        Ok(()) => push_load_result(state, true, None),
        Err(error) => push_load_result(state, false, Some(&error.to_string())),
    }
}

/// `true` iff the addon is registered AND its `enabled` flag is false.
/// Unregistered addons are not considered disabled — `LoadAddOn` will
/// fall through to `MISSING` (or auto-register at load time).
pub(super) fn addon_is_disabled(state: &LuaState, addon_name: &str) -> bool {
    let Ok(sim) = borrow_state(state) else {
        return false;
    };
    sim.addons
        .iter()
        .find(|a| a.folder_name == addon_name)
        .map(|a| !a.enabled)
        .unwrap_or(false)
}

fn push_load_result(
    state: &mut LuaState,
    success: bool,
    error_msg: Option<&str>,
) -> LuaResult<u32> {
    state.push(Val::Bool(success));
    match error_msg {
        Some(msg) => {
            let msg_val = create_string(state, msg);
            state.push(msg_val);
        }
        None => state.push(Val::Nil),
    }
    Ok(2)
}

fn legacy_get_addon_enable_state(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(2.0));
    Ok(1)
}

#[cfg(test)]
#[cfg_attr(not(feature = "client-mists"), allow(dead_code, unused_imports))]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApi;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_runtime_addon_lookup_accepts_classic_panel_manager() {
        let ui = TempAddonBase::new("mists-classic-uiparent-panel-manager");
        ui.add_addon(
            "Blizzard_UIParentPanelManager",
            "Blizzard_UIParentPanelManager_Classic.toc",
            r#"
## Title: Blizzard_UIParentPanelManager
## AllowLoad: Game
## AllowLoadGameType: classic
Classic\UIParentPanelManager.lua
"#,
        );

        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.state().borrow_mut().addon_base_paths = vec![ui.path.clone()];

        let found = {
            let lua = env.rilua();
            find_runtime_addon_toc(lua.state(), "Blizzard_UIParentPanelManager")
        };

        assert!(
            found.is_some(),
            "Mists should resolve the 5.5.4 Classic UIParentPanelManager variant"
        );
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_runtime_addon_lookup_keeps_allowed_addons_available() {
        let ui = TempAddonBase::new("mists-allowed-addon");
        ui.add_addon(
            "Blizzard_AllowedRuntimeAddon",
            "Blizzard_AllowedRuntimeAddon.toc",
            r#"
## Title: Blizzard_AllowedRuntimeAddon
## AllowLoad: Game
Core.lua
"#,
        );

        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.state().borrow_mut().addon_base_paths = vec![ui.path.clone()];

        let found = {
            let lua = env.rilua();
            find_runtime_addon_toc(lua.state(), "Blizzard_AllowedRuntimeAddon")
        };

        assert!(
            found.is_some(),
            "allowed runtime addons should still resolve"
        );
    }

    struct TempAddonBase {
        path: PathBuf,
    }

    impl TempAddonBase {
        fn new(suffix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("wow-sim-c-addons-{suffix}-{unique}"));
            std::fs::create_dir_all(&path).expect("temp addon base should be created");
            Self { path }
        }

        fn add_addon(&self, addon_name: &str, toc_name: &str, toc_contents: &str) {
            let addon_dir = self.path.join(addon_name);
            std::fs::create_dir_all(&addon_dir).expect("addon dir should be created");
            std::fs::write(addon_dir.join(toc_name), toc_contents).expect("toc should be written");
            std::fs::write(addon_dir.join("Core.lua"), "").expect("Core.lua should be written");
            write_parent_file(&addon_dir, Path::new("Classic/UIParentPanelManager.lua"));
        }
    }

    impl Drop for TempAddonBase {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn write_parent_file(addon_dir: &Path, file_path: &Path) {
        let path = addon_dir.join(file_path);
        std::fs::create_dir_all(path.parent().expect("file path should have parent"))
            .expect("parent dir should be created");
        std::fs::write(path, "").expect("file should be written");
    }
}
