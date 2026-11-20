#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn loaded_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![common::blizzard_ui_dir()];
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn original_game_menu_populates_buttons_and_handles_escape() {
    test_timeout! {
        let env = loaded_game_ui();
        env.exec(r#"
            GameMenuFrame_Show()
            assert(GameMenuFrame:IsShown(), "original show path must open menu")
            local options = false
            for button in GameMenuFrame.buttonPool:EnumerateActive() do
                if button:GetText() == GAMEMENU_OPTIONS then options = true end
            end
            assert(options, "original menu must populate its options button")
            GameMenuFrame_EscapePressed()
            assert(not GameMenuFrame:IsShown(), "original escape path must close menu")
        "#).expect("original game menu consumer");
    }
}

#[test]
fn original_tracker_manager_initializes_header_and_collapse_button() {
    test_timeout! {
        let env = loaded_game_ui();
        env.exec(r#"
            local tracker = ObjectiveTrackerFrame
            tracker.init, tracker.headerText = false, "Preservation tracker"
            ObjectiveTrackerManager.backgroundAlpha = 0.375
            ObjectiveTrackerManager:AddContainer(tracker)
            assert(tracker.Header.Text:GetText() == "Preservation tracker")
            assert(tracker.NineSlice:GetAlpha() == 0.375)
            tracker:SetCollapsed(false)
            local button = tracker.Header.MinimizeButton
            button:GetScript("OnClick")(button, "LeftButton")
            assert(tracker:IsCollapsed(), "original minimize click must collapse")
            assert(button:GetNormalTexture():GetAtlas() == "ui-questtrackerbutton-expand-all")
        "#).expect("original tracker manager and header consumers");
    }
}
