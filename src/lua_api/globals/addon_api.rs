//! Addon-related WoW API functions.

use super::super::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register addon-related global functions (C_AddOns namespace and legacy globals).
pub fn register_addon_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let c_addons = lua.create_table()?;
    register_metadata_functions(lua, &c_addons, &state)?;
    register_enable_disable(lua, &c_addons, &state)?;
    register_query_functions(lua, &c_addons, &state)?;
    register_version_check(lua, &c_addons, &state)?;
    register_stub_functions(lua, &c_addons, &state)?;
    lua.globals().set("C_AddOns", c_addons)?;

    register_global_constants(lua)?;
    register_legacy_globals(lua, &state)?;
    register_profiler(lua, &state)?;

    Ok(())
}

/// Register GetAddOnMetadata, GetAddOnInfo, GetAddOnName, GetAddOnTitle, GetAddOnNotes, GetAddOnSecurity, GetNumAddOns.
fn register_metadata_functions(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    register_metadata_core(lua, c_addons, state)?;
    register_metadata_by_index(lua, c_addons, state)?;

    c_addons.set(
        "GetAddOnSecurity",
        lua.create_function(|lua, _index: i64| {
            Ok(Value::String(lua.create_string("INSECURE")?))
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| Ok(s.borrow().addons.len() as i32))?,
    )?;

    Ok(())
}

fn register_metadata_core(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnMetadata",
        lua.create_function(move |lua, (addon, field): (String, String)| {
            resolve_metadata(&s, lua, &addon, &field)
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnInfo",
        lua.create_function(move |lua, index_or_name: Value| {
            let state = s.borrow();
            let addon = find_addon_by_value(&state.addons, &index_or_name);
            addon_info_tuple(lua, addon)
        })?,
    )?;
    Ok(())
}

fn addon_info_tuple(lua: &Lua, addon: Option<&AddonInfo>) -> Result<(Value, Value, Value, Value, Value, Value, Value)> {
    if let Some(addon) = addon {
        Ok((
            Value::String(lua.create_string(&addon.folder_name)?),
            Value::String(lua.create_string(&addon.title)?),
            Value::String(lua.create_string(&addon.notes)?),
            Value::Boolean(true),
            Value::Nil,
            Value::String(lua.create_string("INSECURE")?),
            Value::Boolean(false),
        ))
    } else {
        Ok((
            Value::Nil, Value::Nil, Value::Nil,
            Value::Boolean(false), Value::Nil, Value::Nil, Value::Boolean(false),
        ))
    }
}

fn register_metadata_by_index(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnName",
        lua.create_function(move |lua, index: i64| {
            let state = s.borrow();
            match state.addons.get((index - 1) as usize) {
                Some(a) => Ok(Value::String(lua.create_string(&a.folder_name)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnTitle",
        lua.create_function(move |lua, index: i64| {
            let state = s.borrow();
            match state.addons.get((index - 1) as usize) {
                Some(a) => Ok(Value::String(lua.create_string(&a.title)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnNotes",
        lua.create_function(move |lua, index: i64| {
            let state = s.borrow();
            match state.addons.get((index - 1) as usize) {
                Some(a) if !a.notes.is_empty() => {
                    Ok(Value::String(lua.create_string(&a.notes)?))
                }
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    Ok(())
}

/// Register EnableAddOn, DisableAddOn, EnableAllAddOns, DisableAllAddOns, GetAddOnEnableState.
fn register_enable_disable(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let s = Rc::clone(state);
    c_addons.set(
        "EnableAddOn",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            set_addon_enabled(&s, &addon, true);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "DisableAddOn",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            set_addon_enabled(&s, &addon, false);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "EnableAllAddOns",
        lua.create_function(move |_, _character: Option<String>| {
            for addon in &mut s.borrow_mut().addons {
                addon.enabled = true;
            }
            Ok(())
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "DisableAllAddOns",
        lua.create_function(move |_, _character: Option<String>| {
            for addon in &mut s.borrow_mut().addons {
                addon.enabled = false;
            }
            Ok(())
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "GetAddOnEnableState",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            let state = s.borrow();
            let enabled = find_addon_by_value(&state.addons, &addon)
                .map(|a| a.enabled)
                .unwrap_or(false);
            Ok(if enabled { 2i32 } else { 0i32 })
        })?,
    )?;

    Ok(())
}

/// Register IsAddOnLoaded, IsAddOnLoadable, IsAddOnLoadOnDemand, DoesAddOnExist.
fn register_query_functions(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let s = Rc::clone(state);
    c_addons.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            Ok(find_addon_by_value(&state.addons, &addon)
                .map(|a| a.loaded)
                .unwrap_or(false))
        })?,
    )?;

    c_addons.set(
        "IsAddOnLoadable",
        lua.create_function(|_, _addon: String| Ok(true))?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            Ok(find_addon_by_value(&state.addons, &addon)
                .map(|a| a.load_on_demand)
                .unwrap_or(false))
        })?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "DoesAddOnExist",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            Ok(find_addon_by_value(&state.addons, &addon).is_some())
        })?,
    )?;

    Ok(())
}

/// Register IsAddonVersionCheckEnabled and SetAddonVersionCheck.
fn register_version_check(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let s = Rc::clone(state);
    c_addons.set(
        "IsAddonVersionCheckEnabled",
        lua.create_function(move |_, ()| Ok(s.borrow().cvars.get_bool("checkAddonVersion")))?,
    )?;

    let s = Rc::clone(state);
    c_addons.set(
        "SetAddonVersionCheck",
        lua.create_function(move |_, enabled: bool| {
            s.borrow()
                .cvars
                .set("checkAddonVersion", if enabled { "1" } else { "0" });
            Ok(())
        })?,
    )?;

    Ok(())
}

/// Register stub functions that return empty values.
fn register_stub_functions(
    lua: &Lua,
    c_addons: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    c_addons.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    c_addons.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    let s = Rc::clone(state);
    c_addons.set("LoadAddOn", create_load_addon_fn(lua, s)?)?;
    c_addons.set(
        "GetScriptsDisallowedForBeta",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    Ok(())
}

/// Register ADDON_ACTIONS_BLOCKED table and AddOnPerformance nil.
fn register_global_constants(lua: &Lua) -> Result<()> {
    lua.globals()
        .set("ADDON_ACTIONS_BLOCKED", lua.create_table()?)?;
    lua.globals().set("AddOnPerformance", Value::Nil)?;
    Ok(())
}

/// Register C_AddOnProfiler namespace and legacy memory usage globals.
///
/// Uses actual addon load times as a proxy for runtime profiler metrics.
/// Application metric = total load time * 3 (simulates addons being ~33% of frame time).
/// Overall metric = sum of all addon load times.
/// Per-addon metric = that addon's load time.
fn register_profiler(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let c_profiler = lua.create_table()?;

    c_profiler.set("IsEnabled", lua.create_function(|_, ()| Ok(true))?)?;

    let s = Rc::clone(state);
    c_profiler.set(
        "GetApplicationMetric",
        lua.create_function(move |_, _metric: Value| {
            let state = s.borrow();
            let overall: f64 = state.addons.iter().map(|a| a.load_time_secs).sum();
            // Application time = addon time * 3 so addons show as ~33% of total
            Ok(overall * 3.0)
        })?,
    )?;

    let s = Rc::clone(state);
    c_profiler.set(
        "GetOverallMetric",
        lua.create_function(move |_, _metric: Value| {
            let state = s.borrow();
            let overall: f64 = state.addons.iter().map(|a| a.load_time_secs).sum();
            Ok(overall)
        })?,
    )?;

    let s = Rc::clone(state);
    c_profiler.set(
        "GetAddOnMetric",
        lua.create_function(move |_, (addon, _metric): (Value, Value)| {
            let state = s.borrow();
            let val = find_addon_by_value(&state.addons, &addon)
                .map(|a| a.load_time_secs)
                .unwrap_or(0.0);
            Ok(val)
        })?,
    )?;

    c_profiler.set(
        "CheckForPerformanceMessage",
        lua.create_function(|_, ()| Ok(mlua::Value::Nil))?,
    )?;

    c_profiler.set(
        "AddPerformanceMessageShown",
        lua.create_function(|_, _msg: Value| Ok(()))?,
    )?;

    lua.globals().set("C_AddOnProfiler", c_profiler)?;

    register_legacy_memory_globals(lua, state)
}

fn register_legacy_memory_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    lua.globals().set(
        "UpdateAddOnMemoryUsage",
        lua.create_function(|_, ()| Ok(()))?,
    )?;

    let s = Rc::clone(state);
    lua.globals().set(
        "GetAddOnMemoryUsage",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            let kb = find_addon_by_value(&state.addons, &addon)
                .map(|a| a.load_time_secs * 500.0)
                .unwrap_or(0.0);
            Ok(kb)
        })?,
    )?;

    Ok(())
}

/// Register legacy global functions that mirror C_AddOns.
fn register_legacy_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_legacy_addon_query(lua, state)?;
    register_legacy_addon_stubs(lua, state)?;
    Ok(())
}

fn register_legacy_addon_query(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let s = Rc::clone(state);
    globals.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| Ok(s.borrow().addons.len() as i32))?,
    )?;

    let s = Rc::clone(state);
    globals.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            Ok(find_addon_by_value(&state.addons, &addon)
                .map(|a| a.loaded)
                .unwrap_or(false))
        })?,
    )?;

    let s = Rc::clone(state);
    globals.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = s.borrow();
            Ok(find_addon_by_value(&state.addons, &addon)
                .map(|a| a.load_on_demand)
                .unwrap_or(false))
        })?,
    )?;

    let s = Rc::clone(state);
    globals.set(
        "GetAddOnMetadata",
        lua.create_function(move |lua, (addon, field): (String, String)| {
            resolve_metadata(&s, lua, &addon, &field)
        })?,
    )?;
    Ok(())
}

