//! Locale, region, and build info WoW API functions.

use mlua::{Lua, Result, Value};

/// Register locale, region, and build-related global functions.
pub fn register_locale_api(lua: &Lua) -> Result<()> {
    register_build_info(lua)?;
    register_realm_functions(lua)?;
    register_locale_and_region(lua)?;
    register_client_type_checks(lua)?;
    register_expansion_functions(lua)?;
    register_expansion_constants(lua)?;
    register_glue_functions(lua)?;
    Ok(())
}

/// Register `GetBuildInfo()` - game version info.
fn register_build_info(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let get_build_info = lua.create_function(|lua, ()| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.7")?),
            Value::String(lua.create_string("58238")?),
            Value::String(lua.create_string("Jan 7 2025")?),
            Value::Integer(110007),
            Value::String(lua.create_string("11.0.7")?),
            Value::String(lua.create_string("Release")?),
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    Ok(())
}

/// Register realm-related functions.
fn register_realm_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetRealmName",
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimulatedRealm")?)))?,
    )?;
    globals.set(
        "GetNormalizedRealmName",
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimulatedRealm")?)))?,
    )?;
    globals.set("GetRealmID", lua.create_function(|_, ()| Ok(1i32))?)?;

    Ok(())
}

/// Register locale and region functions.
fn register_locale_and_region(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetLocale",
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("enUS")?)))?,
    )?;
    globals.set("GetCurrentRegion", lua.create_function(|_, ()| Ok(1i32))?)?;
    globals.set(
        "GetCurrentRegionName",
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("US")?)))?,
    )?;

    Ok(())
}

/// Register client type check functions.
fn register_client_type_checks(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("IsMacClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsWindowsClient", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("IsLinuxClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsTestBuild", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPTRClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsTrialAccount", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsVeteranTrialAccount", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPublicTestClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPublicBuild", lua.create_function(|_, ()| Ok(true))?)?;

    Ok(())
}

/// Register expansion level functions.
fn register_expansion_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("GetExpansionLevel", lua.create_function(|_, ()| Ok(10))?)?;
    globals.set("GetMaxLevelForPlayerExpansion", lua.create_function(|_, ()| Ok(80))?)?;
    globals.set(
        "GetMaxLevelForExpansionLevel",
        lua.create_function(|_, expansion: i32| {
            let max_level = match expansion {
                0 => 60,  // Classic
                1 => 70,  // TBC
                2 => 80,  // WotLK
                3 => 85,  // Cata
                4 => 90,  // MoP
                5 => 100, // WoD
                6 => 110, // Legion
                7 => 120, // BfA
                8 => 60,  // Shadowlands (level squish)
                9 => 70,  // Dragonflight
                10 => 80, // The War Within
                _ => 80,
            };
            Ok(max_level)
        })?,
    )?;
    globals.set("GetServerExpansionLevel", lua.create_function(|_, ()| Ok(10))?)?;
    globals.set("GetClientDisplayExpansionLevel", lua.create_function(|_, ()| Ok(10))?)?;
    globals.set("GetMinimumExpansionLevel", lua.create_function(|_, ()| Ok(0))?)?;
    globals.set("GetMaximumExpansionLevel", lua.create_function(|_, ()| Ok(10))?)?;
    globals.set("GetAccountExpansionLevel", lua.create_function(|_, ()| Ok(10))?)?;
    globals.set("GetAutoCompleteRealms", lua.create_function(|lua, ()| lua.create_table())?)?;

    Ok(())
}

/// Register expansion level constants (LE_EXPANSION_*).
fn register_expansion_constants(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("LE_EXPANSION_CLASSIC", 0)?;
    globals.set("LE_EXPANSION_BURNING_CRUSADE", 1)?;
    globals.set("LE_EXPANSION_WRATH_OF_THE_LICH_KING", 2)?;
    globals.set("LE_EXPANSION_CATACLYSM", 3)?;
    globals.set("LE_EXPANSION_MISTS_OF_PANDARIA", 4)?;
    globals.set("LE_EXPANSION_WARLORDS_OF_DRAENOR", 5)?;
    globals.set("LE_EXPANSION_LEGION", 6)?;
    globals.set("LE_EXPANSION_BATTLE_FOR_AZEROTH", 7)?;
    globals.set("LE_EXPANSION_SHADOWLANDS", 8)?;
    globals.set("LE_EXPANSION_DRAGONFLIGHT", 9)?;
    globals.set("LE_EXPANSION_WAR_WITHIN", 10)?;
    globals.set("LE_EXPANSION_LEVEL_CURRENT", 10)?;

    Ok(())
}

/// Register glue screen and login state functions.
fn register_glue_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let c_glue = lua.create_table()?;
    c_glue.set("IsOnGlueScreen", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("C_Glue", c_glue)?;

    globals.set("InGlue", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsLoggedIn", lua.create_function(|_, ()| Ok(false))?)?;

    Ok(())
}
