//! Retail preservation of the original WorldMap display-state owner through real controls.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::collect_lua_error_startup;

fn load_normal_game_worldmap() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    let canvas_index = startup
        .iter()
        .position(|a| a.name == "Blizzard_MapCanvas")
        .expect("MapCanvas is selected for normal Retail Game startup");
    let world_index = startup
        .iter()
        .position(|a| a.name == "Blizzard_WorldMap")
        .expect("WorldMap is selected for normal Retail Game startup");
    assert!(
        canvas_index < world_index,
        "MapCanvas is a WorldMap prerequisite"
    );
    assert_eq!(
        startup[canvas_index].toc_path.file_name().unwrap(),
        "Blizzard_MapCanvas.toc"
    );
    assert_eq!(
        startup[world_index].toc_path.file_name().unwrap(),
        "Blizzard_WorldMap_Mainline.toc"
    );

    let minimap = startup
        .iter()
        .find(|a| a.name == "Blizzard_Minimap")
        .expect("Minimap is selected for normal Retail Game startup");
    assert_eq!(
        minimap.toc_path.file_name().unwrap(),
        "Blizzard_Minimap_Mainline.toc"
    );

    let assert_early_world_map = || {
        env.exec(
            r#"
            assert(WorldMapFrame == nil and QuestLogOwnerMixin == nil)
            assert(debug.getinfo(ToggleWorldMap, "S").what == "C")
            assert(OpenWorldMap == nil or debug.getinfo(OpenWorldMap, "S").what == "C")
            "#,
        )
        .expect("only bootstrap verbs before the original WorldMap owner");
    };
    for (index, addon) in startup.into_iter().enumerate() {
        let loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded);
        if index == canvas_index || index == world_index {
            assert!(!loaded, "observe the actual first load of {}", addon.name);
            assert_early_world_map();
        }
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|e| panic!("startup {}: {e}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
        if index == canvas_index {
            assert_early_world_map();
        }
    }
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    collect_lua_error_startup(&env);
    env
}

