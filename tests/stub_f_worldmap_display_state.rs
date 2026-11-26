//! Retail preservation of the original WorldMap display-state owner through real controls.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use std::path::Path;

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    StartupAddon, discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen,
    load_startup_addon, register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::collect_lua_error_startup;

const DISPLAY_STATE_CONTROLS_LUA_PART_1: &str = r#"
local map = assert(WorldMapFrame)
local owner = QuestLogOwnerMixin.SetDisplayState
assert(map.HandleUserActionMaximizeSelf == QuestLogOwnerMixin.HandleUserActionMaximizeSelf)
assert(map.HandleUserActionMinimizeSelf == QuestLogOwnerMixin.HandleUserActionMinimizeSelf)
assert(map.HandleUserActionToggleSidePanel == QuestLogOwnerMixin.HandleUserActionToggleSidePanel)
local controls = map.BorderFrame.MaximizeMinimizeFrame
local panel = map.SidePanelToggle
assert(type(controls.MaximizeButton:GetScript("OnClick")) == "function")
assert(type(controls.MinimizeButton:GetScript("OnClick")) == "function")
assert(type(panel.OpenButton:GetScript("OnClick")) == "function")
assert(type(panel.CloseButton:GetScript("OnClick")) == "function")
assert(not map:IsShown())
local function geometry()
  assert(map.SetDisplayState == owner)
  local scroll, child = map:GetCanvasContainer(), map:GetCanvas()
  assert(scroll == map.ScrollContainer and child == scroll.Child)
  local w, h = map:GetWidth(), map:GetHeight()
  local sw, sh = scroll:GetWidth(), scroll:GetHeight()
  local cw, ch = child:GetWidth(), child:GetHeight()
  assert(w > 0 and h > 0 and sw > 0 and sh > 0 and cw > 0 and ch > 0)
  assert(scroll.currentScale > 0)
  assert(scroll.targetScale > 0)
  return w, h, sw, sh, cw, ch
end
SetCVar("miniWorldMap", 1)
"#;

const DISPLAY_STATE_CONTROLS_LUA_PART_2: &str = r#"SetCVar("questLogOpen", 0)
for _ = 1, 2 do
  ToggleWorldMap()
  assert(map:IsShown() and map:IsMinimized() and not map.QuestLog:IsShown())
  local mapID = map:GetMapID()
  assert(mapID > 0)
  local w, h, sw, sh, cw, ch = geometry()
  controls.MaximizeButton:Click("LeftButton")
  assert(map:IsShown() and map:IsMaximized() and not panel:IsShown())
  assert(not map.QuestLog:IsShown() and GetCVar("miniWorldMap") == "0")
  local largeW, largeH, largeSW, largeSH, largeCW, largeCH = geometry()
  assert(largeW > w and largeH > h and largeSW > sw and largeSH > sh)
  assert(largeCW == cw and largeCH == ch and map:GetMapID() == mapID)
  controls.MinimizeButton:Click("LeftButton")
  assert(map:IsShown() and map:IsMinimized() and panel:IsShown())
  assert(GetCVar("miniWorldMap") == "1")
  local smallW, smallH, smallSW, smallSH = geometry()
  assert(smallW == w and smallH == h and smallSW == sw and smallSH == sh)
  panel.OpenButton:Click("LeftButton")
  assert(map.QuestLog:IsShown() and GetCVar("questLogOpen") == "1")
  local logW, logH, logSW, logSH = geometry()
  assert(logW > w and logH == h and logSW == sw and logSH == sh)
  panel.CloseButton:Click("LeftButton")
  assert(not map.QuestLog:IsShown() and GetCVar("questLogOpen") == "0")
  local restoredW, restoredH = geometry()
"#;

const DISPLAY_STATE_CONTROLS_LUA_PART_3: &str = r#"  assert(restoredW == w and restoredH == h and map:GetMapID() == mapID)
  ToggleWorldMap()
  assert(not map:IsShown())
end
"#;

fn display_state_controls_lua() -> String {
    [DISPLAY_STATE_CONTROLS_LUA_PART_1, DISPLAY_STATE_CONTROLS_LUA_PART_2, DISPLAY_STATE_CONTROLS_LUA_PART_3].concat()
}

const MINIMAP_ZONE_BUTTON_LUA: &str = r#"
local map = assert(WorldMapFrame)
local button = assert(MinimapCluster.ZoneTextButton)
assert(button.OnClick == MinimapZoneTextButtonMixin.OnClick)
assert(type(button:GetScript("OnClick")) == "function")
assert(button:IsVisible() and button:IsEnabled())
assert(GetBindingKey("TOGGLEWORLDMAP") == "M")
assert(not map:IsShown())
SetCVar("miniWorldMap", 1)
SetCVar("questLogOpen", 0)
local savedMapID, savedW, savedH
for i = 1, 2 do
  button:Click("LeftButton")
  assert(map:IsShown() and map:IsMinimized() and not map.QuestLog:IsShown())
  local scroll, child = map:GetCanvasContainer(), map:GetCanvas()
  assert(child == scroll.Child)
  local mapID, w, h = map:GetMapID(), scroll:GetWidth(), scroll:GetHeight()
  assert(mapID > 0 and w > 0 and h > 0)
  assert(child:GetWidth() > 0 and child:GetHeight() > 0 and scroll.currentScale > 0)
  if i == 1 then savedMapID, savedW, savedH = mapID, w, h
  else assert(mapID == savedMapID and w == savedW and h == savedH) end
  assert(button:IsVisible() and button:IsEnabled())
  button:Click("LeftButton")
  assert(not map:IsShown())
