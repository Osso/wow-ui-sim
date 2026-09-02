//! The Edit Mode selection highlight of a status tracking bar container is
//! edit-mode-only art (`editmode-actionbar-highlight-NineSlice-*`, corners
//! 8 units outside the frame). Startup once called
//! `EditModeAccountSettingsMixin:RefreshStatusTrackingBar2()`, which sets
//! `isInEditMode` and `HighlightSystem()` on `SecondaryStatusTrackingBarContainer`,
//! so every render showed light-blue brackets at both bars' ends and a blue
//! tint over the reputation bar where the client shows the plain frame.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    for (name, toc_path) in &discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game) {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[test]
fn status_tracking_bars_carry_no_edit_mode_highlight_after_startup() {
    let env = load_settled_game_ui();
    let report: String = env
        .eval(
            r#"
            local out = {}
            for _, name in ipairs({ "MainStatusTrackingBarContainer", "SecondaryStatusTrackingBarContainer" }) do
                local c = _G[name]
                if c.Selection and c.Selection:IsShown() then out[#out + 1] = name .. ": Selection shown" end
                if c.isHighlighted == true then out[#out + 1] = name .. ": isHighlighted" end
                if c.isInEditMode == true then out[#out + 1] = name .. ": isInEditMode" end
            end
            if EditModeManagerFrame:IsShown() then out[#out + 1] = "EditModeManagerFrame shown" end
            return table.concat(out, "; ")
            "#,
        )
        .expect("status tracking state");
    assert_eq!(report, "", "edit-mode-only state left on a status tracking bar: {report}");
}
