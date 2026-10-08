//! Unused historical members stay absent; classic lookup stays reachable.
pub(crate) const REMOVED_MEMBERS: &[&str] = &[
    "C_AreaPoiInfo.GetAreaPOITimeLeft",
    "C_Calendar.GetDate",
    "C_ChatInfo.ReportPlayer",
    "C_Club.AddClubStreamToChatWindow",
    "C_DateAndTime.GetDateFromEpoch",
    "C_DateAndTime.GetTodaysDate",
    "C_DateAndTime.GetYesterdaysDate",
    "C_PvP.GetBrawlInfo",
    "C_ReportSystem.ReportPlayer",
    "C_Social.GetLastScreenshot",
    "C_Social.GetNumCharactersPerMedia",
    "C_Social.GetScreenshotByIndex",
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
fn patch_8_1_5_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert_retired(&env);
}

#[test]
#[cfg(feature = "client-mists")]
fn patch_8_1_5_classic_members_stay_reachable() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    for symbol in REMOVED_MEMBERS {
        env.exec(&format!("assert(type({symbol}) == 'function', '{symbol}')"))
            .unwrap();
    }
}