fn register_legacy_addon_stubs(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let s = Rc::clone(state);
    globals.set("LoadAddOn", create_load_addon_fn(lua, s)?)?;
    globals.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    globals.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    globals.set(
        "GetAddOnEnableState",
        lua.create_function(|_, (_addon, _character): (Value, Option<String>)| Ok(2i32))?,
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Runtime addon loading
// ---------------------------------------------------------------------------

/// Create the `LoadAddOn(name)` function that actually loads on-demand addons.
///
/// Returns `(loaded: bool, reason: string|nil)`.
fn create_load_addon_fn(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |lua, addon_name: String| {
        load_addon_runtime(lua, &state, &addon_name)
    })
}

/// Runtime addon loading implementation.
///
/// Searches `addon_base_paths` for the addon directory, loads it via the
/// standard loader pipeline, registers it, and fires `ADDON_LOADED`.
fn load_addon_runtime(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    addon_name: &str,
) -> Result<(bool, Value)> {
    // Already loaded? Return early.
    {
        let s = state.borrow();
        if s.addons.iter().any(|a| a.folder_name == addon_name && a.loaded) {
            return Ok((true, Value::Nil));
        }
    }

    // Search addon_base_paths for the addon directory and its TOC file.
    let toc_path = find_addon_toc(state, addon_name);

    let toc_path = match toc_path {
        Some(p) => p,
        None => {
            let reason = lua.create_string("MISSING")?;
            return Ok((false, Value::String(reason)));
        }
    };

    // Parse TOC to check dependencies, then load them first.
    if let Ok(toc) = crate::toc::TocFile::from_file(&toc_path) {
        for dep in toc.dependencies() {
            let already_loaded = {
                let s = state.borrow();
                s.addons.iter().any(|a| a.folder_name == dep && a.loaded)
            };
            if !already_loaded {
                let _ = load_addon_runtime(lua, state, &dep);
            }
        }
    }

    // Load the addon via the standard loader pipeline.
    let loader_env = crate::lua_api::LoaderEnv::new(lua, Rc::clone(state));
    match crate::loader::load_addon(&loader_env, &toc_path) {
        Ok(result) => {
            let load_time_secs = result.timing.total().as_secs_f64();
            eprintln!(
                "[LoadAddOn] {} loaded: {} Lua, {} XML ({:.1?})",
                addon_name,
                result.lua_files,
                result.xml_files,
                result.timing.total()
            );
            register_loaded_addon(state, addon_name, load_time_secs);
            fire_addon_loaded(&loader_env, addon_name);
            Ok((true, Value::Nil))
        }
        Err(e) => {
            eprintln!("[LoadAddOn] {} failed: {}", addon_name, e);
            let reason = lua.create_string("CORRUPT")?;
            Ok((false, Value::String(reason)))
        }
    }
}

/// Search addon_base_paths for an addon's TOC file.
fn find_addon_toc(state: &Rc<RefCell<SimState>>, addon_name: &str) -> Option<std::path::PathBuf> {
    let s = state.borrow();
    s.addon_base_paths
        .iter()
        .map(|base| base.join(addon_name))
        .find_map(|dir| {
            if dir.is_dir() {
                crate::loader::find_toc_file(&dir)
            } else {
                None
            }
        })
}

/// Register a newly loaded addon in SimState.
fn register_loaded_addon(state: &Rc<RefCell<SimState>>, name: &str, load_time_secs: f64) {
    let mut s = state.borrow_mut();
    // Update existing entry or create new one.
    if let Some(existing) = s.addons.iter_mut().find(|a| a.folder_name == name) {
        existing.loaded = true;
        existing.load_time_secs = load_time_secs;
    } else {
        s.addons.push(AddonInfo {
            folder_name: name.to_string(),
            title: name.to_string(),
            notes: String::new(),
            enabled: true,
            loaded: true,
            load_on_demand: true,
            load_time_secs,
        });
    }
}

/// Fire the ADDON_LOADED event for a just-loaded addon.
fn fire_addon_loaded(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    let arg = env.lua().create_string(addon_name).ok().map(Value::String);
    if let Some(arg) = arg
        && let Err(e) = env.fire_event_with_args("ADDON_LOADED", &[arg]) {
            eprintln!("[LoadAddOn] Error firing ADDON_LOADED for {}: {}", addon_name, e);
        }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use crate::lua_api::AddonInfo;

/// Find an addon by Lua index (1-based integer) or name (string).
fn find_addon_by_value<'a>(addons: &'a [AddonInfo], value: &Value) -> Option<&'a AddonInfo> {
    match value {
        Value::Integer(idx) => addons.get((*idx - 1) as usize),
        Value::String(s) => {
            let name = s.to_string_lossy();
            addons.iter().find(|a| a.folder_name == *name)
        }
        _ => None,
    }
}

