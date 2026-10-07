//! Retirement survives unmodified cached Game loading.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_1_0_cached_namespace_retirement(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(crate::patch_10_1_0_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
