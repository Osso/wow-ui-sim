//! C_HousingInspectMode over supplied hover state. Moved from
//! patch_12_0_5_sweep_globals.rs: Forever's HousingInspectModeUIDocumentation.lua
//! is byte-identical to Retail's and Forever ships Blizzard_HousingInspectModeUI.
#![cfg(feature = "housing-inspect-mode")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua environment")
}

fn drained_event_names(env: &WowLuaEnv) -> Vec<String> {
    let events = env.state().borrow_mut().events.drain();
    events.into_iter().map(|event| event.name).collect()
}

#[test]
fn housing_inspect_mode_transitions_fire_state_events_and_gate_hover() {
    let env = env();
    drained_event_names(&env);
    env.state().borrow_mut().housing.inspect_mode.hovered_decor_guid =
        Some("Housing-4-1-100-4F2A".into());
    env.exec(
        r#"
        assert(C_HousingInspectMode.IsInInspectMode() == false)
        assert(C_HousingInspectMode.IsHoveringDecor() == false)
        assert(C_HousingInspectMode.GetHoveredDecorGUID() == '')
        C_HousingInspectMode.EnterInspectMode()
        C_HousingInspectMode.EnterInspectMode()
        assert(C_HousingInspectMode.IsInInspectMode() == true)
        assert(C_HousingInspectMode.IsHoveringDecor() == true)
        assert(C_HousingInspectMode.GetHoveredDecorGUID() == 'Housing-4-1-100-4F2A')
        "#,
    )
    .expect("entering inspect mode reports the hovered decor");
    assert_eq!(
        drained_event_names(&env),
        ["HOUSING_INSPECT_MODE_STATE_UPDATED"],
        "only the real transition fires"
    );
    env.exec(
        r#"
        C_HousingInspectMode.ExitInspectMode()
        assert(C_HousingInspectMode.IsInInspectMode() == false)
        assert(C_HousingInspectMode.IsHoveringDecor() == false)
        "#,
    )
    .expect("exiting hides the hover");
    assert_eq!(
        drained_event_names(&env),
        ["HOUSING_INSPECT_MODE_STATE_UPDATED"]
    );
}
