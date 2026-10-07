//! Full cached Game UI proof of the 10.2.6 retirement boundary.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_2_6_cached_retirements(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(crate::patch_10_2_6_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
        env.exec("assert(GetCVar('profanityFilter') ~= nil)").unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
