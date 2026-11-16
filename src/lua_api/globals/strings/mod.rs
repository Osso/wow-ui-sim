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
    let secure_global =
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
            Some(Val::Table(
                crate::lua_api::globals::security::secure_env_table(state)?,
            ))
        } else {
            None
        };
    register_global_strings(state, global, secure_global, existing_policy);
    register_integer_constants(state, global, secure_global, existing_policy);
    register_float_constants(state, global, existing_policy);
    register_string_constants(state, global, existing_policy);
    Ok(())
}

fn register_global_strings(
    state: &mut LuaState,
    global: Val,
    secure_global: Option<Val>,
    existing_policy: ExistingGlobalPolicy,
) {
    for (name, value) in crate::global_strings::GLOBAL_STRINGS.entries() {
        let preserve_existing = match existing_policy {
            ExistingGlobalPolicy::Preserve => table_get(state, global, name) != Val::Nil,
            ExistingGlobalPolicy::Replace => {
                cfg!(feature = "client-wowforever")
                    && matches!(table_get(state, global, name), Val::Function(_))
            }
        };
        if preserve_existing {
            continue;
        }
        let resolved = resolve_lua_escapes(value);
        let lua_value = create_string(state, &resolved);
        table_set(state, global, name, lua_value);
        if let Some(secure_global) = secure_global {
            // INFERRED (R88ac.ac2): the PHF source has no native Flags field.
            table_set(state, secure_global, name, lua_value);
        }
    }
}

fn register_integer_constants(
    state: &mut LuaState,
    global: Val,
    secure_global: Option<Val>,
    existing_policy: ExistingGlobalPolicy,
) {
    for defs in INT_DEFS {
        for &(name, value) in defs.iter() {
            let supported = should_register_int_constant(name, ACTIVE_RETAIL_API_EPOCH);
            if supported && should_set_global(state, global, name, existing_policy) {
                let value = Val::Num(value as f64);
                table_set(state, global, name, value);
                if is_r88ac_verified_le_game_global(name) {
                    if let Some(secure_global) = secure_global {
                        table_set(state, secure_global, name, value);
                    }
                }
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

// VERIFIED (R88ac.ac3): all 42 modeled LE_GAME_* names in INT_DEFS occur in the
// supplement's Game/flag-3 rows; no other LE_GAME_* name is in these arrays.
fn is_r88ac_verified_le_game_global(name: &str) -> bool {
    name.starts_with("LE_GAME_")
        && string_data::GAME_ERROR_STRINGS
            .iter()
            .any(|(known_name, _)| *known_name == name)
}

fn should_register_int_constant(name: &str, epoch: RetailApiEpoch) -> bool {
    !(epoch == RetailApiEpoch::Retail12_0_0 && name == "LE_FRAME_TUTORIAL_LINK_TRANSMOG_OUTFIT")
}

#[cfg(test)]
mod tests {
    use super::{register_all_ui_strings, restore_missing_ui_strings};

    #[test]
    fn ui_string_restore_preserves_all_non_nil_values_and_fills_deleted_keys() {
        let mut lua = rilua::Lua::new().unwrap();
        crate::lua_api::globals::security::create_secure_environment(&mut lua).unwrap();
        register_all_ui_strings(&mut lua).unwrap();
        lua.exec(
            r#"
            ACCEPT = false
            CANCEL = 19
            YES = function() return 'custom' end
            NO = {}
            SavedYes, SavedNo = YES, NO
        "#,
        )
        .unwrap();
        restore_missing_ui_strings(&mut lua).unwrap();
        lua.exec(
            r#"
            assert(ACCEPT == false)
            assert(CANCEL == 19)
            assert(rawequal(YES, SavedYes) and YES() == 'custom')
            assert(rawequal(NO, SavedNo))
            ACCEPT, CANCEL = nil, nil
        "#,
        )
        .unwrap();
        restore_missing_ui_strings(&mut lua).unwrap();
        lua.exec("assert(ACCEPT == 'Accept'); assert(CANCEL == 'Cancel')")
            .unwrap();
    }

    #[test]
    fn ui_string_replace_preserves_functions_only_on_forever() {
        let mut lua = rilua::Lua::new().unwrap();
        crate::lua_api::globals::security::create_secure_environment(&mut lua).unwrap();
        lua.exec(
            r#"
            ACCEPT = function() return 'custom' end
            SavedAccept = ACCEPT
            CANCEL, YES, NO = false, 19, {}
        "#,
        )
        .unwrap();
        register_all_ui_strings(&mut lua).unwrap();
        if cfg!(feature = "client-wowforever") {
            lua.exec("assert(rawequal(ACCEPT, SavedAccept)); assert(ACCEPT() == 'custom')")
                .unwrap();
        } else {
            lua.exec("assert(ACCEPT == 'Accept')").unwrap();
        }
        lua.exec("assert(CANCEL == 'Cancel'); assert(YES == 'Yes'); assert(NO == 'No')")
            .unwrap();
    }

    #[test]
    fn ui_string_registration_reads_raw_globals_without_index_callbacks() {
        let mut lua = rilua::Lua::new().unwrap();
        crate::lua_api::globals::security::create_secure_environment(&mut lua).unwrap();
        lua.exec(
            r#"
            IndexCalls = 0
            setmetatable(_G, { __index = function()
                IndexCalls = IndexCalls + 1
                return 'virtual value'
            end })
        "#,
        )
        .unwrap();
        restore_missing_ui_strings(&mut lua).unwrap();
        lua.exec("assert(IndexCalls == 0); assert(rawget(_G, 'ACCEPT') == 'Accept')")
            .unwrap();
        lua.exec("ACCEPT = nil").unwrap();
        register_all_ui_strings(&mut lua).unwrap();
        lua.exec("assert(IndexCalls == 0); assert(rawget(_G, 'ACCEPT') == 'Accept')")
            .unwrap();
    }
}
