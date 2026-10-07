//! Current cached Game parent-key and retirement boundaries.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_2_0_cached_parent_keys_and_console_retirements(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(crate::patch_10_2_0_publication_fixes::PARENT_KEY_ASSERTIONS).unwrap();
        env.exec(crate::patch_10_2_0_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
