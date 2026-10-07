//! Preserve the unmodified full cached Game load boundary.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_11_0_2_cached_retirements_and_item_binding(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(r#"
        for i = 1, 2 do
            assert(C_GameModeManager.GetFeatureSetting == nil)
            assert(C_GameModeManager.IsFeatureEnabled == nil)
            assert(C_WeeklyRewards.GetWeeklyRewardTextureKit == nil)
            assert(rawget(C_WeeklyRewards, 'GetWeeklyRewardTextureKit') == nil)
        end
        assert(type(C_WeeklyRewards.GetActivities) == 'function')
        assert(C_Item.IsItemBindToAccountUntilEquip('item:253451') == true)
        assert(C_Item.IsItemBindToAccountUntilEquip(19019) == false)
        assert(GetCVarBool('loadDeprecationFallbacks'))
    "#).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
