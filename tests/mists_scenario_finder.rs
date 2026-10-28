//! Supplied Mists scenario eligibility through the unchanged original PVEFrame.
#![cfg(feature = "client-mists")]

use crate::mists_lfr_availability::load_original_pve_frame;
use wow_ui_sim::lua_api::WowLuaEnv;

fn prepare_scenario_presentation() -> WowLuaEnv {
    let env = load_original_pve_frame();
    assert_eq!(env.state().borrow().scenario_finder_availability, None);
    env.state().borrow_mut().lfr_availability = Some(true);
    // Supply only the original Lua presentation list. Mists catalog production is
    // outside this eligibility contract; no function or handler is replaced.
    env.exec("ScenariosList = { 90001 }").unwrap();
    env
}

fn assert_original_scenario(env: &WowLuaEnv, can_use: bool, reason: &str, level: i32) {
    {
        let mut sim = env.state().borrow_mut();
        sim.scenario_finder_availability = Some((can_use, reason.to_owned()));
        sim.player.level = level;
    }
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec("GroupFinderFrame:GetScript('OnEvent')(GroupFinderFrame, 'PLAYER_LEVEL_CHANGED')")
        .expect("original PVEFrame.lua:270 must consume supplied scenario eligibility");
    let (shown, enabled, tooltip): (bool, bool, Option<String>) = env
        .eval(
            "local button = GroupFinderFrame.groupButton4; return button:IsShown(), button:IsEnabled(), button.tooltip",
        )
        .unwrap();
    assert!(shown, "original nonempty scenario presentation list");
    assert_eq!(enabled, can_use);
    assert_eq!(
        tooltip.as_deref(),
        if can_use { None } else { Some(reason) }
    );
    let (count, actual_use, actual_reason): (i32, bool, String) = env
        .eval(
            "local function pack(...) return select('#', ...), ... end; return pack(C_LFGInfo.CanPlayerUseScenarioFinder())",
        )
        .unwrap();
    assert_eq!(
        (count, actual_use, actual_reason.as_str()),
        (2, can_use, reason)
    );
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}

fn exercise_original_scenario_and_isolation(initially_allowed: bool) {
    let first = prepare_scenario_presentation();
    assert_original_scenario(&first, initially_allowed, "initial supplied reason", 1);
    assert_original_scenario(&first, false, "scenario service unavailable", 90);
    assert_original_scenario(&first, true, "", 90);
    let second = prepare_scenario_presentation();
    assert_original_scenario(
        &second,
        !initially_allowed,
        "other environment restriction",
        1,
    );
    assert_original_scenario(&second, true, "other allowed reason", 1);
    assert_original_scenario(&second, false, "changed restriction", 90);
    first
        .exec("assert(GroupFinderFrame.groupButton4:IsEnabled())")
        .unwrap();
    assert_original_scenario(&first, false, "first environment changed", 1);
    second
        .exec("assert(GroupFinderFrame.groupButton4.tooltip == 'changed restriction')")
        .unwrap();
    assert_original_scenario(&first, true, "recovered", 1);

    first.state().borrow_mut().scenario_finder_availability = None;
    first
        .exec("GroupFinderFrame:GetScript('OnEvent')(GroupFinderFrame, 'PLAYER_LEVEL_CHANGED')")
        .expect("original PVEFrame.lua:270 must accept the scenario default");
    let (allowed, reason): (bool, String) = first
        .eval("return C_LFGInfo.CanPlayerUseScenarioFinder()")
        .unwrap();
    assert!(allowed);
    assert_eq!(reason, "");
    first
        .exec("assert(GroupFinderFrame.groupButton4:IsEnabled())")
        .unwrap();
    assert_original_scenario(&first, false, "supplied after unset", 90);
    assert_original_scenario(&first, true, "", 90);
}

#[test]
fn original_pve_frame_uses_stock_scenario_default() {
    let env = prepare_scenario_presentation();
    env.exec("local ok, reason = xpcall(function() GroupFinderFrame:GetScript('OnEvent')(GroupFinderFrame, 'PLAYER_LEVEL_CHANGED') end, debug.traceback); if not ok then error(reason, 0) end")
        .expect("original PVEFrame.lua:270 must accept the stock scenario default");
    let (allowed, reason): (bool, String) = env
        .eval("return C_LFGInfo.CanPlayerUseScenarioFinder()")
        .unwrap();
    assert!(allowed);
    assert_eq!(reason, "");
    env.exec("assert(GroupFinderFrame.groupButton4:IsEnabled())")
        .unwrap();
    assert_original_scenario(&env, false, "scenario unavailable", 1);
    assert_original_scenario(&env, true, "", 90);
}

#[test]
fn original_pve_frame_scenario_allowed_then_transitions_and_isolation() {
    exercise_original_scenario_and_isolation(true);
}

#[test]
fn original_pve_frame_scenario_denied_then_transitions_and_isolation() {
    exercise_original_scenario_and_isolation(false);
}
