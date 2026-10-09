//! Unused historical members stay absent; classic lookup stays reachable.
pub(crate) const REMOVED_MEMBERS: &[&str] = &[
    "C_ClubFinder.ReportPosting",
    "C_ClubFinder.ReturnCommunityApplicantList",
    "C_ClubFinder.ReturnGuildApplicantList",
    "C_ClubFinder.ReturnPendingCommunityApplicantList",
    "C_ClubFinder.ReturnPendingGuildApplicantList",
    "C_Commentator.GetElapsedMs",
    "C_Commentator.GetTrackedDefensiveCooldowns",
    "C_Commentator.GetTrackedOffensiveCooldowns",
    "C_Commentator.IsTrackedDefensiveCooldown",
    "C_Commentator.IsTrackedOffensiveCooldown",
    "C_PvP.GetMatchPVPStatIDs",
    "C_RecruitAFriend.CheckEmailEnabled",
    "C_RecruitAFriend.IsSendingEnabled",
    "C_RecruitAFriend.SendRecruit",
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
fn patch_8_2_5_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert_retired(&env);
}

#[test]
#[cfg(feature = "client-mists")]
fn patch_8_2_5_classic_members_stay_reachable() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    for symbol in REMOVED_MEMBERS {
        env.exec(&format!("assert(type({symbol}) == 'function', '{symbol}')"))
            .unwrap();
    }
}
