//! Cached Blizzard secure-action dispatch exercises the modeled global after full preload.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_11_0_7_cached_clear_all_raid_targets(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(
        r#"
        A_Admin.SetPartySize(2)
        A_Admin.SetTarget('Enemy', 60, 1, true)
        SetRaidTarget('player', 8)
        SetRaidTarget('party1', 7)
        SetRaidTarget('target', 6)
        local button = CreateFrame('Button')
        button:SetAttribute('type', 'raidtarget')
        button:SetAttribute('unit', 'target')
        button:SetAttribute('useOnKeyDown', false)
        button:SetAttribute('action', 'clear-all')
        assert(SecureActionButton_OnClick(button, 'LeftButton', false, false, true))
        assert(GetRaidTargetIndex('player') == nil)
        assert(GetRaidTargetIndex('party1') == nil)
        assert(GetRaidTargetIndex('target') == nil)
        assert(type(C_ArrowCalloutManager.AcknowledgeCallout) == 'function')
        assert(C_ArrowCalloutManager.HideWorldLootObjectCallout == nil)
        assert(C_WorldLootObject.GetCurrentWorldLootObjectSwapInventoryType == nil)
        "#,
    ).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
}
