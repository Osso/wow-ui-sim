//! C_Map namespace and related map/location API functions.
//!
//! Contains map, exploration, navigation, and location-related API functions.

use mlua::{Lua, Result, Value};

/// Register C_Map namespace and map-related functions.
pub fn register_c_map_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // C_Map namespace - map and area information
    let c_map = lua.create_table()?;
    c_map.set(
        "GetAreaInfo",
        lua.create_function(|lua, area_id: i32| {
            // Return area name for the given area ID
            // In simulation, return a placeholder
            Ok(Value::String(lua.create_string(&format!("Area_{}", area_id))?))
        })?,
    )?;
    c_map.set(
        "GetMapInfo",
        lua.create_function(|lua, map_id: i32| {
            // Return map info table
            let info = lua.create_table()?;
            info.set("mapID", map_id)?;
            info.set("name", format!("Map_{}", map_id))?;
            info.set("mapType", 3)?; // Zone type
            info.set("parentMapID", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_map.set(
        "GetBestMapForUnit",
        lua.create_function(|_, _unit: String| {
            // Return a default map ID
            Ok(Value::Integer(1)) // Durotar
        })?,
    )?;
    c_map.set(
        "GetPlayerMapPosition",
        lua.create_function(|lua, (_map_id, _unit): (i32, String)| {
            // Return a position vector (x, y)
            let pos = lua.create_table()?;
            pos.set("x", 0.5)?;
            pos.set("y", 0.5)?;
            Ok(Value::Table(pos))
        })?,
    )?;
    c_map.set(
        "GetMapChildrenInfo",
        lua.create_function(|lua, (_map_id, _map_type, _all_descendants): (i32, Option<i32>, Option<bool>)| {
            // Return empty table of child maps
            lua.create_table()
        })?,
    )?;
    c_map.set(
        "GetWorldPosFromMapPos",
        lua.create_function(|lua, (map_id, pos): (i32, Value)| {
            // pos is a Vector2DMixin with x, y fields
            // Returns (instanceID, Vector2DMixin with world coordinates)
            let (x, y) = if let Value::Table(ref t) = pos {
                let x: f64 = t.get("x").unwrap_or(0.5);
                let y: f64 = t.get("y").unwrap_or(0.5);
                (x, y)
            } else {
                (0.5, 0.5)
            };
            // Convert map coords (0-1) to world coords (arbitrary scale)
            // Use a simple scale of 1000 units per map
            let world_x = x * 1000.0;
            let world_y = y * 1000.0;
            let world_pos = lua.create_table()?;
            world_pos.set("x", world_x)?;
            world_pos.set("y", world_y)?;
            // Add GetXY method
            world_pos.set(
                "GetXY",
                lua.create_function(move |_, _: Value| Ok((world_x, world_y)))?,
            )?;
            // Instance ID is typically same as map_id for simplicity
            Ok((map_id, world_pos))
        })?,
    )?;
    c_map.set(
        "GetMapWorldSize",
        lua.create_function(|_, _map_id: i32| {
            // Return width, height in world units (arbitrary scale)
            Ok((1000.0f64, 1000.0f64))
        })?,
    )?;
    globals.set("C_Map", c_map)?;

    // Zone text functions
    globals.set(
        "GetRealZoneText",
        lua.create_function(|_, ()| Ok("Stormwind City"))?,
    )?;
    globals.set(
        "GetZoneText",
        lua.create_function(|_, ()| Ok("Stormwind City"))?,
    )?;
    globals.set(
        "GetSubZoneText",
        lua.create_function(|_, ()| Ok("Trade District"))?,
    )?;
    globals.set(
        "GetMinimapZoneText",
        lua.create_function(|_, ()| Ok("Trade District"))?,
    )?;

    // UiMapPoint - map point creation helper
    let ui_map_point = lua.create_table()?;
    ui_map_point.set(
        "CreateFromVector2D",
        lua.create_function(|lua, (map_id, pos): (i32, Value)| {
            // Extract x, y from position table
            let (x, y) = if let Value::Table(ref t) = pos {
                let x: f64 = t.get("x").unwrap_or(0.5);
                let y: f64 = t.get("y").unwrap_or(0.5);
                (x, y)
            } else {
                (0.5, 0.5)
            };
            // Create a map point table
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;
    ui_map_point.set(
        "CreateFromCoordinates",
        lua.create_function(|lua, (map_id, x, y): (i32, f64, f64)| {
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;
    globals.set("UiMapPoint", ui_map_point)?;

    // C_MapExplorationInfo namespace - map exploration data
    let c_map_exploration = lua.create_table()?;
    c_map_exploration.set(
        "GetExploredAreaIDsAtPosition",
        lua.create_function(|lua, (_map_id, _pos): (i32, Value)| {
            // Return empty table (no explored areas in sim)
            lua.create_table()
        })?,
    )?;
    c_map_exploration.set(
        "GetExploredMapTextures",
        lua.create_function(|lua, _map_id: i32| {
            // Return empty table (no textures in sim)
            lua.create_table()
        })?,
    )?;
    globals.set("C_MapExplorationInfo", c_map_exploration)?;

    // C_DateAndTime namespace - date/time utilities
    let c_date_time = lua.create_table()?;
    c_date_time.set(
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
    c_date_time.set(
        "GetServerTimeLocal",
        lua.create_function(|_, ()| Ok(0i64))?,
    )?;
    c_date_time.set(
        "GetSecondsUntilDailyReset",
        lua.create_function(|_, ()| Ok(86400i32))?,
    )?;
    c_date_time.set(
        "GetSecondsUntilWeeklyReset",
        lua.create_function(|_, ()| Ok(604800i32))?,
    )?;
    globals.set("C_DateAndTime", c_date_time)?;

    // C_Minimap namespace - minimap utilities
    let c_minimap = lua.create_table()?;
    c_minimap.set(
        "IsInsideQuestBlob",
        lua.create_function(|_, (_quest_id, _x, _y): (i32, f64, f64)| Ok(false))?,
    )?;
    c_minimap.set(
        "GetViewRadius",
        lua.create_function(|_, ()| Ok(200.0f64))?,
    )?;
    c_minimap.set(
        "SetPlayerTexture",
        lua.create_function(|_, (_file_id, _icon_id): (i32, i32)| Ok(()))?,
    )?;
    globals.set("C_Minimap", c_minimap)?;

    // C_Navigation namespace - quest navigation waypoints
    let c_navigation = lua.create_table()?;
    c_navigation.set(
        "GetFrame",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_navigation.set(
        "GetDistance",
        lua.create_function(|_, ()| Ok(0.0f64))?,
    )?;
    c_navigation.set(
        "GetDestination",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_navigation.set(
        "IsAutoFollowEnabled",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_navigation.set(
        "SetAutoFollowEnabled",
        lua.create_function(|_, _enabled: bool| Ok(()))?,
    )?;
    globals.set("C_Navigation", c_navigation)?;

    // C_TaxiMap namespace - flight path utilities
    let c_taxi_map = lua.create_table()?;
    c_taxi_map.set(
        "GetAllTaxiNodes",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_taxi_map.set(
        "GetTaxiNodesForMap",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_taxi_map.set(
        "ShouldMapShowTaxiNodes",
        lua.create_function(|_, _map_id: i32| Ok(true))?,
    )?;
    globals.set("C_TaxiMap", c_taxi_map)?;

    Ok(())
}
