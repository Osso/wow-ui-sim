//! Bounded current-retail contracts, not historical/native parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_0_2_retired_members_do_not_regrow_on_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, entry in ipairs({
            {C_GameModeManager, 'GetFeatureSetting'},
            {C_GameModeManager, 'IsFeatureEnabled'},
            {C_WeeklyRewards, 'GetWeeklyRewardTextureKit'},
        }) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        assert(type(C_WeeklyRewards.GetActivities) == 'function')
        assert(type(C_GameRules.IsGameRuleActive) == 'function')
    "#).unwrap();
}

#[test]
fn patch_11_0_2_account_until_equipped_reads_intrinsic_item_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(type(rawget(C_Item, 'IsItemBindToAccountUntilEquip')) == 'function')
        -- Veilroot Fountain: generated ItemBind metadata is 9 (account until equipped).
        for _, item in ipairs({253451, '253451', 'item:253451',
            '|cnIQ1:|Hitem:253451::::::::80:70:::::::::|h[Veilroot Fountain]|h|r'}) do
            assert(C_Item.IsItemBindToAccountUntilEquip(item) == true)
        end
        assert(C_Item.IsItemBindToAccountUntilEquip(19019) == false)
        assert(C_Item.IsItemBindToAccountUntilEquip(99999999) == false)
        assert(C_Item.IsItemBindToAccountUntilEquip('not-an-item') == false)
        assert(C_Item.IsItemBindToAccount(253451) == true)
    "#).unwrap();
}
