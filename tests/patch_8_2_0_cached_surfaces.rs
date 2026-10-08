//! Behavior survives unmodified cached Blizzard full-UI loading.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_8_2_0_cached_volume_and_retirement(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(crate::patch_8_2_0_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
    env.exec(crate::patch_8_2_0_publication_fixes::VOLUME_ASSERTIONS).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
