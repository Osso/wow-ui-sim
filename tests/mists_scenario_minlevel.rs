//! Bounded supplied Mists minimum; empty-catalog presentation is preservation.
#![cfg(feature = "client-mists")]

use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use crate::mists_lfr_availability::load_original_pve_frame;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::toc::TocFile;

const UNKNOWN: &str = "native-unknown: C_PartyInfo.GetMinLevel has no supplied active-party minimum or documented default";
const UNSUPPORTED: &str = "unsupported: C_PartyInfo.GetMinLevel explicit category";

fn load_original_scenario() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_GroupFinder")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    for suffix in [
        "Shared/ScenarioFinder_Shared.lua",
        "Shared/ScenarioFinder.xml",
    ] {
        assert!(toc.files.iter().any(|file| file.ends_with(suffix)));
    }
    let env = load_original_pve_frame();
    env.exec(
        r#"
        local frame = assert(ScenarioFinderFrame, 'original XML frame')
        assert(frame:GetParent() == GroupFinderFrame)
        assert(frame.NoScenariosCover:GetParent() == frame)
        assert(frame.NoScenariosCover.Label:GetParent() == frame.NoScenariosCover)
        assert(frame:GetScript('OnShow') == ScenarioFinderFrame_OnShow)
        assert(frame:GetScript('OnEvent') == ScenarioFinderFrame_OnEvent)
        local info = debug.getinfo(ScenarioFinderFrame_UpdateAvailability, 'S')
        local suffix = 'Blizzard_GroupFinder/Shared/ScenarioFinder_Shared.lua'
        local source = info.source:gsub('\\', '/')
        assert(info.what == 'Lua' and source:sub(-#suffix) == suffix)
        assert(#GetScenariosChoiceOrder() == 0 and GetNumRandomScenarios() == 0)
        print('minimum consumer provenance', source, info.linedefined, info.lastlinedefined)
    "#,
    )
    .expect("selected original ScenarioFinder identity before line58 proof");
    env
}

fn assert_empty_catalog_preservation(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(
        r#"
        local ok, reason = xpcall(ScenarioFinderFrame_UpdateAvailability, debug.traceback)
        if not ok then error(reason, 0) end
        assert(ScenarioFinderFrame.NoScenariosCover:IsShown())
        assert(ScenarioFinderFrame.NoScenariosCover.Label:GetText() == NO_SCENARIO_AVAILABLE)
    "#,
    )
    .expect("preservation: original empty-catalog consumer must complete past line58");
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}

#[test]
fn original_empty_catalog_preserves_cover_after_supplied_minimum() {
    let env = load_original_scenario();
    assert_eq!(env.state().borrow().mists_party_min_level, None);
    assert_empty_catalog_preservation(&env);
    assert_eq!(
        query_minimum(&env),
        env.state().borrow().player.level as f64
    );
    env.state().borrow_mut().mists_party_min_level = Some(85.0);
    assert_empty_catalog_preservation(&env);
    env.state().borrow_mut().mists_party_min_level = Some(90.0);
    assert_empty_catalog_preservation(&env);
    env.state().borrow_mut().mists_party_min_level = None;
    assert_empty_catalog_preservation(&env);
    assert_eq!(
        query_minimum(&env),
        env.state().borrow().player.level as f64
    );
}

fn query_minimum(env: &WowLuaEnv) -> f64 {
    let (count, minimum): (i32, f64) = env.eval(
        "local function pack(...) return select('#', ...), ... end; return pack(C_PartyInfo.GetMinLevel())"
    ).expect("supplied query returns one exact number");
    assert_eq!(count, 1);
    minimum
}

#[test]
fn supplied_minimum_is_exact_environment_local_and_solo_defaults_to_player() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    first.state().borrow_mut().player.level = 42;
    assert_eq!(query_minimum(&first), 42.0);
    first.state().borrow_mut().mists_party_min_level = Some(85.0);
    second.state().borrow_mut().mists_party_min_level = Some(90.0);
    assert_eq!(query_minimum(&first), 85.0);
    assert_eq!(query_minimum(&second), 90.0);
    first.state().borrow_mut().mists_party_min_level = Some(90.0);
    assert_eq!(query_minimum(&first), 90.0);
    second.state().borrow_mut().mists_party_min_level = None;
    second.state().borrow_mut().player.level = 37;
    assert_eq!(query_minimum(&second), 37.0);
    assert_eq!(query_minimum(&first), 90.0);
}

#[test]
fn group_minimum_still_requires_a_supplied_value() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().party_group_active = true;
    let error = env
        .exec("C_PartyInfo.GetMinLevel()")
        .expect_err("group minimum remains an explicit supplied input");
    assert!(error.to_string().contains(UNKNOWN), "{error}");
    for minimum in [36.0, 58.0] {
        env.state().borrow_mut().mists_party_min_level = Some(minimum);
        assert_eq!(query_minimum(&env), minimum);
    }
}

#[test]
fn explicit_category_is_unsupported_even_with_supplied_minimum() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().mists_party_min_level = Some(85.0);
    for category in [1, 2] {
        let error = env
            .exec(&format!("C_PartyInfo.GetMinLevel({category})"))
            .expect_err("explicit categories have no supplied/native contract");
        assert!(error.to_string().contains(UNSUPPORTED), "{error}");
    }
    assert_eq!(query_minimum(&env), 85.0);
}

#[test]
fn original_scenario_cover_tracks_solo_player_level() {
    let env = load_original_scenario();
    {
        let mut sim = env.state().borrow_mut();
        let mut dungeon = sim.lfd_dungeons[0].clone();
        dungeon.dungeon_id = 987654;
        dungeon.min_level = 50;
        dungeon.max_level = 60;
        sim.lfd_dungeons.push(dungeon);
        sim.player.level = 40;
    }
    env.exec(
        r#"
        local name, scenarios = debug.getupvalue(ScenarioFinderFrame_UpdateAvailability, 1)
        assert(name == 'allScenarios')
        scenarios[1] = 987654
    "#,
    )
    .unwrap();
    env.exec("local ok, reason = xpcall(ScenarioFinderFrame_UpdateAvailability, debug.traceback); if not ok then error(reason, 0) end")
        .expect("original ScenarioFinder_Shared.lua:58 must consume the solo default");
    env.exec("assert(ScenarioFinderFrame.NoScenariosCover:IsShown())")
        .unwrap();
    env.state().borrow_mut().player.level = 55;
    env.exec("ScenarioFinderFrame_UpdateAvailability()")
        .unwrap();
    env.exec("assert(not ScenarioFinderFrame.NoScenariosCover:IsShown())")
        .unwrap();
}
