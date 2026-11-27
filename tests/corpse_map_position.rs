#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! `C_DeathInfo.GetCorpseMapPosition` reads the corpse observations supplied by
//! `A_Admin.SetCorpseMapPosition`. Original consumer: the WorldMap
//! `DeathMapDataProvider` (Blizzard_SharedMapDataProviders/DeathMapDataProvider.lua:8-17,
//! installed by Blizzard_WorldMap.lua:191 on Retail and :201 on Forever) acquires a
//! `CorpsePinTemplate` pin at the returned position when the map changes.

use wow_ui_sim::lua_api::WowLuaEnv;

const OPEN_MAP: &str = r#"
local mapID = ...
OpenWorldMap(mapID)
local count, x, y = 0, -1, -1
for pin in WorldMapFrame:EnumeratePinsByTemplate("CorpsePinTemplate") do
    count = count + 1
    x, y = pin:GetPosition()
end
return WorldMapFrame:GetMapID(), count, x, y
"#;

fn open_map(env: &WowLuaEnv, map_id: i32) -> (i32, i32, f64, f64) {
    let chunk = format!("return (function(...) {OPEN_MAP} end)({map_id})");
    env.eval(&chunk).expect("the original world map should open")
}

fn assert_corpse_pins_follow_inputs(env: &WowLuaEnv) {
    env.exec("A_Admin.SetPlayerHealth(0, 100); A_Admin.SetCorpseMapPosition(84, 0.625, 0.375)")
        .expect("supply a corpse on map 84");
    assert_eq!(open_map(env, 85), (85, 0, -1.0, -1.0));
    assert_eq!(open_map(env, 84), (84, 1, 0.625, 0.375));
    env.exec("A_Admin.SetCorpseMapPosition(85, 0.25, 0.75)")
        .expect("supply a corpse on map 85");
    assert_eq!(open_map(env, 85), (85, 1, 0.25, 0.75));
    env.exec("A_Admin.ClearCorpse()").expect("remove the corpse");
    assert_eq!(open_map(env, 84), (84, 0, -1.0, -1.0));
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn world_map_corpse_pin_follows_the_supplied_corpse(env: &WowLuaEnv) {
    assert_corpse_pins_follow_inputs(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_world_map_corpse_pin_follows_the_supplied_corpse() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_corpse_pins_follow_inputs(&env);
    });
}