/// Find a mutable addon by Lua index or name, then set its enabled flag.
fn set_addon_enabled(state: &Rc<RefCell<SimState>>, addon: &Value, enabled: bool) {
    let mut state = state.borrow_mut();
    match addon {
        Value::Integer(idx) => {
            if let Some(a) = state.addons.get_mut((*idx - 1) as usize) {
                a.enabled = enabled;
            }
        }
        Value::String(s) => {
            let name = s.to_string_lossy();
            if let Some(a) = state.addons.iter_mut().find(|a| a.folder_name == *name) {
                a.enabled = enabled;
            }
        }
        _ => {}
    }
}

/// Resolve addon metadata for a given addon name and field.
fn resolve_metadata(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
    addon_name: &str,
    field: &str,
) -> Result<Value> {
    let state = state.borrow();
    let addon_info = state
        .addons
        .iter()
        .find(|a| a.folder_name == addon_name || a.title == addon_name);

    let value = if let Some(info) = addon_info {
        match field {
            "Version" => "@project-version@",
            "X-Flavor" => "Mainline",
            "Title" => &info.title,
            "Notes" => &info.notes,
            "Author" => "",
            "Group" => &info.folder_name,
            _ => "",
        }
    } else {
        match field {
            "Version" => "@project-version@",
            "X-Flavor" => "Mainline",
            "Title" => addon_name,
            "Group" => addon_name,
            _ => "",
        }
    };

    if value.is_empty() {
        Ok(Value::Nil)
    } else {
        Ok(Value::String(lua.create_string(value)?))
    }
}
