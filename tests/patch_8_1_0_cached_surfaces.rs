//! Retail retirement survives unmodified cached full-UI startup.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_8_1_0_cached_retirement(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    crate::patch_8_1_0_publication_fixes::assert_retired(env);
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
