//! Tests for `C_Map` probes backed by `SimState.maps` +
//! `SimState.player_map_position`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{MapChildRect, MapRect};

fn seed_eastern_kingdoms_child_rects(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let parent = state
        .maps
        .get_mut(&13)
        .expect("Eastern Kingdoms must be in default_maps");
    parent.child_rects = vec![MapChildRect {
        map_id: 84,
        left: 0.40,
        right: 0.55,
        top: 0.60,
        bottom: 0.75,
    }];
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_map_art_id_returns_seeded_art_id() {
    let env = env();
    let (stormwind, eastern_kingdoms, azeroth): (i32, i32, i32) = env
        .eval(
            r#"
            return C_Map.GetMapArtID(84),
                   C_Map.GetMapArtID(13),
                   C_Map.GetMapArtID(947)
            "#,
        )
        .unwrap();
    assert_eq!(stormwind, 104, "Stormwind City art id");
    assert_eq!(eastern_kingdoms, 62, "Eastern Kingdoms art id");
    assert_eq!(azeroth, 0, "Azeroth world map has no tileset");
}

#[test]
fn get_map_art_id_returns_nothing_for_unknown_map() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapArtID(999999))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_map_children_info_returns_direct_children() {
    let env = env();
    let (count, first_id, first_name, first_type, first_parent): (i32, i32, String, i32, i32) = env
        .eval(
            r#"
            local children = C_Map.GetMapChildrenInfo(13)
            local first = children[1]
            return #children, first.mapID, first.name, first.mapType, first.parentMapID
            "#,
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "Eastern Kingdoms has one seeded child (Stormwind)"
    );
    assert_eq!(first_id, 84);
    assert_eq!(first_name, "Stormwind City");
    assert_eq!(first_type, 3, "Zone");
    assert_eq!(first_parent, 13);
}

#[test]
fn get_map_children_info_with_all_descendants_walks_tree() {
    let env = env();
    let (count, ids): (i32, Vec<i32>) = env
        .eval(
            r#"
            local function array(tbl, key)
                local out = {}
                for i = 1, #tbl do out[i] = tbl[i][key] end
                return out
            end
            local children = C_Map.GetMapChildrenInfo(947, nil, true)
            return #children, array(children, "mapID")
            "#,
        )
        .unwrap();
    assert_eq!(count, 2, "Azeroth → Eastern Kingdoms → Stormwind");
    let mut sorted = ids;
    sorted.sort();
    assert_eq!(sorted, vec![13, 84]);
}

#[test]
fn get_map_info_exposes_cosmic_parent_for_seeded_world_maps() {
    let env = env();
    let (cosmic_type, azeroth_parent, dorn_parent): (i32, i32, i32) = env
        .eval(
            r#"
            return C_Map.GetMapInfo(946).mapType,
                   C_Map.GetMapInfo(947).parentMapID,
                   C_Map.GetMapInfo(2248).parentMapID
            "#,
        )
        .unwrap();
    assert_eq!(cosmic_type, 0, "946 is the Cosmic map root (UiMap.db2)");
    assert_eq!(azeroth_parent, 946, "947 Azeroth hangs off Cosmic");
    assert_eq!(dorn_parent, 947, "Isle of Dorn's nearest seeded ancestor");
}

#[test]
fn get_map_children_info_filters_by_map_type() {
    let env = env();
    // Azeroth (947) has a Continent (13) and Stormwind (84, Zone).
    // Ask for Zones only with allDescendants=true — we should get 84.
    let (count, first_id, first_type): (i32, i32, i32) = env
        .eval(
            r#"
            local zones = C_Map.GetMapChildrenInfo(947, 3, true)
            return #zones, zones[1].mapID, zones[1].mapType
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(first_id, 84);
    assert_eq!(first_type, 3);
}

#[test]
fn get_map_children_info_returns_empty_array_for_childless_map() {
    let env = env();
    let count: i32 = env.eval("return #C_Map.GetMapChildrenInfo(84)").unwrap();
    assert_eq!(count, 0, "Stormwind City has no seeded children");
}

#[test]
fn get_map_children_info_returns_nothing_for_unknown_map() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapChildrenInfo(999999))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_player_map_position_returns_default_center() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local pos = C_Map.GetPlayerMapPosition(84, "player")
            return pos.x, pos.y
            "#,
        )
        .unwrap();
    assert_eq!(x, 0.5);
    assert_eq!(y, 0.5);
}

