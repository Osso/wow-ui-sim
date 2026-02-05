//! UI string constants and localization globals.
//!
//! This module separates data from logic:
//! - `string_data` - Static arrays of string constants
//! - Registration functions that iterate over the data

pub mod string_data;

use mlua::{Lua, Result, Value};
use string_data::*;

/// Helper to register string constants from a static array.
fn register_strings(globals: &mlua::Table, data: &[StringDef]) -> Result<()> {
    for (name, value) in data {
        globals.set(*name, *value)?;
    }
    Ok(())
}

/// Helper to register integer constants from a static array.
fn register_ints(globals: &mlua::Table, data: &[IntDef]) -> Result<()> {
    for (name, value) in data {
        globals.set(*name, *value)?;
    }
    Ok(())
}

/// Helper to register float constants from a static array.
fn register_floats(globals: &mlua::Table, data: &[FloatDef]) -> Result<()> {
    for (name, value) in data {
        globals.set(*name, *value)?;
    }
    Ok(())
}

/// Registers keybinding functions in the Lua globals table.
pub fn register_keybinding_functions(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    globals.set(
        "GetBindingKey",
        lua.create_function(|_, _action: String| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBinding",
        lua.create_function(|_lua, index: i32| {
            if index < 1 {
                return Ok(mlua::MultiValue::new());
            }
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Nil,
                Value::Nil,
                Value::Nil,
            ]))
        })?,
    )?;
    globals.set(
        "GetNumBindings",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    globals.set(
        "SetBinding",
        lua.create_function(|_, (_key, _action): (String, Option<String>)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingClick",
        lua.create_function(
            |_, (_key, _button, _mouse_button): (String, String, Option<String>)| Ok(true),
        )?,
    )?;
    globals.set(
        "SetBindingSpell",
        lua.create_function(|_, (_key, _spell): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingItem",
        lua.create_function(|_, (_key, _item): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingMacro",
        lua.create_function(|_, (_key, _macro): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "GetCurrentBindingSet",
        lua.create_function(|_, ()| Ok(1))?,
    )?;
    globals.set(
        "SaveBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "LoadBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "GetBindingAction",
        lua.create_function(|_, (_key, _check_override): (String, Option<bool>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBindingText",
        lua.create_function(
            |lua, (key, _prefix, _abbrev): (String, Option<String>, Option<bool>)| {
                Ok(Value::String(lua.create_string(&key)?))
            },
        )?,
    )?;

    Ok(())
}

/// Registers tooltip colors (requires Lua for table creation).
pub fn register_tooltip_colors(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let (r, g, b, a) = TOOLTIP_DEFAULT_COLOR;
    let tooltip_default_color = lua.create_table()?;
    tooltip_default_color.set("r", r)?;
    tooltip_default_color.set("g", g)?;
    tooltip_default_color.set("b", b)?;
    tooltip_default_color.set("a", a)?;
    globals.set("TOOLTIP_DEFAULT_COLOR", tooltip_default_color)?;

    let (r, g, b, a) = TOOLTIP_DEFAULT_BG_COLOR;
    let tooltip_default_bg_color = lua.create_table()?;
    tooltip_default_bg_color.set("r", r)?;
    tooltip_default_bg_color.set("g", g)?;
    tooltip_default_bg_color.set("b", b)?;
    tooltip_default_bg_color.set("a", a)?;
    globals.set("TOOLTIP_DEFAULT_BACKGROUND_COLOR", tooltip_default_bg_color)?;

    Ok(())
}

/// Registers item quality colors table (requires Lua for table creation).
pub fn register_item_quality_colors(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let item_quality_colors = lua.create_table()?;
    for (idx, r, g, b, hex) in ITEM_QUALITY_COLORS_DATA {
        let color = lua.create_table()?;
        color.set("r", *r)?;
        color.set("g", *g)?;
        color.set("b", *b)?;
        color.set("hex", *hex)?;
        color.set("color", format!("|c{}|r", hex))?;
        item_quality_colors.set(*idx, color)?;
    }
    globals.set("ITEM_QUALITY_COLORS", item_quality_colors)?;

    Ok(())
}

/// Registers class name lookup tables (requires Lua for table creation).
pub fn register_class_name_tables(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let class_names_male = lua.create_table()?;
    let class_names_female = lua.create_table()?;
    for (key, name) in CLASS_NAMES_DATA {
        class_names_male.set(*key, *name)?;
        class_names_female.set(*key, *name)?;
    }
    globals.set("LOCALIZED_CLASS_NAMES_MALE", class_names_male)?;
    globals.set("LOCALIZED_CLASS_NAMES_FEMALE", class_names_female)?;

    Ok(())
}

/// Registers raid marker icon list (requires Lua for table creation).
pub fn register_icon_list(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let icon_list = lua.create_table()?;
    for (icon, idx) in ICON_LIST_DATA {
        icon_list.set(*idx, *icon)?;
    }
    globals.set("ICON_LIST", icon_list)?;

    Ok(())
}

/// Main entry point: registers all UI string constants.
pub fn register_all_ui_strings(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    // Register generated global strings first (20k+ from WoW CSV exports).
    // Curated constants below override these where they overlap.
    for (name, value) in &crate::global_strings::GLOBAL_STRINGS {
        globals.set(*name, *value)?;
    }

    // Error strings
    register_strings(globals, ERROR_STRINGS)?;

    // Game constants
    register_ints(globals, GAME_INT_CONSTANTS)?;
    register_strings(globals, GAME_STRING_CONSTANTS)?;

    // Expansion constants
    register_ints(globals, EXPANSION_CONSTANTS)?;

    // Inventory slot constants
    register_ints(globals, INVENTORY_SLOT_CONSTANTS)?;

    // Raid target markers
    register_strings(globals, RAID_TARGET_STRINGS)?;

    // Taxi/flight path constants
    register_floats(globals, TAXI_FLOAT_CONSTANTS)?;

    // Keyboard modifier text
    register_strings(globals, KEYBOARD_MODIFIER_STRINGS)?;

    // Keybinding functions
    register_keybinding_functions(lua, globals)?;

    // UI category and button strings
    register_strings(globals, UI_CATEGORY_STRINGS)?;
    register_strings(globals, UI_BUTTON_STRINGS)?;

    // Item-related strings
    register_strings(globals, ITEM_STRINGS)?;
    register_strings(globals, SOCKET_STRINGS)?;
    register_strings(globals, ITEM_BINDING_STRINGS)?;
    register_strings(globals, ITEM_REQUIREMENT_STRINGS)?;
    register_strings(globals, ITEM_UPGRADE_STRINGS)?;

    // Binding headers and names
    register_strings(globals, BINDING_HEADER_STRINGS)?;
    register_strings(globals, BINDING_NAME_STRINGS)?;

    // Misc UI strings
    register_strings(globals, MISC_UI_STRINGS)?;

    // Combat and duel strings
    register_strings(globals, DUEL_STRINGS)?;
    register_strings(globals, COMBAT_TEXT_STRINGS)?;
    register_ints(globals, COMBAT_LOG_RAID_TARGET_CONSTANTS)?;

    // Loot and currency strings
    register_strings(globals, LOOT_STRINGS)?;
    register_strings(globals, CURRENCY_STRINGS)?;

    // XP and quest strings
    register_strings(globals, XP_QUEST_STRINGS)?;

    // Chat format strings
    register_strings(globals, CHAT_FORMAT_STRINGS)?;

    // Guild news constants
    register_ints(globals, GUILD_NEWS_CONSTANTS)?;

    // Duration strings
    register_strings(globals, DURATION_STRINGS)?;

    // HUD edit mode strings
    register_strings(globals, HUD_EDIT_MODE_STRINGS)?;

    // Unit frame strings
    register_strings(globals, UNIT_FRAME_STRINGS)?;

    // Tooltip related
    register_tooltip_colors(lua, globals)?;
    register_strings(globals, TOOLTIP_STRINGS)?;

    // Item quality colors table
    register_item_quality_colors(lua, globals)?;

    // Class name tables
    register_class_name_tables(lua, globals)?;

    // Font paths
    register_strings(globals, FONT_PATH_STRINGS)?;

    // LFG strings
    register_strings(globals, LFG_STRINGS)?;
    register_strings(globals, LFG_TYPE_STRINGS)?;
    register_strings(globals, LFG_ERROR_STRINGS)?;

    // Stat strings
    register_strings(globals, STAT_STRINGS)?;
    register_strings(globals, ITEM_MOD_STRINGS)?;

    // Slash commands
    register_strings(globals, SLASH_COMMAND_STRINGS)?;

    // Error strings
    register_strings(globals, LOOT_ERROR_STRINGS)?;
    register_strings(globals, SPELL_ERROR_STRINGS)?;

    // Instance strings
    register_strings(globals, INSTANCE_STRINGS)?;

    // Objective tracker strings
    register_strings(globals, OBJECTIVE_TRACKER_STRINGS)?;

    // Character strings
    register_strings(globals, CHARACTER_STRINGS)?;

    // Achievement strings
    register_strings(globals, ACHIEVEMENT_STRINGS)?;

    // Spellbook strings
    register_strings(globals, SPELLBOOK_STRINGS)?;

    // Dungeon difficulty strings
    register_strings(globals, DUNGEON_DIFFICULTY_STRINGS)?;

    // Icon list
    register_icon_list(lua, globals)?;

    // Font color codes
    register_strings(globals, FONT_COLOR_CODE_STRINGS)?;

    // Time strings
    register_strings(globals, TIME_STRINGS)?;

    Ok(())
}
