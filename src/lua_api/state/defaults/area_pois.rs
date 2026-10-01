use super::*;

pub(in crate::lua_api::state) fn default_area_pois() -> HashMap<i32, AreaPoiInfo> {
    [
        stormwind_portal_room_poi(),
        legion_invasion_poi(),
        warsong_gulch_poi(),
        cinderbrew_meadery_poi(),
        darkmoon_island_poi(),
    ]
    .into_iter()
    .map(|p| (p.area_poi_id, p))
    .collect()
}

fn stormwind_portal_room_poi() -> AreaPoiInfo {
    AreaPoiInfo {
        area_poi_id: 7000,
        name: "Stormwind Portal Room".into(),
        ui_map_id: Some(84),
        position: (0.52, 0.38),
        atlas_name: Some("Mage-Portal".into()),
        description: Some("Portals to every capital city.".into()),
        faction_id: None,
        icon_widget_set: None,
        linked_ui_map_id: None,
        is_current_event: false,
        should_glow: false,
        is_suppressible: false,
        is_locked: false,
        seconds_left: None,
    }
}

fn legion_invasion_poi() -> AreaPoiInfo {
    AreaPoiInfo {
        area_poi_id: 7001,
        name: "Legion Invasion".into(),
        ui_map_id: Some(13),
        position: (0.41, 0.62),
        atlas_name: Some("DemonInvasion3".into()),
        description: Some("A demonic incursion.".into()),
        faction_id: None,
        icon_widget_set: None,
        linked_ui_map_id: None,
        is_current_event: true,
        should_glow: true,
        is_suppressible: false,
        is_locked: false,
        seconds_left: Some(3600),
    }
}

fn warsong_gulch_poi() -> AreaPoiInfo {
    AreaPoiInfo {
        area_poi_id: 1001,
        is_suppressible: false,
        is_locked: false,
        name: "Warsong Gulch".into(),
        ui_map_id: Some(8685),
        position: (0.452, 0.641),
        atlas_name: Some("worldquest-icon-pvpbattle".into()),
        description: Some("Compete in the current PvP brawl.".into()),
        faction_id: None,
        icon_widget_set: None,
        linked_ui_map_id: None,
        is_current_event: true,
        should_glow: true,
        seconds_left: None,
    }
}

fn cinderbrew_meadery_poi() -> AreaPoiInfo {
    AreaPoiInfo {
        area_poi_id: 1002,
        is_suppressible: false,
        is_locked: false,
        name: "The Cinderbrew Meadery".into(),
        ui_map_id: Some(1980),
        position: (0.518, 0.274),
        atlas_name: Some("worldquest-icon-worldevent".into()),
        description: Some("A seasonal brewing challenge.".into()),
        faction_id: None,
        icon_widget_set: None,
        linked_ui_map_id: None,
        is_current_event: true,
        should_glow: true,
        seconds_left: None,
    }
}

fn darkmoon_island_poi() -> AreaPoiInfo {
    AreaPoiInfo {
        area_poi_id: 1004,
        is_suppressible: false,
        is_locked: false,
        name: "Darkmoon Island".into(),
        ui_map_id: Some(5861),
        position: (0.281, 0.734),
        atlas_name: Some("worldquest-icon-tournament".into()),
        description: Some("Take part in the traveling carnival.".into()),
        faction_id: None,
        icon_widget_set: None,
        linked_ui_map_id: None,
        is_current_event: true,
        should_glow: true,
        seconds_left: None,
    }
}

// Seed `SimState.lfg_category_info` with standard retail categories.
