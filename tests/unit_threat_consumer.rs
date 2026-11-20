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
    let result: String = env.eval(r#"
        assert(type(CompactUnitFrame_UpdateHealthColor) == "function")
        local frame = CreateFrame("Frame")
        frame.unit = "player"
        frame.displayedUnit = "player"
        frame.optionTable = {}
        frame.displayThreatHealthBarColor = true
        frame.healthBar = CreateFrame("StatusBar", nil, frame)
        A_Admin.SetUnitThreatSituation("player", "target", 1, 60, 50, 500)
        A_Admin.SetUnitThreatSituation("pet", "target", 3, 100, 100, 1000)
        CompactUnitFrame_UpdateHealthColor(frame)
        local r1, g1, b1 = frame.healthBar:GetStatusBarColor()
        local isTanking, status, scaled, raw, value = UnitDetailedThreatSituation("player", "target")
        assert(isTanking == false and status == 1 and scaled == 60 and raw == 50 and value == 500)
        assert(UnitThreatSituation("player") == 1)
        assert(UnitThreatPercentageOfLead("player", "target") == 50)
        A_Admin.SetUnitThreatSituation("player", "target", 2, 100, 100, 1200)
        CompactUnitFrame_UpdateHealthColor(frame)
        local r2, g2, b2 = frame.healthBar:GetStatusBarColor()
        assert(UnitDetailedThreatSituation("player", "target") == true)
        assert(UnitThreatSituation("player") == 2)
        return string.format("first=%s,%s,%s second=%s,%s,%s", tostring(r1), tostring(g1), tostring(b1), tostring(r2), tostring(g2), tostring(b2))
    "#).expect("original Blizzard threat consumer runs");
    let (first, second) = result.split_once(" second=").expect("color pair is returned");
    assert_ne!(first, second, "consumer color must change for the supplied threat states");
}
