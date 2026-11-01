//! Retail and PTR HUD set IDs; values use Retail build 70688 evidence.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn hud_selection_queries_return_fixed_set_ids() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local cases = {
            {C_UIWidgetManager.GetTopCenterWidgetSetID, 1},
            {C_UIWidgetManager.GetBelowMinimapWidgetSetID, 2},
            {C_UIWidgetManager.GetPowerBarWidgetSetID, 283},
            {C_UIWidgetManager.GetObjectiveTrackerWidgetSetID, 240},
        }
        for _, case in ipairs(cases) do
            assert(select('#', case[1]()) == 1)
            assert(case[1]() == case[2])
        end
        "#,
    )
    .expect("HUD selectors return their fixed IDs");
}