#[test]
fn get_player_map_position_supports_get_xy() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local pos = C_Map.GetPlayerMapPosition(84, "player")
            return pos:GetXY()
            "#,
        )
        .unwrap();
    assert_eq!(x, 0.5);
    assert_eq!(y, 0.5);
}

#[test]
fn get_player_map_position_reflects_sim_state_mutation() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.player_map_position = (0.25, 0.75);
    }

    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local pos = C_Map.GetPlayerMapPosition(84, "player")
            return pos.x, pos.y
            "#,
        )
        .unwrap();
    assert_eq!(x, 0.25);
    assert_eq!(y, 0.75);
}

#[test]
fn get_player_map_position_returns_nil_for_unknown_map() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return C_Map.GetPlayerMapPosition(999999, "player") == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_player_map_position_returns_nil_for_non_player_unit() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return C_Map.GetPlayerMapPosition(84, "target") == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_map_info_at_position_resolves_child_inside_rect() {
    let env = env();
    seed_eastern_kingdoms_child_rects(&env);

    let (map_id, name): (i32, String) = env
        .eval(
            r#"
            local info = C_Map.GetMapInfoAtPosition(13, 0.475, 0.675)
            return info.mapID, info.name
            "#,
        )
        .unwrap();
    assert_eq!(map_id, 84);
    assert_eq!(name, "Stormwind City");
}

#[test]
fn get_map_info_at_position_returns_nothing_outside_any_rect() {
    let env = env();
    seed_eastern_kingdoms_child_rects(&env);

    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapInfoAtPosition(13, 0.10, 0.10))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_map_info_at_position_returns_nothing_for_unknown_map() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapInfoAtPosition(999999, 0.5, 0.5))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_map_info_at_position_returns_nothing_when_parent_has_no_rects() {
    let env = env();

    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapInfoAtPosition(13, 0.5, 0.5))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_map_info_at_position_is_inclusive_at_rect_edges() {
    let env = env();
    seed_eastern_kingdoms_child_rects(&env);

    let (left_edge, right_edge, top_edge, bottom_edge): (i32, i32, i32, i32) = env
        .eval(
            r#"
            return C_Map.GetMapInfoAtPosition(13, 0.40, 0.65).mapID,
                   C_Map.GetMapInfoAtPosition(13, 0.55, 0.65).mapID,
                   C_Map.GetMapInfoAtPosition(13, 0.45, 0.60).mapID,
                   C_Map.GetMapInfoAtPosition(13, 0.45, 0.75).mapID
            "#,
        )
        .unwrap();
    assert_eq!(left_edge, 84);
    assert_eq!(right_edge, 84);
    assert_eq!(top_edge, 84);
    assert_eq!(bottom_edge, 84);
}

#[test]
fn get_map_info_at_position_picks_first_rect_when_overlapping() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        let azeroth = state.maps.get_mut(&947).unwrap();
        azeroth.child_rects = vec![
            MapChildRect {
                map_id: 13,
                left: 0.0,
                right: 1.0,
                top: 0.0,
                bottom: 1.0,
            },
            MapChildRect {
                map_id: 84,
                left: 0.4,
                right: 0.6,
                top: 0.4,
                bottom: 0.6,
            },
        ];
    }

    let map_id: i32 = env
        .eval("return C_Map.GetMapInfoAtPosition(947, 0.5, 0.5).mapID")
        .unwrap();
    assert_eq!(
        map_id, 13,
        "First-match wins when child rects overlap (data order is authoritative)"
    );
}

#[test]
fn get_map_info_at_position_skips_rect_pointing_to_unknown_child() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        let parent = state.maps.get_mut(&13).unwrap();
        parent.child_rects = vec![MapChildRect {
            map_id: 4242,
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        }];
    }

    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapInfoAtPosition(13, 0.5, 0.5))")
        .unwrap();
    assert_eq!(
        nret, 0,
        "A rect whose map_id is missing from state.maps must yield no return values"
    );
}

fn seed_stormwind_in_eastern_kingdoms_in_azeroth(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.maps.get_mut(&84).unwrap().rect_on_parent = Some(MapRect {
        left: 0.40,
        right: 0.55,
        top: 0.60,
        bottom: 0.75,
    });
    state.maps.get_mut(&13).unwrap().rect_on_parent = Some(MapRect {
        left: 0.20,
        right: 0.40,
        top: 0.10,
        bottom: 0.50,
    });
}

