//! Full cached Game UI checks for the bounded 10.2.7 stable migration.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_2_7_cached_stable_migration(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(crate::patch_10_2_7_publication_fixes::RETIREMENT_ASSERTIONS).unwrap();
        env.state().borrow_mut().pet_stables_open = true;
        env.state().borrow_mut().events.drain();
        env.exec("C_StableInfo.ClosePetStables(); assert(not C_StableInfo.IsAtPetStable())").unwrap();
        let events = env.state().borrow_mut().events.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "PET_STABLE_CLOSED");
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
