//! Tests for c_map_api.rs: C_Map, zone text, UiMapPoint, C_DateAndTime, C_Minimap, etc.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_Map
// ============================================================================

#[test]
fn test_get_area_info() {
    let env = env();
    let name: String = env.eval("return C_Map.GetAreaInfo(1)").unwrap();
    assert_eq!(name, "Area_1");
}

#[test]
fn test_get_map_info() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Map.GetMapInfo(1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_map_info_has_name_field() {
    let env = env();
    let name: String = env.eval("return C_Map.GetMapInfo(1).name").unwrap();
    assert!(!name.is_empty());
}

#[test]
fn test_get_best_map_for_unit() {
    let env = env();
    let map_id: i32 = env.eval(r#"return C_Map.GetBestMapForUnit("player")"#).unwrap();
    assert_eq!(map_id, 1);
}

#[test]
fn test_get_player_map_position() {
    let env = env();
    let is_table: bool = env.eval(r#"
        local pos = C_Map.GetPlayerMapPosition(1, "player")
        return type(pos) == "table" or type(pos) == "userdata"
    "#).unwrap();
    assert!(is_table);
}

#[test]
fn test_get_map_children_info() {
    let env = env();
    let is_table: bool = env.eval("return type(C_Map.GetMapChildrenInfo(1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_map_world_size() {
    let env = env();
    let (w, h): (f64, f64) = env.eval("return C_Map.GetMapWorldSize(1)").unwrap();
    assert!(w > 0.0);
    assert!(h > 0.0);
}

// ============================================================================
// Zone text functions
// ============================================================================

#[test]
fn test_get_real_zone_text() {
    let env = env();
    let zone: String = env.eval("return GetRealZoneText()").unwrap();
    assert!(!zone.is_empty());
}

#[test]
fn test_get_zone_text() {
    let env = env();
    let zone: String = env.eval("return GetZoneText()").unwrap();
    assert!(!zone.is_empty());
}

#[test]
fn test_get_sub_zone_text() {
    let env = env();
    let zone: String = env.eval("return GetSubZoneText()").unwrap();
    assert!(!zone.is_empty());
}

#[test]
fn test_get_minimap_zone_text() {
    let env = env();
    let zone: String = env.eval("return GetMinimapZoneText()").unwrap();
    assert!(!zone.is_empty());
}

// ============================================================================
// UiMapPoint
// ============================================================================

#[test]
fn test_ui_map_point_create_from_vector() {
    let env = env();
    env.exec(r#"
        local pos = {x = 0.5, y = 0.5, GetXY = function(self) return self.x, self.y end}
        local point = UiMapPoint.CreateFromVector2D(1, pos)
        assert(point ~= nil, "Should create a map point")
        assert(point.uiMapID == 1, "Map ID should be 1")
    "#).unwrap();
}

#[test]
fn test_ui_map_point_create_from_coordinates() {
    let env = env();
    env.exec(r#"
        local point = UiMapPoint.CreateFromCoordinates(2, 0.3, 0.7)
        assert(point.uiMapID == 2)
    "#).unwrap();
}

// ============================================================================
// C_DateAndTime
// ============================================================================

#[test]
fn test_get_current_calendar_time() {
    let env = env();
    let is_table: bool = env.eval("return type(C_DateAndTime.GetCurrentCalendarTime()) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_server_time_local() {
    let env = env();
    let time: i32 = env.eval("return C_DateAndTime.GetServerTimeLocal()").unwrap();
    assert_eq!(time, 0);
}

#[test]
fn test_get_seconds_until_daily_reset() {
    let env = env();
    let secs: i32 = env.eval("return C_DateAndTime.GetSecondsUntilDailyReset()").unwrap();
    assert_eq!(secs, 86400);
}

#[test]
fn test_get_seconds_until_weekly_reset() {
    let env = env();
    let secs: i32 = env.eval("return C_DateAndTime.GetSecondsUntilWeeklyReset()").unwrap();
    assert_eq!(secs, 604800);
}

// ============================================================================
// C_Minimap
// ============================================================================

#[test]
fn test_minimap_is_inside_quest_blob() {
    let env = env();
    let val: bool = env.eval("return C_Minimap.IsInsideQuestBlob(1, 0.5, 0.5)").unwrap();
    assert!(!val);
}

#[test]
fn test_minimap_get_view_radius() {
    let env = env();
    let radius: f64 = env.eval("return C_Minimap.GetViewRadius()").unwrap();
    assert!(radius > 0.0);
}

#[test]
fn test_minimap_set_player_texture_no_error() {
    let env = env();
    env.exec("C_Minimap.SetPlayerTexture(0, 0)").unwrap();
}

// ============================================================================
// C_Navigation
// ============================================================================

#[test]
fn test_navigation_get_frame_nil() {
    let env = env();
    let is_nil: bool = env.eval("return C_Navigation.GetFrame() == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_navigation_get_distance() {
    let env = env();
    let dist: f64 = env.eval("return C_Navigation.GetDistance()").unwrap();
    assert_eq!(dist, 0.0);
}

#[test]
fn test_navigation_is_auto_follow_enabled() {
    let env = env();
    let val: bool = env.eval("return C_Navigation.IsAutoFollowEnabled()").unwrap();
    assert!(!val);
}

#[test]
fn test_navigation_set_auto_follow_enabled_no_error() {
    let env = env();
    env.exec("C_Navigation.SetAutoFollowEnabled(true)").unwrap();
}

// ============================================================================
// C_MapExplorationInfo
// ============================================================================

#[test]
fn test_get_explored_area_ids() {
    let env = env();
    let is_table: bool = env.eval("return type(C_MapExplorationInfo.GetExploredAreaIDsAtPosition(1, {})) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_explored_map_textures() {
    let env = env();
    let is_table: bool = env.eval("return type(C_MapExplorationInfo.GetExploredMapTextures(1)) == 'table'").unwrap();
    assert!(is_table);
}

// ============================================================================
// C_TaxiMap
// ============================================================================

#[test]
fn test_get_all_taxi_nodes() {
    let env = env();
    let is_table: bool = env.eval("return type(C_TaxiMap.GetAllTaxiNodes(1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_taxi_nodes_for_map() {
    let env = env();
    let is_table: bool = env.eval("return type(C_TaxiMap.GetTaxiNodesForMap(1)) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_should_map_show_taxi_nodes() {
    let env = env();
    let val: bool = env.eval("return C_TaxiMap.ShouldMapShowTaxiNodes(1)").unwrap();
    assert!(val);
}
