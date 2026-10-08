//! Bounded existing PvP role backing and already-absent historical names.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_role_state_and_retirements(env: &WowLuaEnv) {
    for (tank, healer, dps) in [(true, false, true), (false, true, false), (false, false, false)] {
        env.eval::<()>(&format!("SetPVPRoles({tank}, {healer}, {dps})"))
            .expect("write role selection");
        let roles: (bool, bool, bool) = env
            .eval("return GetPVPRoles()")
            .expect("read stored role selection");
        assert_eq!(roles, (tank, healer, dps));
        let state = env.state().borrow();
        assert_eq!(state.lfg_roles.tank, tank);
        assert_eq!(state.lfg_roles.healer, healer);
        assert_eq!(state.lfg_roles.dps, dps);
    }
    let absent: bool = env
        .eval(
            r#"
            assert(rawget(_G, 'PrepVoidStorageForTransmogrify') == nil)
            assert(_G.PrepVoidStorageForTransmogrify == nil)
            local frame = CreateFrame('Frame')
            local ok, err = pcall(frame.RegisterEvent, frame, 'UNIT_DYNAMIC_FLAGS')
            assert(not ok and string.find(err, 'Attempt to register unknown event', 1, true))
            assert(not frame:IsEventRegistered('UNIT_DYNAMIC_FLAGS'))
            return true
            "#,
        )
        .expect("historical names remain absent without new retirement gates");
    assert!(absent);
}

#[test]
fn patch_5_3_0_bare_role_state_and_absence() {
    let env = WowLuaEnv::new().expect("bare environment");
    assert_role_state_and_retirements(&env);
}

prefork_full_ui_case! {
fn patch_5_3_0_cached_role_state_and_absence(env: &WowLuaEnv) {
    assert_role_state_and_retirements(env);
}
}
