#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! `C_Map.CloseWorldMapInteraction` ends the supplied `WorldMap` player
//! interaction. Original consumer: `WorldMapMixin:OnHide` (Blizzard_WorldMap.lua)
//! calls it whenever the world map closes, here through `ToggleWorldMap`.

use wow_ui_sim::lua_api::WowLuaEnv;

const MAP_SESSION: &str = r#"
local closeMap = ...
A_Admin.OpenWorldMapInteraction()
OpenWorldMap(84)
if closeMap then ToggleWorldMap() end
local worldMap = Enum.PlayerInteractionType.WorldMap
return WorldMapFrame:IsShown(), C_PlayerInteractionManager.IsInteractingWithNpcOfType(worldMap)
"#;

fn map_session(env: &WowLuaEnv, close_map: bool) -> (bool, bool) {
    let chunk = format!("return (function(...) {MAP_SESSION} end)({close_map})");
    env.eval(&chunk).expect("the original world map should open")
}

fn assert_closing_the_map_ends_the_interaction(env: &WowLuaEnv) {
    assert_eq!(map_session(env, false), (true, true));
    assert_eq!(map_session(env, true), (false, false));
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn closing_the_world_map_ends_the_map_interaction(env: &WowLuaEnv) {
    assert_closing_the_map_ends_the_interaction(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_closing_the_world_map_ends_the_map_interaction() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_closing_the_map_ends_the_interaction(&env);
    });
}
