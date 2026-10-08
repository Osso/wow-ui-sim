//! Source contract: insecure Logout/Quit must not mutate existing session state.
//! Native notification/error text and logout countdown timing are not claimed.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn assert_insecure_session_actions_blocked(env: &WowLuaEnv) {
    let mut observations = serde_json::Map::new();
    for symbol in ["Logout", "Quit"] {
        {
            let mut state = env.state().borrow_mut();
            state.is_logged_in = true;
            state.simulator_exit_requested = false;
        }
        let code = format!(
            r#"
            local taint
            local invoke = function()
                taint = debug.getstacktaint()
                {symbol}()
            end
            debug.setobjecttaint(invoke, 'Patch732Addon')
            local succeeded = pcall(invoke)
            return succeeded, taint
            "#
        );
        let (allowed, taint): (bool, String) =
            env.eval(&code).expect("probe insecure session action");
        assert_eq!(taint, "Patch732Addon", "probe must execute as addon code");
        let state = env.state().borrow();
        let unchanged = state.is_logged_in && !state.simulator_exit_requested;
        observations.insert(
            symbol.to_string(),
            serde_json::json!({"call_succeeded": allowed, "state_unchanged": unchanged,
                               "taint": taint, "ok": !allowed && unchanged}),
        );
    }
    if let Ok(path) = std::env::var("P732_PROTECTION_OUT") {
        std::fs::write(path, serde_json::to_string_pretty(&observations).unwrap())
            .expect("write protection observations");
    }
    assert!(
        observations.values().all(|row| row["ok"] == true),
        "insecure session actions must be blocked before mutation: {observations:?}"
    );
}

#[test]
fn patch_7_3_2_bare_insecure_session_actions_are_blocked() {
    let env = WowLuaEnv::new().expect("create environment");
    assert_insecure_session_actions_blocked(&env);
}

#[test]
fn patch_7_3_2_secure_session_actions_preserve_existing_transitions() {
    for symbol in ["Logout", "Quit"] {
        let env = WowLuaEnv::new().expect("create environment");
        assert_insecure_session_actions_blocked(&env);
        env.exec(&format!("assert(issecure()); {symbol}()"))
            .expect("secure session action");
        let state = env.state().borrow();
        assert_eq!(state.is_logged_in, symbol != "Logout");
        assert_eq!(state.simulator_exit_requested, symbol == "Quit");
    }
}

prefork_full_ui_case! {
fn patch_7_3_2_cached_insecure_session_actions_are_blocked(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    assert_insecure_session_actions_blocked(env);
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
