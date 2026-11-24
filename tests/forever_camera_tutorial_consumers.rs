//! Forever original camera and tutorial consumer proofs.
#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn load_panel_consumers() -> WowLuaEnv {
    let env = new_blizzard_addon_env(&blizzard_ui_dir());
    crate::common::panel_fixtures::load_panel_addons(&env);
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn forever_camera_fov_defaults_drive_graphics_slider() {
    let env = load_panel_consumers();
    env.exec(
        r#"
        local _, minValue, maxValue = GetCameraFOVDefaults()
        local options = Settings.CreateSliderOptions(minValue, maxValue, 5)
        local setting = Settings.GetSetting("PROXY_CAMERA_FOV")
        A_Admin.SetCameraFOV(60)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 60)
        A_Admin.SetCameraFOV(80)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 80)
    "#,
    )
    .expect("Forever Graphics slider should refresh from supplied camera FOV values");
}

#[test]
fn forever_tutorial_enable_state_controls_npe_bootstrap() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    crate::common::panel_fixtures::seed_addon_search_paths(&env);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXML"], &[]);
    crate::common::load_blizzard_addon_bootstrap(
        &env,
        &ui,
        "Blizzard_NewPlayerExperience",
        "Blizzard_NewPlayerExperience_Bootstrap.lua",
    );
    env.exec(
        r#"
        local addon = "Blizzard_NewPlayerExperience"
        assert(not C_AddOns.IsAddOnLoaded(addon), "NPE starts unloaded")
        A_Admin.SetPlayerNPERestricted(true)
        A_Admin.SetTutorialsEnabled(false)
        NPE_LoadUI()
        assert(not C_AddOns.IsAddOnLoaded(addon), "disabled tutorials keep NPE unloaded")
        A_Admin.SetTutorialsEnabled(true)
        NPE_LoadUI()
        assert(C_AddOns.IsAddOnLoaded(addon), "enabled tutorials load NPE for a restricted player")
    "#,
    )
    .expect("Forever NPE bootstrap should obey supplied tutorial enable state");
}

#[test]
fn forever_tutorial_frame_reads_supplied_flags() {
    let env = load_panel_consumers();
    env.exec(
        r#"
        A_Admin.SetTutorialFlag(17, true)
        TutorialFrame_NewTutorial(17)
        assert(not TutorialFrame:IsShown())
        A_Admin.SetTutorialFlag(17, false)
        TutorialFrame_NewTutorial(17)
        assert(TutorialFrame:IsShown())
    "#,
    )
    .expect("Forever TutorialFrame should observe supplied flag state");
}

#[test]
fn forever_settings_reset_clears_flags_seen_by_tutorial_frame() {
    let env = load_panel_consumers();
    load_blizzard_addon_closure_into_env(
        &env,
        &blizzard_ui_dir(),
        &["Blizzard_TutorialManager"],
        &[],
    );
    env.exec(
        r#"
        A_Admin.SetTutorialFlag(17, true)
        InterfaceOverrides.ShowTutorialsOnButtonClick()
        assert(not IsTutorialFlagged(17))
        A_Admin.SetTutorialFlag(18, true)
        TutorialFrame_NewTutorial(18)
        assert(not TutorialFrame:IsShown())
    "#,
    )
    .expect("Forever settings reset should clear tutorial bits observed by TutorialFrame");
}
