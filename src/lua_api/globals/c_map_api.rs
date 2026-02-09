//! C_Map namespace and related map/location API functions.
//!
//! Contains map, exploration, navigation, and location-related API functions.

use mlua::{Lua, Result, Value};

/// Register C_Map namespace and map-related functions.
pub fn register_c_map_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("C_Map", register_c_map(lua)?)?;
    register_zone_text_functions(lua)?;
    globals.set("UiMapPoint", register_ui_map_point(lua)?)?;
    globals.set("C_MapExplorationInfo", register_c_map_exploration(lua)?)?;
    globals.set("C_DateAndTime", register_c_date_and_time(lua)?)?;
    globals.set("C_Minimap", register_c_minimap(lua)?)?;
    globals.set("C_Navigation", register_c_navigation(lua)?)?;
    globals.set("C_TaxiMap", register_c_taxi_map(lua)?)?;

    Ok(())
}

/// C_Map namespace - map and area information.
fn register_c_map(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetAreaInfo",
        lua.create_function(|lua, area_id: i32| {
            Ok(Value::String(lua.create_string(format!("Area_{}", area_id))?))
        })?,
    )?;
    t.set(
        "GetMapInfo",
        lua.create_function(|lua, map_id: i32| {
            let info = lua.create_table()?;
            info.set("mapID", map_id)?;
            info.set("name", format!("Map_{}", map_id))?;
            info.set("mapType", 3)?;
            info.set("parentMapID", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    t.set(
        "GetBestMapForUnit",
        lua.create_function(|_, _unit: String| Ok(Value::Integer(1)))?,
    )?;
    t.set("GetPlayerMapPosition", lua.create_function(create_player_map_position)?)?;
    t.set(
        "GetMapChildrenInfo",
        lua.create_function(|lua, (_map_id, _map_type, _all_descendants): (i32, Option<i32>, Option<bool>)| {
            lua.create_table()
        })?,
    )?;
    t.set("GetWorldPosFromMapPos", lua.create_function(create_world_pos_from_map_pos)?)?;
    t.set(
        "GetMapWorldSize",
        lua.create_function(|_, _map_id: i32| Ok((1000.0f64, 1000.0f64)))?,
    )?;
    t.set(
        "RequestPreloadMap",
        lua.create_function(|_, _map_id: i32| Ok(()))?,
    )?;

    Ok(t)
}

fn create_player_map_position(lua: &Lua, (_map_id, _unit): (i32, String)) -> Result<Value> {
    let pos = lua.create_table()?;
    pos.set("x", 0.5)?;
    pos.set("y", 0.5)?;
    Ok(Value::Table(pos))
}

fn create_world_pos_from_map_pos(lua: &Lua, (map_id, pos): (i32, Value)) -> Result<(i32, mlua::Table)> {
    let (x, y) = if let Value::Table(ref t) = pos {
        let x: f64 = t.get("x").unwrap_or(0.5);
        let y: f64 = t.get("y").unwrap_or(0.5);
        (x, y)
    } else {
        (0.5, 0.5)
    };
    let world_x = x * 1000.0;
    let world_y = y * 1000.0;
    let world_pos = lua.create_table()?;
    world_pos.set("x", world_x)?;
    world_pos.set("y", world_y)?;
    world_pos.set(
        "GetXY",
        lua.create_function(move |_, _: Value| Ok((world_x, world_y)))?,
    )?;
    Ok((map_id, world_pos))
}

/// Zone text functions (GetRealZoneText, GetZoneText, etc.).
fn register_zone_text_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetRealZoneText", lua.create_function(|_, ()| Ok("Stormwind City"))?)?;
    globals.set("GetZoneText", lua.create_function(|_, ()| Ok("Stormwind City"))?)?;
    globals.set("GetSubZoneText", lua.create_function(|_, ()| Ok("Trade District"))?)?;
    globals.set("GetMinimapZoneText", lua.create_function(|_, ()| Ok("Trade District"))?)?;
    Ok(())
}

/// UiMapPoint - map point creation helper.
fn register_ui_map_point(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "CreateFromVector2D",
        lua.create_function(|lua, (map_id, pos): (i32, Value)| {
            let (x, y) = if let Value::Table(ref t) = pos {
                let x: f64 = t.get("x").unwrap_or(0.5);
                let y: f64 = t.get("y").unwrap_or(0.5);
                (x, y)
            } else {
                (0.5, 0.5)
            };
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;
    t.set(
        "CreateFromCoordinates",
        lua.create_function(|lua, (map_id, x, y): (i32, f64, f64)| {
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;

    Ok(t)
}

/// C_MapExplorationInfo namespace - map exploration data.
fn register_c_map_exploration(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetExploredAreaIDsAtPosition",
        lua.create_function(|lua, (_map_id, _pos): (i32, Value)| lua.create_table())?,
    )?;
    t.set(
        "GetExploredMapTextures",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;

    Ok(t)
}

/// C_DateAndTime namespace - date/time utilities.
fn register_c_date_and_time(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "GetCurrentCalendarTime",
        lua.create_function(|lua, ()| {
            let info = lua.create_table()?;
            info.set("year", 2024)?;
            info.set("month", 1)?;
            info.set("monthDay", 1)?;
            info.set("weekday", 1)?;
            info.set("hour", 12)?;
            info.set("minute", 0)?;
            Ok(info)
        })?,
    )?;
    t.set("GetServerTimeLocal", lua.create_function(|_, ()| Ok(0i64))?)?;
    t.set("GetSecondsUntilDailyReset", lua.create_function(|_, ()| Ok(86400i32))?)?;
    t.set("GetSecondsUntilWeeklyReset", lua.create_function(|_, ()| Ok(604800i32))?)?;

    Ok(t)
}

/// C_Minimap namespace - minimap utilities.
fn register_c_minimap(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set(
        "IsInsideQuestBlob",
        lua.create_function(|_, (_quest_id, _x, _y): (i32, f64, f64)| Ok(false))?,
    )?;
    t.set("GetViewRadius", lua.create_function(|_, ()| Ok(200.0f64))?)?;
    t.set(
        "SetPlayerTexture",
        lua.create_function(|_, (_file_id, _icon_id): (i32, i32)| Ok(()))?,
    )?;
    // Tracking system stubs
    t.set("GetNumTrackingTypes", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetTrackingInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    t.set("GetTrackingFilter", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    t.set("ClearAllTracking", lua.create_function(|_, ()| Ok(()))?)?;
    t.set(
        "SetTrackingFilterByFilterIndex",
        lua.create_function(|_, (_index, _value): (i32, bool)| Ok(()))?,
    )?;
    t.set("ShouldUseHybridMinimap", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetUiMapID", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsFilteredOut", lua.create_function(|_, _filter: Value| Ok(false))?)?;

    Ok(t)
}

/// C_Navigation namespace - quest navigation waypoints.
fn register_c_navigation(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetFrame", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetDistance", lua.create_function(|_, ()| Ok(0.0f64))?)?;
    t.set("GetDestination", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsAutoFollowEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("SetAutoFollowEnabled", lua.create_function(|_, _enabled: bool| Ok(()))?)?;

    Ok(t)
}

/// C_TaxiMap namespace - flight path utilities.
fn register_c_taxi_map(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;

    t.set("GetAllTaxiNodes", lua.create_function(|lua, _map_id: i32| lua.create_table())?)?;
    t.set("GetTaxiNodesForMap", lua.create_function(|lua, _map_id: i32| lua.create_table())?)?;
    t.set("ShouldMapShowTaxiNodes", lua.create_function(|_, _map_id: i32| Ok(true))?)?;

    Ok(t)
}
