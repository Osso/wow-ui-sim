//! Retail/Forever client identity and expansion globals.
//!
//! Values come from the pinned client profile, generated global strings, and
//! loaded Blizzard constants. Native value parity is INFERRED R88g9 unless a
//! source comment states otherwise.

use crate::client_profile::{
    CLIENT_BUILD_DATE, CLIENT_DISPLAY_EXPANSION_LEVEL, CLIENT_INTERFACE, SIMULATOR_LOCALE,
    client_identity,
};
use crate::lua_api::methods::{create_string, table_get};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetBuildInfo", get_build_info)?;
    LuaApiMut::register_function(lua, "GetExpansionLevel", get_expansion_level)?;
    LuaApiMut::register_function(
        lua,
        "GetClientDisplayExpansionLevel",
        get_client_display_expansion_level,
    )?;
    LuaApiMut::register_function(lua, "GetAccountExpansionLevel", get_account_expansion_level)?;
    LuaApiMut::register_function(lua, "GetMaxPlayerLevel", get_max_player_level)?;
    LuaApiMut::register_function(
        lua,
        "GetMaxLevelForExpansionLevel",
        get_max_level_for_expansion_level,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetMaxLevelForPlayerExpansion",
        get_max_level_for_player_expansion,
    )?;
    LuaApiMut::register_function(lua, "GetLocale", get_locale)?;
    LuaApiMut::register_function(lua, "IsMacClient", is_mac_client)?;
    LuaApiMut::register_function(lua, "IsWindowsClient", is_windows_client)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_build_info(state: &mut LuaState) -> LuaResult<u32> {
    let (version, build) =
        client_identity().map_err(|error| rilua::runtime_error(error.to_string()))?;
    let version = create_string(state, &version);
    let build = create_string(state, &build);
    let date = create_string(state, CLIENT_BUILD_DATE);
    let locale = create_string(state, "");
    let separator = create_string(state, " ");
    state.push(version);
    state.push(build);
    state.push(date);
    state.push(Val::Num(f64::from(CLIENT_INTERFACE)));
    state.push(locale);
    state.push(separator);
    Ok(6)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_expansion_level(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(f64::from(
        crate::client_profile::ACTIVE.expansion_level(),
    )));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_client_display_expansion_level(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(f64::from(CLIENT_DISPLAY_EXPANSION_LEVEL)));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_account_expansion_level(state: &mut LuaState) -> LuaResult<u32> {
    get_client_display_expansion_level(state)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_max_player_level(state: &mut LuaState) -> LuaResult<u32> {
    let max_level = global_max_player_level(state);
    state.push(Val::Num(max_level));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_max_level_for_expansion_level(state: &mut LuaState) -> LuaResult<u32> {
    let expansion_level = i32::from_stack(state, 1)?;
    let max_level = max_level_for_expansion(state, expansion_level);
    state.push(Val::Num(max_level));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_max_level_for_player_expansion(state: &mut LuaState) -> LuaResult<u32> {
    let max_level = max_level_for_expansion(state, CLIENT_DISPLAY_EXPANSION_LEVEL as i32);
    state.push(Val::Num(max_level));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_locale(state: &mut LuaState) -> LuaResult<u32> {
    let locale = create_string(state, SIMULATOR_LOCALE);
    state.push(locale);
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_mac_client(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(cfg!(target_os = "macos")));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_windows_client(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(cfg!(target_os = "windows")));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn core_max_player_level() -> f64 {
    crate::lua_api::globals::strings::string_data::GAME_INT_CONSTANTS
        .iter()
        .find(|(name, _)| *name == "MAX_PLAYER_LEVEL")
        .map_or(0.0, |(_, value)| f64::from(*value))
}

fn global_max_player_level(state: &mut LuaState) -> f64 {
    let value = table_get(state, Val::Table(state.global), "MAX_PLAYER_LEVEL");
    numeric_level(value)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn max_level_for_expansion(state: &mut LuaState, expansion_level: i32) -> f64 {
    let levels = table_get(state, Val::Table(state.global), "MAX_PLAYER_LEVEL_TABLE");
    let Val::Table(levels) = levels else {
        return pinned_expansion_level_cap(expansion_level)
            .unwrap_or_else(|| global_max_player_level(state));
    };
    let level = state
        .gc
        .tables
        .get(levels)
        .and_then(|table| Some(table.get_int(i64::from(expansion_level))))
        .filter(|value| matches!(value, Val::Num(_)));
    level.map_or_else(
        || {
            pinned_expansion_level_cap(expansion_level)
                .unwrap_or_else(|| global_max_player_level(state))
        },
        numeric_level,
    )
}

fn pinned_expansion_level_cap(expansion_level: i32) -> Option<f64> {
    crate::client_profile::EXPANSION_LEVEL_CAPS
        .iter()
        .find(|(level, _)| *level == expansion_level)
        .map(|(_, cap)| f64::from(*cap))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn numeric_level(value: Val) -> f64 {
    match value {
        Val::Num(number) => number,
        _ => core_max_player_level(),
    }
}