end
SetCVar("miniWorldMap", 0)
"#;

const MAPPIN_WORLDQUEST_LUA_PART_1: &str = r#"
local map = assert(WorldMapFrame)
local slash = assert(SlashCmdList[SLASH_COMMAND.MAPPIN])
assert(LinkUtil.IsLinkHandlerRegistered(LinkTypes.WorldQuest))
assert(C_AddOns.IsAddOnLoaded("Blizzard_WorldMap"))
assert(not map:IsShown() and not C_Map.HasUserWaypoint())
SetCVar("miniWorldMap", 1)
SetCVar("questLogOpen", 0)
slash("not-a-map 25 75")
assert(not C_Map.HasUserWaypoint() and not map:IsShown())
for _, case in ipairs({{2248, 25, 75}, {2214, 60, 40}}) do
  local id, x, y = unpack(case)
  assert(C_Map.GetMapInfo(id))
  slash(("%d %d %d"):format(id, x, y))
  local point = assert(C_Map.GetUserWaypoint())
  local px, py = point.position:GetXY()
  assert(point.uiMapID == id and px == x/100 and py == y/100)
  assert(map:IsShown() and map:GetMapID() == id)
  local pin = assert(map:EnumeratePinsByTemplate("WaypointLocationPinTemplate")())
  local pinX, pinY = pin:GetPosition()
  assert(pinX == px and pinY == py)
  local scroll, child = map:GetCanvasContainer(), map:GetCanvas()
  assert(child == scroll.Child and scroll.targetScale > 0)
  assert(child:GetWidth() > 0 and child:GetHeight() > 0)
  ToggleWorldMap()
"#;

const MAPPIN_WORLDQUEST_LUA_PART_2: &str = r#"  assert(not map:IsShown())
end
SetItemRef("worldquest:1", "|Hworldquest:1|h[World Quest]|h", "LeftButton", DEFAULT_CHAT_FRAME)
assert(map:IsShown())
ToggleWorldMap()
assert(not map:IsShown())
C_Map.ClearUserWaypoint()
assert(not C_Map.HasUserWaypoint())
"#;

fn mappin_worldquest_lua() -> String {
    [MAPPIN_WORLDQUEST_LUA_PART_1, MAPPIN_WORLDQUEST_LUA_PART_2].concat()
}

fn assert_world_map_startup_order(startup: &[StartupAddon]) -> (usize, usize) {
    let canvas = startup.iter().position(|a| a.name == "Blizzard_MapCanvas").unwrap();
    let world = startup.iter().position(|a| a.name == "Blizzard_WorldMap").unwrap();
    assert!(canvas < world, "MapCanvas must load before WorldMap");
    assert_eq!(startup[canvas].toc_path.file_name().unwrap(), "Blizzard_MapCanvas.toc");
    assert_eq!(startup[world].toc_path.file_name().unwrap(), "Blizzard_WorldMap_Mainline.toc");
    let minimap = startup.iter().find(|a| a.name == "Blizzard_Minimap").unwrap();
    assert_eq!(minimap.toc_path.file_name().unwrap(), "Blizzard_Minimap_Mainline.toc");
    (canvas, world)
}

fn assert_world_map_not_loaded(env: &WowLuaEnv) {
    env.exec("assert(WorldMapFrame == nil and QuestLogOwnerMixin == nil)")
        .expect("WorldMap globals should not exist before the WorldMap addon loads");
}

fn load_startup_addons(env: &WowLuaEnv, startup: &[StartupAddon], canvas: usize, world: usize) {
    for (index, addon) in startup.iter().enumerate() {
        let loaded = env.state().borrow().addons.iter().any(|entry| entry.folder_name == addon.name && entry.loaded);
        if index == canvas || index == world {
            assert!(!loaded, "observe first load of {}", addon.name);
            assert_world_map_not_loaded(env);
        }
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("startup {}: {error}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" { env.restore_post_cleanup_globals(); }
        if index == canvas { assert_world_map_not_loaded(env); }
    }
}

fn load_normal_game_worldmap() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    let (canvas, world) = assert_world_map_startup_order(&startup);
    load_startup_addons(&env, &startup, canvas, world);
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    collect_lua_error_startup(&env);
    env
}

fn assert_lua_errors_empty(env: &WowLuaEnv) {
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn original_worldmap_display_state_controls_preserve_canvas_geometry() {
    let env = load_normal_game_worldmap();
    env.exec(&display_state_controls_lua()).expect("WorldMap controls should preserve canvas geometry");
    assert_lua_errors_empty(&env);
}

#[test]
fn original_minimap_zone_button_preserves_world_map_toggle() {
    let env = load_normal_game_worldmap();
    env.exec(MINIMAP_ZONE_BUTTON_LUA).expect("Minimap zone button should toggle the WorldMap");
    env.send_key_press("M", None).expect("dispatch M to open map");
    env.exec("assert(WorldMapFrame:IsShown() and WorldMapFrame:IsMaximized())").expect("M opens maximized map");
    env.send_key_press("M", None).expect("dispatch M to close map");
    env.exec("assert(not WorldMapFrame:IsShown())").expect("M closes map");
    assert_lua_errors_empty(&env);
}

#[test]
fn original_mappin_and_worldquest_link_preserve_world_map_open() {
    let env = load_normal_game_worldmap();
    env.exec(&mappin_worldquest_lua()).expect("Mappin and WorldQuest link should open the WorldMap");
    assert_lua_errors_empty(&env);
}
