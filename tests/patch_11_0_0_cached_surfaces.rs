//! Unmodified cached Game load must preserve current retail retirements.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_11_0_0_cached_retirements(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(crate::patch_11_0_0_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
