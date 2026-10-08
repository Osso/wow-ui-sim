//! Mists simulator preservation only, not native historical protection parity.
#![cfg(feature = "client-mists")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_7_3_2_mists_session_actions_keep_previous_state_transitions() {
    for symbol in ["Logout", "Quit"] {
        let env = WowLuaEnv::new().expect("create Mists environment");
        env.state().borrow_mut().is_logged_in = true;
        let code = format!(
            r#"
            local invoke = function() {symbol}() end
            debug.setobjecttaint(invoke, 'Patch732Addon')
            invoke()
            "#
        );
        env.exec(&code).expect("unchanged older-profile action");
        let state = env.state().borrow();
        assert_eq!(state.is_logged_in, symbol != "Logout");
        assert_eq!(state.simulator_exit_requested, symbol == "Quit");
    }
}
