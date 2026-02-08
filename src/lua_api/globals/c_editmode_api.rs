//! C_EditMode namespace for Edit Mode layout management.
//!
//! Provides the minimum API needed for `EditModeManagerFrame:UpdateLayoutInfo()`
//! to initialize and fire `EDIT_MODE_LAYOUTS_UPDATED`, which unblocks action bar
//! positioning via `UpdateBottomActionBarPositions()`.

use mlua::{Lua, Result, Value};

/// Register the C_EditMode namespace.
pub fn register_c_editmode_api(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    // GetLayouts: returns { activeLayout = 1, layouts = {} }
    // Empty layouts table — presets are prepended automatically by UpdateLayoutInfo.
    t.set(
        "GetLayouts",
        lua.create_function(|lua, ()| {
            let info = lua.create_table()?;
            info.set("activeLayout", 1)?;
            info.set("layouts", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;

    // GetAccountSettings: returns array of { setting, value } entries for enums 0–28.
    t.set(
        "GetAccountSettings",
        lua.create_function(|lua, ()| {
            let settings = lua.create_table()?;
            for i in 0..=28 {
                let entry = lua.create_table()?;
                entry.set("setting", i)?;
                entry.set("value", account_setting_default(i))?;
                settings.set(i + 1, entry)?;
            }
            Ok(settings)
        })?,
    )?;

    // No-op stubs
    for name in [
        "SaveLayouts",
        "SetActiveLayout",
        "SetAccountSetting",
        "OnEditModeExit",
        "OnLayoutAdded",
        "OnLayoutDeleted",
    ] {
        t.set(name, lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    }

    // Simple return stubs
    t.set(
        "IsValidLayoutName",
        lua.create_function(|_, _name: Value| Ok(true))?,
    )?;
    t.set(
        "ConvertLayoutInfoToString",
        lua.create_function(|lua, _info: Value| Ok(lua.create_string("")?))?,
    )?;
    t.set(
        "ConvertStringToLayoutInfo",
        lua.create_function(|_, _s: Value| Ok(Value::Nil))?,
    )?;
    t.set(
        "ConvertLayoutInfoToHyperlink",
        lua.create_function(|lua, _info: Value| Ok(lua.create_string("")?))?,
    )?;

    lua.globals().set("C_EditMode", t)?;
    Ok(())
}

/// Default value for an account setting enum index.
/// Enum values 0–28 map to EditMode account settings. Most "Show*" = 1 (visible).
fn account_setting_default(setting: i32) -> i32 {
    match setting {
        // ShowGrid = 0
        4 => 0,
        // GridSpacing = 100
        5 => 100,
        // EnableAdvancedOptions = 0
        8 => 0,
        // DeprecatedShowDebuffFrame = 0
        28 => 0,
        // All other Show* settings = 1, SettingsExpanded = 1, EnableSnap = 1
        _ => 1,
    }
}
