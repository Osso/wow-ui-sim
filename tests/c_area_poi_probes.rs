//! Tests for `C_AreaPoiInfo` probes backed by `SimState.area_pois`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AreaPoiInfo;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_area_poi_info_returns_seeded_row_by_nil_ui_map() {
    let env = env();
    let (id, name, x, y, is_event, should_glow): (i32, String, f64, f64, bool, bool) = env
        .eval(
            r#"
            local p = C_AreaPoiInfo.GetAreaPOIInfo(nil, 7001)
            return p.areaPoiID, p.name, p.position.x, p.position.y,
                   p.isCurrentEvent, p.shouldGlow
            "#,
        )
        .unwrap();
    assert_eq!(id, 7001);
    assert_eq!(name, "Legion Invasion");
    assert_eq!(x, 0.41);
    assert_eq!(y, 0.62);
    assert!(is_event);
    assert!(should_glow);
}

#[test]
fn get_area_poi_info_returns_seeded_optional_fields() {
    let env = env();
    let (atlas, description, stormwind_atlas): (String, String, String) = env
        .eval(
            r#"
            local legion = C_AreaPoiInfo.GetAreaPOIInfo(nil, 7001)
            local sw = C_AreaPoiInfo.GetAreaPOIInfo(nil, 7000)
            return legion.atlasName, legion.description, sw.atlasName
            "#,
        )
        .unwrap();
    assert_eq!(atlas, "DemonInvasion3");
    assert_eq!(description, "A demonic incursion.");
    assert_eq!(stormwind_atlas, "Mage-Portal");
}

#[test]
fn get_area_poi_info_ui_map_id_filters_when_non_nil() {
    let env = env();
    // Stormwind POI is bound to uiMapID 84. Asking for it with map 84
    // succeeds; asking with map 13 (Eastern Kingdoms) returns nothing
    // even though the poi id exists.
    let (match_name, mismatch_count): (String, i32) = env
        .eval(
            r#"
            local ok = C_AreaPoiInfo.GetAreaPOIInfo(84, 7000)
            local mismatch_n = select('#', C_AreaPoiInfo.GetAreaPOIInfo(13, 7000))
            return ok.name, mismatch_n
            "#,
        )
        .unwrap();
    assert_eq!(match_name, "Stormwind Portal Room");
    assert_eq!(
        mismatch_count, 0,
        "mismatched uiMapID should return nothing"
    );
}

#[test]
fn get_area_poi_info_returns_nothing_for_unknown_poi() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_AreaPoiInfo.GetAreaPOIInfo(nil, 999999))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_area_poi_seconds_left_returns_number_for_time_limited_poi() {
    let env = env();
    let seconds: i32 = env
        .eval("return C_AreaPoiInfo.GetAreaPOISecondsLeft(7001)")
        .unwrap();
    assert_eq!(seconds, 3600, "Legion Invasion has 1h remaining");
}

#[test]
fn get_area_poi_seconds_left_returns_nothing_for_permanent_poi() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_AreaPoiInfo.GetAreaPOISecondsLeft(7000))")
        .unwrap();
    assert_eq!(
        nret, 0,
        "permanent POIs have no countdown and return nothing"
    );
}

#[test]
fn get_area_poi_seconds_left_returns_nothing_for_unknown_poi() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_AreaPoiInfo.GetAreaPOISecondsLeft(999999))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn area_poi_table_reflects_sim_state_mutation() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.area_pois.insert(
            42,
            AreaPoiInfo {
                area_poi_id: 42,
                name: "Admin POI".into(),
                ui_map_id: None,
                position: (0.1, 0.2),
                atlas_name: None,
                description: None,
                faction_id: Some(5),
                icon_widget_set: None,
                linked_ui_map_id: None,
                is_current_event: false,
                should_glow: true,
                is_suppressible: false,
                is_locked: false,
                seconds_left: Some(90),
            },
        );
    }
    let (name, faction, seconds, atlas_nil): (String, i32, i32, bool) = env
        .eval(
            r#"
            local p = C_AreaPoiInfo.GetAreaPOIInfo(nil, 42)
            return p.name, p.factionID, C_AreaPoiInfo.GetAreaPOISecondsLeft(42),
                   p.atlasName == nil
            "#,
        )
        .unwrap();
    assert_eq!(name, "Admin POI");
    assert_eq!(faction, 5);
    assert_eq!(seconds, 90);
    assert!(atlas_nil, "unset atlas_name should be nil in the table");
}
