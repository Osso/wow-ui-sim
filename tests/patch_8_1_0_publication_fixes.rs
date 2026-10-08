//! Unused historical members stay absent; classic lookup stays reachable.
pub(crate) const REMOVED_MEMBERS: &[&str] = &[
    "C_Map.GetBountySetIDForMap",
    "C_Calendar.EventGetClubID",
    "C_Calendar.EventSetClubID",
];

#[cfg(feature = "client-retail")]
pub(crate) fn assert_retired(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    for symbol in REMOVED_MEMBERS {
        env.exec(&format!(
            r#"
            local namespace, member = string.match("{symbol}", "^([^.]+)%.(.+)$")
            local api = _G[namespace]
            assert(type(api) == "table", namespace)
            assert(rawget(api, member) == nil, "{symbol} raw")
            assert(api[member] == nil, "{symbol} lookup")
            assert(api[member] == nil, "{symbol} repeat")
            "#
        ))
        .unwrap();
    }
}

#[test]
#[cfg(feature = "client-retail")]
fn patch_8_1_0_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert_retired(&env);
}

#[test]
#[cfg(feature = "client-mists")]
fn patch_8_1_0_classic_members_stay_reachable() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    for symbol in REMOVED_MEMBERS {
        env.exec(&format!("assert(type({symbol}) == 'function', '{symbol}')"))
            .unwrap();
    }
}
