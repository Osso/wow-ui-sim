//! Unused removed namespace members cannot be fabricated by repeated lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
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
    local namespace, member = _G[entry[1]], entry[2]
    assert(rawget(namespace, member) == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
end
assert(type(C_LFGList.GetActivityInfoTable) == "function")
assert(type(C_LFGList.GetLfgCategoryInfo) == "function")
"#;

#[test]
fn patch_9_1_5_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
