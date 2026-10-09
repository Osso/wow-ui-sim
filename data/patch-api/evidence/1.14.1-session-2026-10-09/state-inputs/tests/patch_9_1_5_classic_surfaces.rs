//! Classic namespace lookup remains unaffected by retail retirement.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_1_5_mists_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
for _, entry in ipairs({
    {'C_Commentator', 'SetBlacklistedAuras'},
    {'C_Commentator', 'SetBlacklistedCooldowns'},
    {'C_ItemUpgrade', 'GetItemLevelIncrement'},
    {'C_LFGList', 'GetActivityInfo'},
    {'C_LFGuildInfo', 'GetRecruitingGuildTabardInfo'},
    {'C_PlayerMentorship', 'GetMentorOptionalAchievementIDs'},
    {'C_Soulbinds', 'GetConduitChargesCapacity'},
    {'C_Soulbinds', 'GetConduitCharges'},
    {'C_Soulbinds', 'GetTotalConduitChargesPendingInSoulbind'},
    {'C_Soulbinds', 'GetTotalConduitChargesPending'},
    {'C_LFGList', 'GetCategoryInfo'},
}) do
    assert(type(_G[entry[1]][entry[2]]) == "function", entry[1] .. "." .. entry[2])
end
"#).unwrap();
}
