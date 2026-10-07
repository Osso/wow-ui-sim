//! Unused 9.2.0 namespace retirements must resist lazy lookup.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
for _, name in ipairs({'GetSpecialEventDetails', 'GetSpecialEventInfo'}) do
    assert(rawget(C_PvP, name) == nil, name)
    assert(C_PvP[name] == nil, name)
    assert(C_PvP[name] == nil, name)
end
assert(type(C_PvP.GetSpecialEventBrawlInfo) == 'function')
"#;

#[test]
fn patch_9_2_0_unused_pvp_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}

prefork_full_ui_case! {
    fn patch_9_2_0_cached_pvp_retirements(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(RETIREMENT_ASSERTIONS).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