#[test]
fn original_worldmap_display_state_controls_preserve_canvas_geometry() {
    let env = load_normal_game_worldmap();
    env.exec(
        r#"
        local map = assert(WorldMapFrame)
        local owner = QuestLogOwnerMixin.SetDisplayState
        local function source(fn)
            return debug.getinfo(fn, "S").source:gsub('\\', '/')
        end
        assert(source(owner):find("Blizzard_WorldMap/QuestLogOwnerMixin.lua", 1, true))
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
            assert(map.SetDisplayState == owner, "original owner throughout all transitions")
            local scroll, child = map:GetCanvasContainer(), map:GetCanvas()
            assert(scroll == map.ScrollContainer and child == scroll.Child)
            local w, h, sw, sh = map:GetWidth(), map:GetHeight(), scroll:GetWidth(), scroll:GetHeight()
            assert(w > 0 and h > 0 and sw > 0 and sh > 0)
            local cw, ch = child:GetWidth(), child:GetHeight()
            assert(cw > 0 and ch > 0 and scroll.currentScale > 0 and scroll.targetScale > 0)
            return w, h, sw, sh, cw, ch
        end

        SetCVar("miniWorldMap", 1)
        SetCVar("questLogOpen", 0)
        for i = 1, 2 do
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
            assert(restoredW == w and restoredH == h and map:GetMapID() == mapID)

            ToggleWorldMap()
            assert(not map:IsShown())
        end
        "#,
    )
    .expect("original display-state controls and canvas geometry preserve behavior");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn original_minimap_zone_button_preserves_world_map_toggle() {
    let env = load_normal_game_worldmap();
    env.exec(
        r#"
        local map = assert(WorldMapFrame)
        local button = assert(MinimapCluster.ZoneTextButton)
        local owner = ToggleWorldMap
        local function source(fn)
            return debug.getinfo(fn, "S").source:gsub('\\', '/')
        end
        assert(source(owner):find("Blizzard_WorldMap/Blizzard_WorldMap.lua", 1, true))
        assert(button.OnClick == MinimapZoneTextButtonMixin.OnClick)
        assert(source(button.OnClick):find("Blizzard_Minimap/Mainline/Minimap.lua", 1, true))
        assert(type(button:GetScript("OnClick")) == "function")
        assert(button:IsVisible() and button:IsEnabled())
        assert(GetBindingKey("TOGGLEWORLDMAP") == "M", "normal M binding")
        assert(not map:IsShown())

        SetCVar("miniWorldMap", 1)
        SetCVar("questLogOpen", 0)
        local savedMapID, savedW, savedH
        for i = 1, 2 do
            button:Click("LeftButton")
            assert(map:IsShown() and map:IsMinimized() and not map.QuestLog:IsShown())
            assert(ToggleWorldMap == owner)
            local scroll, child = map:GetCanvasContainer(), map:GetCanvas()
            assert(child == scroll.Child)
            local mapID, w, h = map:GetMapID(), scroll:GetWidth(), scroll:GetHeight()
            assert(mapID > 0 and w > 0 and h > 0)
            assert(child:GetWidth() > 0 and child:GetHeight() > 0 and scroll.currentScale > 0)
            if i == 1 then
                savedMapID, savedW, savedH = mapID, w, h
            else
                assert(mapID == savedMapID and w == savedW and h == savedH)
            end
            assert(button:IsVisible() and button:IsEnabled())
            button:Click("LeftButton")
            assert(not map:IsShown() and ToggleWorldMap == owner)
        end
        SetCVar("miniWorldMap", 0)
        "#,
    )
    .expect("original minimap zone-text clicks toggle the original map owner");
    // The simulator's normal M dispatch models the same TOGGLEWORLDMAP
    // action named in Retail Bindings_Standard.xml; the XML button above
    // independently proves an original Blizzard Lua consumer.
    env.send_key_press("M", None)
        .expect("dispatch M to open map");
    env.exec("assert(WorldMapFrame:IsShown() and WorldMapFrame:IsMaximized())")
        .expect("M opens the maximized map");
    env.send_key_press("M", None)
        .expect("dispatch M to close map");
    env.exec("assert(not WorldMapFrame:IsShown())")
        .expect("M closes the map");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn original_mappin_and_worldquest_link_preserve_world_map_open() {
    let env = load_normal_game_worldmap();
    env.exec(
        r#"
        local map = assert(WorldMapFrame)
        local open = OpenWorldMap
        local slash = assert(SlashCmdList[SLASH_COMMAND.MAPPIN])
        local function source(fn)
            return debug.getinfo(fn, "S").source:gsub('\\', '/')
        end
        assert(source(open):find("Blizzard_WorldMap/Blizzard_WorldMap.lua", 1, true))
        assert(source(OpenMapToUserWaypoint):find("Blizzard_WorldMap/Blizzard_WorldMap.lua", 1, true))
        assert(source(slash):find("Blizzard_SharedMapDataProviders/WaypointLocationDataProvider.lua", 1, true))
        assert(source(SetItemRef):find("Blizzard_UIPanels_Game/Mainline/ItemRef.lua", 1, true))
        assert(LinkUtil.IsLinkHandlerRegistered(LinkTypes.WorldQuest))
        assert(C_AddOns.IsAddOnLoaded("Blizzard_WorldMap"))
        assert(not map:IsShown() and not C_Map.HasUserWaypoint())

        -- Original registered slash callback with ordinary user-entered values.
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
            assert(OpenWorldMap == open)
            ToggleWorldMap()
            assert(not map:IsShown())
        end

        -- The stock WorldQuest link handler opens without a map argument.
        SetItemRef("worldquest:1", "|Hworldquest:1|h[World Quest]|h", "LeftButton", DEFAULT_CHAT_FRAME)
        assert(map:IsShown() and OpenWorldMap == open)
        ToggleWorldMap()
        assert(not map:IsShown())
        C_Map.ClearUserWaypoint()
        assert(not C_Map.HasUserWaypoint())
        "#,
    )
    .expect("original slash and hyperlink handlers open the original map");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}
