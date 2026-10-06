//! Bounded simulator state/publication contracts, not native 11.2.0 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_2_0_player_combat_reads_live_state() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    assert!(!env.eval::<bool>("return PlayerIsInCombat()").unwrap());
    env.state().borrow_mut().player.in_combat = true;
    assert!(env.eval::<bool>("return PlayerIsInCombat()").unwrap());
    assert!(!other.eval::<bool>("return PlayerIsInCombat()").unwrap());
    env.state().borrow_mut().player.in_combat = false;
    assert!(!env.eval::<bool>("return PlayerIsInCombat()").unwrap());
}

#[test]
fn patch_11_2_0_font_gradient_reads_set_and_clear_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local fs = CreateFrame('Frame'):CreateFontString()
        local other = CreateFrame('Frame'):CreateFontString()
        fs:SetAlphaGradient(12, 48)
        local start, length = fs:GetAlphaGradient()
        assert(start == 12 and length == 48)
        fs:SetAlphaGradient(6, 24)
        start, length = fs:GetAlphaGradient()
        assert(start == 6 and length == 24)
        start, length = other:GetAlphaGradient()
        assert(start == 0 and length == 0)
        fs:ClearAlphaGradient()
        start, length = fs:GetAlphaGradient()
        assert(start == 0 and length == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_2_0_removed_members_stay_absent_on_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for i = 1, 2 do
            assert(C_Bank.FetchNextPurchasableBankTabCost == nil)
            assert(C_Container.SortReagentBankBags == nil)
            assert(C_TooltipInfo.GetVoidDepositItem == nil)
            assert(C_TooltipInfo.GetVoidItem == nil)
            assert(C_TooltipInfo.GetVoidWithdrawalItem == nil)
        end
        assert(type(C_Container.GetContainerNumSlots) == 'function')
        assert(type(C_TooltipInfo.GetItemByID) == 'function')
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_2_0_browser_retains_home_not_removed_navigation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local browser = CreateFrame('Browser')
        assert(browser.NavigateTo == nil)
        assert(type(browser.NavigateHome) == 'function')
        assert(select('#', browser:NavigateHome()) == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_2_0_fog_cvars_publish_documented_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({
            'volumeFogDisableLightScattering', 'volumeFogDisableNoise',
            'volumeFogDisableShadows', 'volumeFogUse16BitTexture',
        }) do
            assert(C_CVar.GetCVarDefault(name) == '0', name)
            assert(C_CVar.GetCVarDefault(string.upper(name)) == '0', name)
        end
        "#,
    )
    .unwrap();
}
