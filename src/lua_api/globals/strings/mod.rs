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
    register_binding_getters(lua, globals)?;
    register_binding_setters(lua, globals)?;
    register_binding_persistence(lua, globals)?;
    Ok(())
}

/// Register keybinding query functions (Get*).
fn register_binding_getters(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    register_binding_key_lookups(lua, globals)?;
    register_binding_enumeration(lua, globals)?;
    register_binding_text_helpers(lua, globals)?;
    Ok(())
}

/// GetBindingKey, GetBindingKeyForAction, GetBindingAction — key↔action lookups.
fn register_binding_key_lookups(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    use super::super::keybindings;

    globals.set(
        "GetBindingKey",
        lua.create_function(|lua, action: String| {
            let (k1, k2) = keybindings::get_binding_key(lua, &action)?;
            let mut vals = Vec::new();
            match k1 {
                Some(k) => vals.push(Value::String(lua.create_string(&k)?)),
                None => vals.push(Value::Nil),
            }
            if let Some(k) = k2 {
                vals.push(Value::String(lua.create_string(&k)?));
            }
            Ok(mlua::MultiValue::from_vec(vals))
        })?,
    )?;
    globals.set(
        "GetBindingKeyForAction",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let action: Option<String> = args.into_iter().next().and_then(|v| {
                if let Value::String(s) = v { s.to_str().ok().map(|s| s.to_string()) } else { None }
            });
            let Some(action) = action else { return Ok(Value::Nil) };
            let (k1, _) = keybindings::get_binding_key(lua, &action)?;
            match k1 {
                Some(k) => Ok(Value::String(lua.create_string(&k)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;
    globals.set(
        "GetBindingAction",
        lua.create_function(|lua, (key, _check_override): (String, Option<bool>)| {
            match keybindings::get_binding_action(lua, &key)? {
                Some(a) => Ok(Value::String(lua.create_string(&a)?)),
                None => Ok(Value::String(lua.create_string("")?)),
            }
        })?,
    )?;
    Ok(())
}

/// GetBinding, GetNumBindings, GetCurrentBindingSet — enumerate all bindings.
fn register_binding_enumeration(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    use super::super::keybindings;

    globals.set(
        "GetBinding",
        lua.create_function(|lua, index: i32| {
            let (action, _header, key1, key2) = keybindings::get_binding_at(lua, index)?;
            let mut vals = vec![
                match action { Some(a) => Value::String(lua.create_string(&a)?), None => Value::Nil },
                Value::Nil, // header
            ];
            if let Some(k) = key1 {
                vals.push(Value::String(lua.create_string(&k)?));
            }
            if let Some(k) = key2 {
                vals.push(Value::String(lua.create_string(&k)?));
            }
            Ok(mlua::MultiValue::from_vec(vals))
        })?,
    )?;
    globals.set(
        "GetNumBindings",
        lua.create_function(|lua, ()| keybindings::get_num_bindings(lua))?,
    )?;
    globals.set("GetCurrentBindingSet", lua.create_function(|_, ()| Ok(1))?)?;
    Ok(())
}

/// GetBindingText — display-friendly key name.
fn register_binding_text_helpers(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    globals.set(
        "GetBindingText",
        lua.create_function(
            |lua, (key, _prefix, _abbrev): (Option<String>, Option<Value>, Option<Value>)| {
                match key {
                    Some(k) => Ok(Value::String(lua.create_string(&k)?)),
                    None => Ok(Value::String(lua.create_string("")?)),
                }
            },
        )?,
    )?;
    Ok(())
}

/// Register keybinding assignment functions (Set*).
fn register_binding_setters(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    use super::super::keybindings;

    globals.set(
        "SetBinding",
        lua.create_function(|lua, (key, action): (String, Option<String>)| {
            keybindings::set_binding(lua, &key, action.as_deref())
        })?,
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
    Ok(())
}

/// Register binding persistence functions (Save/Load/GetCurrentSet).
fn register_binding_persistence(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    globals.set("GetCurrentBindingSet", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("SaveBindings", lua.create_function(|_, _which: i32| Ok(()))?)?;
    globals.set("LoadBindings", lua.create_function(|_, _which: i32| Ok(()))?)?;
    Ok(())
}

/// Registers tooltip colors (requires Lua for table creation).
pub fn register_tooltip_colors(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let (r, g, b, a) = TOOLTIP_DEFAULT_COLOR;
    globals.set("TOOLTIP_DEFAULT_COLOR", make_color_table(lua, r, g, b, a)?)?;

    let (r, g, b, a) = TOOLTIP_DEFAULT_BG_COLOR;
    globals.set("TOOLTIP_DEFAULT_BACKGROUND_COLOR", make_color_table(lua, r, g, b, a)?)?;

    Ok(())
}

/// Create a color table with r/g/b/a fields and GetRGB/GetRGBA/WrapTextInColorCode methods.
fn make_color_table(lua: &Lua, r: f64, g: f64, b: f64, a: f64) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("r", r)?;
    t.set("g", g)?;
    t.set("b", b)?;
    t.set("a", a)?;
    t.set("GetRGB", lua.create_function(|_, this: mlua::Table| {
        Ok((this.get::<f64>("r")?, this.get::<f64>("g")?, this.get::<f64>("b")?))
    })?)?;
    t.set("GetRGBA", lua.create_function(|_, this: mlua::Table| {
        Ok((this.get::<f64>("r")?, this.get::<f64>("g")?,
            this.get::<f64>("b")?, this.get::<f64>("a")?))
    })?)?;
    t.set("GenerateHexColor", lua.create_function(|lua, this: mlua::Table| {
        let r: f64 = this.get("r")?;
        let g: f64 = this.get("g")?;
        let b: f64 = this.get("b")?;
        let hex = format!("{:02x}{:02x}{:02x}",
            (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        Ok(mlua::Value::String(lua.create_string(&hex)?))
    })?)?;
    t.set("WrapTextInColorCode", lua.create_function(|lua, (this, text): (mlua::Table, String)| {
        let r: f64 = this.get("r")?;
        let g: f64 = this.get("g")?;
        let b: f64 = this.get("b")?;
        let hex = format!("{:02x}{:02x}{:02x}",
            (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        Ok(mlua::Value::String(lua.create_string(format!("|cff{}{}|r", hex, text))?))
    })?)?;
    Ok(t)
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
    register_generated_global_strings(globals)?;
    register_game_constants(globals)?;
    register_keybinding_functions(lua, globals)?;
    register_ui_strings(globals)?;
    register_item_strings(globals)?;
    register_combat_strings(globals)?;
    register_economy_strings(globals)?;
    register_misc_string_groups(globals)?;
    register_lua_table_constants(lua, globals)?;
    register_remaining_strings(globals)?;
    Ok(())
}

/// Register generated global strings (20k+ from WoW CSV exports).
fn register_generated_global_strings(globals: &mlua::Table) -> Result<()> {
    for (name, value) in &crate::global_strings::GLOBAL_STRINGS {
        globals.set(*name, *value)?;
    }
    register_strings(globals, ERROR_STRINGS)?;
    Ok(())
}

/// Register game, expansion, inventory, taxi, and keyboard constants.
fn register_game_constants(globals: &mlua::Table) -> Result<()> {
    register_ints(globals, GAME_INT_CONSTANTS)?;
    register_strings(globals, GAME_STRING_CONSTANTS)?;
    register_ints(globals, EXPANSION_CONSTANTS)?;
    register_ints(globals, AUTOCOMPLETE_CONSTANTS)?;
    register_ints(globals, INVENTORY_SLOT_CONSTANTS)?;
    register_strings(globals, RAID_TARGET_STRINGS)?;
    register_floats(globals, TAXI_FLOAT_CONSTANTS)?;
    register_strings(globals, KEYBOARD_MODIFIER_STRINGS)?;
    register_ints(globals, TOTEM_SLOT_CONSTANTS)?;
    register_ints(globals, LFG_CATEGORY_CONSTANTS)?;
    register_strings(globals, GAME_ERROR_STRINGS)?;
    register_ints(globals, ACTIONBAR_STATE_CONSTANTS)?;
    register_ints(globals, FRAME_TUTORIAL_CONSTANTS)?;
    Ok(())
}

/// Register UI category and button strings.
fn register_ui_strings(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, UI_CATEGORY_STRINGS)?;
    register_strings(globals, UI_BUTTON_STRINGS)?;
    Ok(())
}

/// Register item-related string groups.
fn register_item_strings(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, ITEM_STRINGS)?;
    register_strings(globals, SOCKET_STRINGS)?;
    register_strings(globals, ITEM_BINDING_STRINGS)?;
    register_strings(globals, ITEM_REQUIREMENT_STRINGS)?;
    register_strings(globals, ITEM_UPGRADE_STRINGS)?;
    register_strings(globals, BINDING_HEADER_STRINGS)?;
    register_strings(globals, BINDING_NAME_STRINGS)?;
    Ok(())
}

/// Register combat, duel, and related strings.
fn register_combat_strings(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, MISC_UI_STRINGS)?;
    register_strings(globals, DUEL_STRINGS)?;
    register_strings(globals, COMBAT_TEXT_STRINGS)?;
    register_ints(globals, COMBAT_LOG_RAID_TARGET_CONSTANTS)?;
    Ok(())
}

/// Register loot, currency, XP, quest, chat, and guild strings.
fn register_economy_strings(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, LOOT_STRINGS)?;
    register_strings(globals, CURRENCY_STRINGS)?;
    register_strings(globals, XP_QUEST_STRINGS)?;
    register_strings(globals, CHAT_FORMAT_STRINGS)?;
    register_ints(globals, GUILD_NEWS_CONSTANTS)?;
    Ok(())
}

/// Register duration, HUD, unit frame, and tooltip strings.
fn register_misc_string_groups(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, DURATION_STRINGS)?;
    register_strings(globals, HUD_EDIT_MODE_STRINGS)?;
    register_strings(globals, UNIT_FRAME_STRINGS)?;
    register_strings(globals, TOOLTIP_STRINGS)?;
    Ok(())
}

/// Register Lua table-based constants (require table creation).
fn register_lua_table_constants(lua: &Lua, globals: &mlua::Table) -> Result<()> {
    register_tooltip_colors(lua, globals)?;
    register_item_quality_colors(lua, globals)?;
    register_class_name_tables(lua, globals)?;
    register_icon_list(lua, globals)?;
    Ok(())
}

/// Register remaining string groups (fonts, LFG, stats, etc.).
fn register_remaining_strings(globals: &mlua::Table) -> Result<()> {
    register_strings(globals, FONT_PATH_STRINGS)?;
    register_strings(globals, LFG_STRINGS)?;
    register_strings(globals, LFG_TYPE_STRINGS)?;
    register_strings(globals, LFG_ERROR_STRINGS)?;
    register_strings(globals, STAT_STRINGS)?;
    register_strings(globals, ITEM_MOD_STRINGS)?;
    register_strings(globals, SLASH_COMMAND_STRINGS)?;
    register_strings(globals, LOOT_ERROR_STRINGS)?;
    register_strings(globals, SPELL_ERROR_STRINGS)?;
    register_strings(globals, INSTANCE_STRINGS)?;
    register_strings(globals, OBJECTIVE_TRACKER_STRINGS)?;
    register_strings(globals, CHARACTER_STRINGS)?;
    register_strings(globals, ACHIEVEMENT_STRINGS)?;
    register_strings(globals, SPELLBOOK_STRINGS)?;
    register_strings(globals, DUNGEON_DIFFICULTY_STRINGS)?;
    register_strings(globals, FONT_COLOR_CODE_STRINGS)?;
    register_strings(globals, TIME_STRINGS)?;
    Ok(())
}
