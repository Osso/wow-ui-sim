//! Retail retirement survives unmodified cached full-UI loading.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_9_0_5_cached_retirement(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(crate::patch_9_0_5_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
