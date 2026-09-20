//! UI string registration for the rilua global surface.

pub mod string_data;

use crate::client_profile::{ACTIVE_RETAIL_API_EPOCH, RetailApiEpoch};
use crate::loader::helpers::resolve_lua_escapes;
use crate::lua_api::methods::{create_string, table_get, table_set};
use rilua::LuaApiMut;
use rilua::Val;
use rilua::vm::state::LuaState;

#[derive(Clone, Copy)]
enum ExistingGlobalPolicy {
    Replace,
    Preserve,
}

pub fn register_all_ui_strings(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_ui_strings(lua, ExistingGlobalPolicy::Replace)
}

pub fn restore_missing_ui_strings(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_ui_strings(lua, ExistingGlobalPolicy::Preserve)
}

fn register_ui_strings(
    lua: &mut rilua::Lua,
    existing_policy: ExistingGlobalPolicy,
) -> crate::Result<()> {
    let state = lua.state_mut();
    let global = Val::Table(state.global);
    register_global_strings(state, global, existing_policy);
    register_integer_constants(state, global, existing_policy);
    register_float_constants(state, global, existing_policy);
    register_string_constants(state, global, existing_policy);
    Ok(())
}

fn register_global_strings(
    state: &mut LuaState,
    global: Val,
    existing_policy: ExistingGlobalPolicy,
) {
    for (name, value) in crate::global_strings::GLOBAL_STRINGS.entries() {
        let existing = table_get(state, global, name);
        let preserve_function =
            cfg!(feature = "client-wowforever") && matches!(existing, Val::Function(_));
        if !preserve_function && should_set_global(state, global, name, existing_policy) {
            let resolved = resolve_lua_escapes(value);
            let lua_value = create_string(state, &resolved);
            table_set(state, global, name, lua_value);
        }
    }
}

fn register_integer_constants(
    state: &mut LuaState,
    global: Val,
    existing_policy: ExistingGlobalPolicy,
) {
    for defs in INT_DEFS {
        for &(name, value) in defs.iter() {
            let supported = should_register_int_constant(name, ACTIVE_RETAIL_API_EPOCH);
            if supported && should_set_global(state, global, name, existing_policy) {
                table_set(state, global, name, Val::Num(value as f64));
            }
        }
    }
}

fn register_float_constants(
    state: &mut LuaState,
    global: Val,
    existing_policy: ExistingGlobalPolicy,
) {
    for defs in FLOAT_DEFS {
        for &(name, value) in defs.iter() {
            if should_set_global(state, global, name, existing_policy) {
                table_set(state, global, name, Val::Num(value));
            }
        }
    }
}

fn register_string_constants(
    state: &mut LuaState,
    global: Val,
    existing_policy: ExistingGlobalPolicy,
) {
    for defs in STRING_DEFS {
        for &(name, value) in defs.iter() {
            if should_set_global(state, global, name, existing_policy) {
                let resolved = resolve_lua_escapes(value);
                let lua_value = create_string(state, &resolved);
                table_set(state, global, name, lua_value);
            }
        }
    }
}

fn should_set_global(
    state: &mut LuaState,
    global: Val,
    name: &str,
    existing_policy: ExistingGlobalPolicy,
) -> bool {
    match existing_policy {
        ExistingGlobalPolicy::Replace => true,
        ExistingGlobalPolicy::Preserve => matches!(table_get(state, global, name), Val::Nil),
    }
}

const INT_DEFS: &[&[crate::lua_api::globals::strings::string_data::IntDef]] = &[
    string_data::GAME_INT_CONSTANTS,
    string_data::EXPANSION_CONSTANTS,
    string_data::AUTOCOMPLETE_CONSTANTS,
    string_data::INVENTORY_SLOT_CONSTANTS,
    string_data::GUILD_NEWS_CONSTANTS,
    string_data::COMBAT_LOG_RAID_TARGET_CONSTANTS,
    string_data::TOTEM_SLOT_CONSTANTS,
    string_data::LFG_CATEGORY_CONSTANTS,
    #[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
    string_data::LAIR_LFG_CATEGORY_CONSTANTS,
    #[cfg(feature = "client-wowforever")]
    string_data::FOREVER_LFG_CATEGORY_COUNT,
    string_data::GAME_ERROR_STRINGS,
    string_data::ACTIONBAR_STATE_CONSTANTS,
    string_data::FRAME_TUTORIAL_CONSTANTS,
];

const FLOAT_DEFS: &[&[crate::lua_api::globals::strings::string_data::FloatDef]] = &[
    string_data::TAXI_FLOAT_CONSTANTS,
    string_data::MOVEMENT_FLOAT_CONSTANTS,
];

const STRING_DEFS: &[&[crate::lua_api::globals::strings::string_data::StringDef]] = &[
    string_data::FONT_COLOR_CODE_STRINGS,
    #[cfg(feature = "retail-12-1-0")]
    string_data::RETAIL_12_1_GLOBAL_STRINGS,
];

fn should_register_int_constant(name: &str, epoch: RetailApiEpoch) -> bool {
    !(epoch == RetailApiEpoch::Retail12_0_0 && name == "LE_FRAME_TUTORIAL_LINK_TRANSMOG_OUTFIT")
}
