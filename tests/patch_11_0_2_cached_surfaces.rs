//! Preserve the unmodified full cached Game load boundary.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_11_0_2_cached_retirements_and_item_binding(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(r#"
        for i = 1, 2 do
            assert(C_GameModeManager.GetFeatureSetting == nil, 'GetFeatureSetting')
            assert(C_GameModeManager.IsFeatureEnabled == nil, 'IsFeatureEnabled')
            assert(C_WeeklyRewards.GetWeeklyRewardTextureKit == nil, 'GetWeeklyRewardTextureKit lookup')
            assert(rawget(C_WeeklyRewards, 'GetWeeklyRewardTextureKit') == nil, 'GetWeeklyRewardTextureKit raw')
        end
        assert(type(C_WeeklyRewards.GetActivities) == 'function', 'GetActivities')
        assert(C_Item.IsItemBindToAccountUntilEquip('item:253451') == true, 'bound item')
        assert(C_Item.IsItemBindToAccountUntilEquip(19019) == false, 'unbound item')
        assert(GetCVarBool('loadDeprecationFallbacks'), 'loadDeprecationFallbacks')
    "#).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
