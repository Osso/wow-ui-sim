#![cfg(feature = "gui")]

//! Tests for MapCanvas pin infrastructure and world quest pin display.

use crate::common;
mod render_order_support;

use render_order_support::env_with_isolated_world_map;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_default_map_is_isle_of_dorn() {
    let env = env();
    let map_id: i32 = env.eval(r#"return C_Map.GetCurrentMapID()"#).unwrap();
    assert_eq!(map_id, 2248, "Default map should be Isle of Dorn");
}

#[test]
fn test_isle_of_dorn_has_seeded_world_quests() {
    let env = env();
    let count: i32 = env
        .eval(r#"return #C_TaskQuest.GetQuestsOnMap(2248)"#)
        .unwrap();
    assert_eq!(
        count, 2,
        "Isle of Dorn (2248) should have 2 seeded world quests"
    );
}

#[test]
fn test_default_map_has_world_quests() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local mapID = C_Map.GetCurrentMapID()
            return #C_TaskQuest.GetQuestsOnMap(mapID)
        "#,
        )
        .unwrap();
    assert!(
        count > 0,
        "Default map should have seeded world quests for pin display"
    );
}

#[test]
fn world_map_zone_map_shows_seeded_quest_pins() {
    let env = env_with_isolated_world_map();

    let (pin_count, debug): (i32, String) = env
        .eval(
            r#"
            if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                return -1, "WorldMapFrame not shown"
            end

            WorldMapFrame:SetMapID(2248)

            local count = 0
            for _pin in WorldMapFrame:EnumeratePinsByTemplate("QuestPinTemplate") do
                count = count + 1
            end

            local quests = C_QuestLog.GetQuestsOnMap(2248)
            local mapID = WorldMapFrame:GetMapID() or 0
            return count, string.format(
                "map=%s quests=%s",
                tostring(mapID),
                tostring(quests and #quests or -1)
            )
            "#,
        )
        .expect("world map quest pin query should run");

    assert!(
        pin_count > 0,
        "seeded zone map should show quest pins: {debug}"
    );
}
