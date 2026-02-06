//! Addon-related WoW API functions.

use super::super::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register addon-related global functions (C_AddOns namespace and legacy globals).
pub fn register_addon_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // C_AddOns namespace - modern addon management API
    let c_addons = lua.create_table()?;

    // GetAddOnMetadata - returns metadata from TOC file
    let state_for_metadata = Rc::clone(&state);
    c_addons.set(
        "GetAddOnMetadata",
        lua.create_function(move |lua, (addon, field): (String, String)| {
            let state = state_for_metadata.borrow();
            // Find the addon by name
            let addon_info = state
                .addons
                .iter()
                .find(|a| a.folder_name == addon || a.title == addon);

            if let Some(info) = addon_info {
                let value = match field.as_str() {
                    "Version" => "@project-version@",
                    "X-Flavor" => "Mainline",
                    "Title" => &info.title,
                    "Notes" => &info.notes,
                    "Author" => "",
                    "Group" => &info.folder_name,
                    _ => "",
                };
                if value.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(value)?))
                }
            } else {
                // Addon not found - return default values
                let value = match field.as_str() {
                    "Version" => "@project-version@",
                    "X-Flavor" => "Mainline",
                    "Title" => addon.as_str(),
                    "Group" => addon.as_str(),
                    _ => "",
                };
                if value.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(value)?))
                }
            }
        })?,
    )?;

    // EnableAddOn(addon, character) - enable a single addon by name or index
    let state_for_enable_one = Rc::clone(&state);
    c_addons.set(
        "EnableAddOn",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            let mut state = state_for_enable_one.borrow_mut();
            match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    if let Some(a) = state.addons.get_mut(idx) {
                        a.enabled = true;
                    }
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    if let Some(a) = state.addons.iter_mut().find(|a| a.folder_name == &*name) {
                        a.enabled = true;
                    }
                }
                _ => {}
            }
            Ok(())
        })?,
    )?;

    // DisableAddOn(addon, character) - disable a single addon by name or index
    let state_for_disable_one = Rc::clone(&state);
    c_addons.set(
        "DisableAddOn",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            let mut state = state_for_disable_one.borrow_mut();
            match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    if let Some(a) = state.addons.get_mut(idx) {
                        a.enabled = false;
                    }
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    if let Some(a) = state.addons.iter_mut().find(|a| a.folder_name == &*name) {
                        a.enabled = false;
                    }
                }
                _ => {}
            }
            Ok(())
        })?,
    )?;

    // EnableAllAddOns(character) - enable all addons
    let state_for_enable_all = Rc::clone(&state);
    c_addons.set(
        "EnableAllAddOns",
        lua.create_function(move |_, _character: Option<String>| {
            let mut state = state_for_enable_all.borrow_mut();
            for addon in &mut state.addons {
                addon.enabled = true;
            }
            Ok(())
        })?,
    )?;

    // DisableAllAddOns(character) - disable all addons
    let state_for_disable_all = Rc::clone(&state);
    c_addons.set(
        "DisableAllAddOns",
        lua.create_function(move |_, _character: Option<String>| {
            let mut state = state_for_disable_all.borrow_mut();
            for addon in &mut state.addons {
                addon.enabled = false;
            }
            Ok(())
        })?,
    )?;

    // GetNumAddOns - return actual addon count
    let state_for_num = Rc::clone(&state);
    c_addons.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| {
            let state = state_for_num.borrow();
            Ok(state.addons.len() as i32)
        })?,
    )?;

    // GetAddOnInfo - return actual addon info
    let state_for_info = Rc::clone(&state);
    c_addons.set(
        "GetAddOnInfo",
        lua.create_function(move |lua, index_or_name: Value| {
            // Return: name, title, notes, loadable, reason, security, newVersion
            let state = state_for_info.borrow();
            let addon = match index_or_name {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize; // Lua is 1-indexed
                    state.addons.get(idx)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().find(|a| a.folder_name == &*name)
                }
                _ => None,
            };

            if let Some(addon) = addon {
                // Mark all addons as loadable so they show with gold text
                let loadable = true;
                Ok((
                    Value::String(lua.create_string(&addon.folder_name)?),
                    Value::String(lua.create_string(&addon.title)?),
                    Value::String(lua.create_string(&addon.notes)?),
                    Value::Boolean(loadable),
                    Value::Nil, // reason
                    Value::String(lua.create_string("INSECURE")?),
                    Value::Boolean(false), // newVersion
                ))
            } else {
                Ok((
                    Value::Nil,
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                ))
            }
        })?,
    )?;

    // IsAddOnLoaded - check if addon is actually loaded
    let state_for_loaded = Rc::clone(&state);
    c_addons.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_loaded.borrow();
            let found = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.loaded).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state
                        .addons
                        .iter()
                        .any(|a| a.folder_name == &*name && a.loaded)
                }
                _ => false,
            };
            Ok(found)
        })?,
    )?;

    c_addons.set(
        "IsAddOnLoadable",
        lua.create_function(|_, _addon: String| Ok(true))?,
    )?;

    // IsAddOnLoadOnDemand - check actual LOD flag
    let state_for_lod = Rc::clone(&state);
    c_addons.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_lod.borrow();
            let lod = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state
                        .addons
                        .get(idx)
                        .map(|a| a.load_on_demand)
                        .unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state
                        .addons
                        .iter()
                        .find(|a| a.folder_name == &*name)
                        .map(|a| a.load_on_demand)
                        .unwrap_or(false)
                }
                _ => false,
            };
            Ok(lod)
        })?,
    )?;

    c_addons.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;

    c_addons.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;

    c_addons.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;

    // DoesAddOnExist - check if addon is in the registry
    let state_for_exists = Rc::clone(&state);
    c_addons.set(
        "DoesAddOnExist",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_exists.borrow();
            let exists = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    idx < state.addons.len()
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().any(|a| a.folder_name == &*name)
                }
                _ => false,
            };
            Ok(exists)
        })?,
    )?;

    // GetAddOnEnableState - check actual enabled state
    let state_for_enable = Rc::clone(&state);
    c_addons.set(
        "GetAddOnEnableState",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            // Returns: enabled state (0 = disabled, 1 = enabled for some, 2 = enabled for all)
            let state = state_for_enable.borrow();
            let enabled = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.enabled).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state
                        .addons
                        .iter()
                        .find(|a| a.folder_name == &*name)
                        .map(|a| a.enabled)
                        .unwrap_or(false)
                }
                _ => false,
            };
            Ok(if enabled { 2i32 } else { 0i32 })
        })?,
    )?;

    // GetAddOnName - return folder name
    let state_for_name = Rc::clone(&state);
    c_addons.set(
        "GetAddOnName",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_name.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                Ok(Value::String(lua.create_string(&addon.folder_name)?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // GetAddOnTitle - return display title
    let state_for_title = Rc::clone(&state);
    c_addons.set(
        "GetAddOnTitle",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_title.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                Ok(Value::String(lua.create_string(&addon.title)?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // GetAddOnNotes - return addon description/notes
    let state_for_notes = Rc::clone(&state);
    c_addons.set(
        "GetAddOnNotes",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_notes.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                if addon.notes.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(&addon.notes)?))
                }
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // GetAddOnSecurity - return security level (always INSECURE for addons)
    c_addons.set(
        "GetAddOnSecurity",
        lua.create_function(|lua, _index: i64| {
            // Security levels: SECURE, INSECURE, BANNED
            Ok(Value::String(lua.create_string("INSECURE")?))
        })?,
    )?;

    // IsAddonVersionCheckEnabled - check if addon version validation is enabled
    let state_for_version_check = Rc::clone(&state);
    c_addons.set(
        "IsAddonVersionCheckEnabled",
        lua.create_function(move |_, ()| {
            let state = state_for_version_check.borrow();
            Ok(state.cvars.get_bool("checkAddonVersion"))
        })?,
    )?;

    // SetAddonVersionCheck - toggle addon version validation
    let state_for_set_version = Rc::clone(&state);
    c_addons.set(
        "SetAddonVersionCheck",
        lua.create_function(move |_, enabled: bool| {
            let state = state_for_set_version.borrow();
            state.cvars.set("checkAddonVersion", if enabled { "1" } else { "0" });
            Ok(())
        })?,
    )?;

    globals.set("C_AddOns", c_addons)?;

    // ADDON_ACTIONS_BLOCKED - table of addon names that have blocked actions (used by AddonList)
    // This is normally populated by the game when addons use protected functions
    globals.set("ADDON_ACTIONS_BLOCKED", lua.create_table()?)?;

    // AddOnPerformance - addon performance monitoring (nil when not loaded)
    // This is populated by Blizzard_AddOnPerformanceWarning addon
    globals.set("AddOnPerformance", Value::Nil)?;

    // Legacy global addon functions - delegate to state
    let state_for_legacy_num = Rc::clone(&state);
    globals.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| {
            let state = state_for_legacy_num.borrow();
            Ok(state.addons.len() as i32)
        })?,
    )?;

    let state_for_legacy_loaded = Rc::clone(&state);
    globals.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_legacy_loaded.borrow();
            let found = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.loaded).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state
                        .addons
                        .iter()
                        .any(|a| a.folder_name == &*name && a.loaded)
                }
                _ => false,
            };
            Ok(found)
        })?,
    )?;

    globals.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;

    let state_for_legacy_lod = Rc::clone(&state);
    globals.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_legacy_lod.borrow();
            let lod = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state
                        .addons
                        .get(idx)
                        .map(|a| a.load_on_demand)
                        .unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state
                        .addons
                        .iter()
                        .find(|a| a.folder_name == &*name)
                        .map(|a| a.load_on_demand)
                        .unwrap_or(false)
                }
                _ => false,
            };
            Ok(lod)
        })?,
    )?;

    globals.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;

    globals.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;

    // Legacy global function version
    globals.set(
        "GetAddOnEnableState",
        lua.create_function(|_, (_addon, _character): (Value, Option<String>)| Ok(2i32))?,
    )?;

    // GetAddOnMetadata legacy global
    let state_for_legacy_metadata = Rc::clone(&state);
    globals.set(
        "GetAddOnMetadata",
        lua.create_function(move |lua, (addon, field): (String, String)| {
            let state = state_for_legacy_metadata.borrow();
            let addon_info = state
                .addons
                .iter()
                .find(|a| a.folder_name == addon || a.title == addon);

            if let Some(info) = addon_info {
                let value = match field.as_str() {
                    "Version" => "@project-version@",
                    "X-Flavor" => "Mainline",
                    "Title" => &info.title,
                    "Notes" => &info.notes,
                    "Author" => "",
                    "Group" => &info.folder_name,
                    _ => "",
                };
                if value.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(value)?))
                }
            } else {
                let value = match field.as_str() {
                    "Version" => "@project-version@",
                    "X-Flavor" => "Mainline",
                    "Title" => addon.as_str(),
                    "Group" => addon.as_str(),
                    _ => "",
                };
                if value.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(value)?))
                }
            }
        })?,
    )?;

    Ok(())
}
