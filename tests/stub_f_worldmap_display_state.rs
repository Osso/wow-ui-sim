//! Retail preservation of the original WorldMap display-state owner through real controls.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::collect_lua_error_startup;

#[test]
fn original_worldmap_display_state_controls_preserve_canvas_geometry() {
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

    for (index, addon) in startup.into_iter().enumerate() {
        let loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded);
        if index == canvas_index || index == world_index {
            assert!(!loaded, "observe the actual first load of {}", addon.name);
            env.exec("assert(WorldMapFrame == nil and QuestLogOwnerMixin == nil)")
                .expect("no WorldMap owner/consumer while entering a prerequisite");
        }
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|e| panic!("startup {}: {e}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
        if index == canvas_index {
            env.exec("assert(WorldMapFrame == nil and QuestLogOwnerMixin == nil)")
                .expect("MapCanvas completion precedes WorldMap ownership");
        }
    }
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    collect_lua_error_startup(&env);

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
