#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
}

fn load_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment initializes");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn unit_threat_indicator_tracks_two_supplied_states() {
    let env = load_game_ui();
    let result: bool = env.eval(r#"
        assert(type(UnitFrame_UpdateThreatIndicator) == "function")
        SetCVar("threatWarning", 1)
        A_Admin.SetTarget("Threat Test Mob", 63, 1, true)
        local icon = TargetFrame.threatIndicator
        assert(icon ~= nil)
        A_Admin.SetUnitThreatSituation("player", "target", 0, 0, 0, 0)
        UnitFrame_UpdateThreatIndicator(icon, nil, "player")
        local first = icon:IsShown()
        A_Admin.SetUnitThreatSituation("player", "target", 3, 100, 100, 1000)
        UnitFrame_UpdateThreatIndicator(icon, nil, "player")
        return first == false and icon:IsShown() == true
    "#).expect("original Blizzard threat consumer runs");
    assert!(result, "consumer output must change for the supplied threat states");
}
