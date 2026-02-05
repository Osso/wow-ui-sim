//! Locale, region, and build info WoW API functions.

use mlua::{Lua, Result, Value};

/// Register locale, region, and build-related global functions.
pub fn register_locale_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // GetBuildInfo() - Return game version info
    let get_build_info = lua.create_function(|lua, ()| {
        // Returns: version, build, date, tocversion, localizedVersion, buildType
        // 11.0.7 is The War Within (TWW)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.7")?), // version
            Value::String(lua.create_string("58238")?),  // build
            Value::String(lua.create_string("Jan 7 2025")?), // date
            Value::Integer(110007),                      // tocversion
            Value::String(lua.create_string("11.0.7")?), // localizedVersion
            Value::String(lua.create_string("Release")?), // buildType
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // GetRealmName() - Return mock realm name
    let get_realm_name =
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimulatedRealm")?)))?;
    globals.set("GetRealmName", get_realm_name)?;

    // GetNormalizedRealmName() - Return mock normalized realm name
    let get_normalized_realm_name =
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimulatedRealm")?)))?;
    globals.set("GetNormalizedRealmName", get_normalized_realm_name)?;

    // GetRealmID() - Return mock realm ID
    globals.set("GetRealmID", lua.create_function(|_, ()| Ok(1i32))?)?;

    // GetLocale() - Return mock locale
    let get_locale =
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("enUS")?)))?;
    globals.set("GetLocale", get_locale)?;

    // GetCurrentRegion() - Return region ID
    // 1=US, 2=Korea, 3=Europe, 4=Taiwan, 5=China
    globals.set(
        "GetCurrentRegion",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;

    // GetCurrentRegionName() - Return region name
    globals.set(
        "GetCurrentRegionName",
        lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("US")?)))?,
    )?;

    // Client type checks
    globals.set("IsMacClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsWindowsClient", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("IsLinuxClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsTestBuild", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPTRClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsTrialAccount", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set(
        "IsVeteranTrialAccount",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set(
        "IsPublicTestClient",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("IsPublicBuild", lua.create_function(|_, ()| Ok(true))?)?;

    // GetExpansionLevel() - Returns the current expansion level (0-10)
    // 0=Classic, 1=TBC, 2=WotLK, 3=Cata, 4=MoP, 5=WoD, 6=Legion, 7=BfA, 8=SL, 9=DF, 10=TWW
    globals.set(
        "GetExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?, // The War Within
    )?;

    // GetMaxLevelForPlayerExpansion() - Returns max level for player's expansion
    globals.set(
        "GetMaxLevelForPlayerExpansion",
        lua.create_function(|_, ()| Ok(80))?, // TWW max level
    )?;

    // GetMaxLevelForExpansionLevel(expansion) - Returns max level for given expansion
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

    // GetServerExpansionLevel() - Server's expansion level
    globals.set(
        "GetServerExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetClientDisplayExpansionLevel() - Client display expansion
    globals.set(
        "GetClientDisplayExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetMinimumExpansionLevel() - Minimum expansion level for trial accounts
    globals.set(
        "GetMinimumExpansionLevel",
        lua.create_function(|_, ()| Ok(0))?,
    )?;

    // GetMaximumExpansionLevel() - Maximum expansion level
    globals.set(
        "GetMaximumExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetAccountExpansionLevel() - Account's expansion level
    globals.set(
        "GetAccountExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetAutoCompleteRealms() - Return empty table of auto-complete realms
    globals.set(
        "GetAutoCompleteRealms",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;

    // Expansion level constants (LE_EXPANSION_*)
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

    // C_Glue namespace (glue screen utilities)
    let c_glue = lua.create_table()?;
    let is_on_glue_screen = lua.create_function(|_, ()| Ok(false))?;
    c_glue.set("IsOnGlueScreen", is_on_glue_screen)?;
    globals.set("C_Glue", c_glue)?;

    // InGlue() - Check if in glue screen (character selection). Always false in sim.
    globals.set("InGlue", lua.create_function(|_, ()| Ok(false))?)?;

    // IsLoggedIn() - Check if player is logged in (false until PLAYER_LOGIN fires)
    globals.set("IsLoggedIn", lua.create_function(|_, ()| Ok(false))?)?;

    Ok(())
}
