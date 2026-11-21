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
fn compact_aggro_highlight_tracks_two_supplied_states() {
    let env = load_game_ui();
    let result: bool = env.eval(r#"
        assert(type(CompactUnitFrame_UpdateAggroHighlight) == "function")
        local cr, cg, cb = GetThreatStatusColor(3)
        assert(cr == 1 and cg == 0 and cb == 0 and GetThreatStatusColor(99) == nil)
        A_Admin.SetPartySize(1)
        local frame = CreateFrame("Frame")
        frame.unit = "player"
        frame.displayedUnit = "player"
        frame.optionTable = { displayAggroHighlight = true }
        frame.aggroHighlight = frame:CreateTexture()
        A_Admin.SetUnitThreatSituation("player", "target", 0, 50, 50, 500)
        A_Admin.SetUnitThreatSituation("pet", "target", 3, 100, 100, 1000)
        CompactUnitFrame_UpdateAggroHighlight(frame)
        local first = frame.aggroHighlight:IsShown()
        local isTanking, status, scaled, raw, value = UnitDetailedThreatSituation("player", "target")
        local firstSituation = UnitThreatSituation("player")
        local firstLead = UnitThreatPercentageOfLead("player", "target")
        A_Admin.SetUnitThreatSituation("player", "target", 2, 100, 100, 1200)
        A_Admin.SetUnitThreatSituation("pet", "target", 0, 80, 80, 1000)
        CompactUnitFrame_UpdateAggroHighlight(frame)
        local second = frame.aggroHighlight:IsShown()
        assert(first == false and second == true, "Blizzard aggro highlight must change")
        assert(isTanking == false and status == 0 and scaled == 50 and raw == 50 and value == 500)
        assert(firstSituation == 0 and firstLead == 50)
        assert(UnitDetailedThreatSituation("player", "target") == true)
        assert(UnitThreatSituation("player") == 2)
        assert(UnitThreatPercentageOfLead("player", "target") == 100)
        return first ~= second
    "#).expect("original Blizzard threat consumer runs");
    assert!(result, "consumer visibility must change for the supplied threat states");
}
