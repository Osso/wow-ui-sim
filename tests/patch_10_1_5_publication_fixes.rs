//! Bounded unused namespace retirement, not historical/native parity.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
assert(rawget(C_CampaignInfo, 'UsesNormalQuestIcons') == nil)
assert(C_CampaignInfo.UsesNormalQuestIcons == nil)
assert(C_CampaignInfo.UsesNormalQuestIcons == nil)
assert(type(C_CampaignInfo.GetCampaignInfo) == 'function')
"#;

#[test]
fn patch_10_1_5_unused_campaign_member_is_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
