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

const OPEN_MAPS: &str = r#"
local map = WorldMapFrame
if map:IsShown() then ToggleWorldMap() end
local events = {}
local handle = EventRegistry:RegisterFrameEventAndCallbackWithHandle("WORLD_MAP_OPEN", function(_, mapID)
    events[#events + 1] = mapID or false
end, {})
for index, mapID in ipairs({947, 2248}) do
    assert(select('#', C_Map.OpenWorldMap(mapID)) == 0)
    assert(map:IsShown() and map:GetMapID() == mapID, 'World map must select ' .. mapID)
    assert(events[index] == mapID, 'WORLD_MAP_OPEN must carry the selected map')
end
assert(select('#', C_Map.OpenWorldMap()) == 0)
assert(map:IsShown() and map:GetMapID() == 2248)
assert(events[3] == false and #events == 3, 'omitted map ID must publish nil')
handle:Unregister()
ToggleWorldMap()
"#;

const OPEN_MAP_POLICY: &str = r#"
local map = WorldMapFrame
local denied = {}
local handle = EventRegistry:RegisterFrameEventAndCallbackWithHandle("ADDON_ACTION_BLOCKED", function(_, addon, operation)
    denied[#denied + 1] = {addon, operation}
end, {})
A_Admin.SetInCombat(false)
WorldMapOpenRestrictionProbe(947)
assert(map:IsShown() and map:GetMapID() == 947)
A_Admin.SetInCombat(true)
WorldMapOpenRestrictionProbe(2248)
assert(map:GetMapID() == 947 and #denied == 1)
assert(denied[1][1] == 'WorldMapOpenRestrictionProbe')
assert(denied[1][2] == 'C_Map.OpenWorldMap()')
A_Admin.SetInCombat(false)
WorldMapOpenRestrictionProbe(2248)
assert(map:GetMapID() == 2248 and #denied == 1)
handle:Unregister()
ToggleWorldMap()
"#;

fn load_open_map_probe(env: &WowLuaEnv) {
    let temp = tempfile::tempdir().unwrap();
    let addon = temp.path().join("WorldMapOpenRestrictionProbe");
    std::fs::create_dir(&addon).unwrap();
    let toc = addon.join("WorldMapOpenRestrictionProbe.toc");
    std::fs::write(&toc, "## Title: WorldMapOpenRestrictionProbe\nprobe.lua\n").unwrap();
    std::fs::write(addon.join("probe.lua"), "function WorldMapOpenRestrictionProbe(id) return C_Map.OpenWorldMap(id) end").unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("load map request probe");
}

fn assert_original_map_requests(env: &WowLuaEnv) {
    env.exec(OPEN_MAPS).expect("original WorldMap must handle the open event");
    load_open_map_probe(env);
    env.exec(OPEN_MAP_POLICY).expect("protected map requests preserve combat policy");
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn open_world_map_selects_requested_pages(env: &WowLuaEnv) {
    assert_original_map_requests(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_open_world_map_selects_requested_pages() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_original_map_requests(&env);
    });
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
