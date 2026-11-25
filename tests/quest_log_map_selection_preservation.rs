//! Exercise the visible original world map and quest POI selection together.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![crate::common::blizzard_ui_dir()];
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env
}

#[test]
fn original_quest_map_follows_two_selected_maps() {
    game_env()
        .exec(
            r#"
            assert(C_AddOns.LoadAddOn('Blizzard_WorldMap'))
            OpenQuestLog()
            assert(WorldMapFrame:IsVisible() and QuestMapFrame:IsVisible(),
                'original world map and quest log should be visible')
            for _, mapID in ipairs({84, 23}) do
                C_Map.SetMapForQuestLog(mapID)
                assert(WorldMapFrame:GetMapID() == mapID,
                    'the original visible map should show the selected zone')
                assert(QuestMapFrame:SyncQuestSystemWithCurrentMap(),
                    'the original quest log should synchronize quest pins')
                assert(C_QuestLog.GetMapForQuestPOIs() == mapID,
                    'quest pins should use the selected zone')
            end
            "#,
        )
        .expect("original world map and quest pin selection");
}