#[test]
fn get_map_rect_on_map_returns_identity_for_same_map() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) =
        env.eval("return C_Map.GetMapRectOnMap(84, 84)").unwrap();
    assert!((l - 0.0).abs() < 1e-9);
    assert!((r - 1.0).abs() < 1e-9);
    assert!((t - 0.0).abs() < 1e-9);
    assert!((b - 1.0).abs() < 1e-9);
}

#[test]
fn get_map_rect_on_map_returns_rect_on_direct_parent() {
    let env = env();
    seed_stormwind_in_eastern_kingdoms_in_azeroth(&env);

    let (l, r, t, b): (f64, f64, f64, f64) =
        env.eval("return C_Map.GetMapRectOnMap(84, 13)").unwrap();
    assert!((l - 0.40).abs() < 1e-9);
    assert!((r - 0.55).abs() < 1e-9);
    assert!((t - 0.60).abs() < 1e-9);
    assert!((b - 0.75).abs() < 1e-9);
}

#[test]
fn get_map_rect_on_map_composes_rect_up_through_grandparent() {
    let env = env();
    seed_stormwind_in_eastern_kingdoms_in_azeroth(&env);

    let (l, r, t, b): (f64, f64, f64, f64) =
        env.eval("return C_Map.GetMapRectOnMap(84, 947)").unwrap();
    let span_x = 0.40 - 0.20;
    let span_y = 0.50 - 0.10;
    let expected_l = 0.20 + 0.40 * span_x;
    let expected_r = 0.20 + 0.55 * span_x;
    let expected_t = 0.10 + 0.60 * span_y;
    let expected_b = 0.10 + 0.75 * span_y;
    assert!(
        (l - expected_l).abs() < 1e-9,
        "l: got {l}, want {expected_l}"
    );
    assert!(
        (r - expected_r).abs() < 1e-9,
        "r: got {r}, want {expected_r}"
    );
    assert!(
        (t - expected_t).abs() < 1e-9,
        "t: got {t}, want {expected_t}"
    );
    assert!(
        (b - expected_b).abs() < 1e-9,
        "b: got {b}, want {expected_b}"
    );
}

#[test]
fn get_map_rect_on_map_returns_nothing_when_top_is_not_an_ancestor() {
    let env = env();
    seed_stormwind_in_eastern_kingdoms_in_azeroth(&env);

    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapRectOnMap(84, 1))")
        .unwrap();
    assert_eq!(nret, 0, "Dun Morogh is not an ancestor of Stormwind");
}

#[test]
fn get_map_rect_on_map_returns_nothing_when_chain_link_missing_rect() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.maps.get_mut(&84).unwrap().rect_on_parent = Some(MapRect {
            left: 0.4,
            right: 0.5,
            top: 0.6,
            bottom: 0.7,
        });
        state.maps.get_mut(&13).unwrap().rect_on_parent = None;
    }
    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapRectOnMap(84, 947))")
        .unwrap();
    assert_eq!(nret, 0, "Eastern Kingdoms missing rect must abort the walk");
}

#[test]
fn get_map_rect_on_map_returns_nothing_for_unknown_ui_map_id() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_Map.GetMapRectOnMap(999999, 947))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_map_rect_on_map_drives_get_map_center_on_map_helper() {
    let env = env();
    seed_stormwind_in_eastern_kingdoms_in_azeroth(&env);

    let (cx, cy): (f64, f64) = env
        .eval("return MapUtil.GetMapCenterOnMap(84, 13)")
        .unwrap();
    assert!((cx - (0.40 + (0.55 - 0.40) * 0.5)).abs() < 1e-9);
    assert!((cy - (0.60 + (0.75 - 0.60) * 0.5)).abs() < 1e-9);
}

#[test]
fn set_map_for_quest_log_updates_current_map_id() {
    let env = env();
    let (before, after): (i32, i32) = env
        .eval(
            r#"
            local before = C_Map.GetCurrentMapID()
            C_Map.SetMapForQuestLog(1)
            return before, C_Map.GetCurrentMapID()
            "#,
        )
        .unwrap();
    assert_eq!(before, 2248);
    assert_eq!(after, 1);
}
