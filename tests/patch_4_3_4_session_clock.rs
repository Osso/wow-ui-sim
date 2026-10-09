#![cfg(feature = "client-retail")]

use std::time::{Duration, Instant};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn read_session_time(env: &WowLuaEnv) -> f64 {
    env.eval("return GetSessionTime()")
        .expect("GetSessionTime should expose the client-open clock")
}

#[test]
fn session_time_reports_client_elapsed_seconds_and_new_clients_start_fresh() {
    const ELAPSED_SECONDS: u64 = 7200;
    const SCHEDULING_MARGIN_SECONDS: f64 = 10.0;
    let env = WowLuaEnv::new().expect("create client environment");
    env.state().borrow_mut().start_time = Instant::now() - Duration::from_secs(ELAPSED_SECONDS);

    let elapsed = read_session_time(&env);
    assert!(elapsed >= ELAPSED_SECONDS as f64);
    assert!(elapsed < ELAPSED_SECONDS as f64 + SCHEDULING_MARGIN_SECONDS);

    let reopened_client = WowLuaEnv::new().expect("create independent client environment");
    let fresh_elapsed = read_session_time(&reopened_client);
    assert!(fresh_elapsed >= 0.0);
    assert!(fresh_elapsed < SCHEDULING_MARGIN_SECONDS);
}

#[test]
fn session_time_keeps_client_origin_across_login_and_character_screen_transitions() {
    const ELAPSED_SECONDS: u64 = 3600;
    const SCHEDULING_MARGIN_SECONDS: f64 = 10.0;
    let env = WowLuaEnv::new().expect("create client environment");
    env.state().borrow_mut().start_time = Instant::now() - Duration::from_secs(ELAPSED_SECONDS);
    let before = read_session_time(&env);

    {
        let mut state = env.state().borrow_mut();
        state.set_screen_kind(ScreenKind::Login);
    }
    let on_login_screen = read_session_time(&env);
    {
        let mut state = env.state().borrow_mut();
        state.set_screen_kind(ScreenKind::CharacterSelect);
    }
    let on_character_screen = read_session_time(&env);
    {
        let mut state = env.state().borrow_mut();
        state.set_screen_kind(ScreenKind::Game);
        state.is_logged_in = true;
    }
    let after_login = read_session_time(&env);

    assert!(before >= ELAPSED_SECONDS as f64);
    assert!(on_login_screen >= before);
    assert!(on_character_screen >= on_login_screen);
    assert!(after_login >= on_character_screen);
    assert!(after_login < ELAPSED_SECONDS as f64 + SCHEDULING_MARGIN_SECONDS);
}
