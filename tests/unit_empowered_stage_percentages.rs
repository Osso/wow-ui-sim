//! Per-stage fractions from the same timeline used by empowered durations.
#![cfg(feature = "player-cast-durations")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn empowered_stage_percentages_track_uneven_stages_hold_and_updates() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(include_str!(
        "fixtures/unit_empowered_stage_percentages.lua"
    ))
    .unwrap();
    env.exec("CheckEmpoweredStageFractions()").unwrap();
}

#[test]
fn empowered_stage_percentages_disappear_on_replacement_and_completion() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.StartEmpower(357208, 'Replaced', '', {1, 2}, 3)
        A_Admin.SetCasting(19750, 'Ordinary cast', '', 30)
        assert(select('#', UnitEmpoweredStagePercentages('player')) == 0)
        A_Admin.StartEmpower(357208, 'Completing', '', {1, 1}, 0)
        local values = UnitEmpoweredStagePercentages('player')
        assert(#values == 3 and values[1] == 0.5 and values[2] == 0.5 and values[3] == 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(3);
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(select('#', UnitEmpoweredStagePercentages('player')) == 0)")
        .unwrap();
}
