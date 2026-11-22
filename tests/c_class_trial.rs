#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn class_trial_admin_state_supplies_trial_flag_and_deadline() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        A_Admin.SetClassTrialState(true, 130)
        assert(C_ClassTrial.IsClassTrialCharacter())
        assert(C_ClassTrial.GetClassTrialLogoutTimeSeconds() == 130)

        A_Admin.SetClassTrialState(true, 250)
        assert(C_ClassTrial.IsClassTrialCharacter())
        assert(C_ClassTrial.GetClassTrialLogoutTimeSeconds() == 250)

        A_Admin.SetClassTrialState(false, 0)
        assert(not C_ClassTrial.IsClassTrialCharacter())
        assert(C_ClassTrial.GetClassTrialLogoutTimeSeconds() == 0)
        "#,
    )
    .expect("host-supplied class-trial state should feed the C API");
}
