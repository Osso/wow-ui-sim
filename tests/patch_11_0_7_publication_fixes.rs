//! Bounded simulator state and absence contracts, not native 11.0.7 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_0_7_retired_members_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, entry in ipairs({
            {C_ArrowCalloutManager, 'HideWorldLootObjectCallout'},
            {C_ArrowCalloutManager, 'SetWorldLootObjectCalloutFromGUID'},
            {C_ArrowCalloutManager, 'SwapWorldLootObjectCallout'},
            {C_WorldLootObject, 'GetCurrentWorldLootObjectSwapInventoryType'},
        }) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        assert(type(C_ArrowCalloutManager.AcknowledgeCallout) == 'function')
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_0_7_remove_raid_targets_clears_all_guid_markers_before_event() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetPartySize(2)
        A_Admin.SetTarget('Enemy', 60, 1, true)
        SetRaidTarget('player', 8)
        SetRaidTarget('party1', 7)
        SetRaidTarget('party2', 6)
        SetRaidTarget('target', 5)
        local updates = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('RAID_TARGET_UPDATE')
        frame:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'RAID_TARGET_UPDATE')
            assert(select('#', ...) == 0)
            for _, unit in ipairs({'player', 'self', 'party1', 'party2', 'target'}) do
                assert(GetRaidTargetIndex(unit) == nil, unit)
            end
            updates = updates + 1
        end)
        assert(select('#', RemoveRaidTargets()) == 0)
        assert(updates == 1)
        assert(select('#', RemoveRaidTargets()) == 0)
        assert(updates == 2)
        frame:UnregisterAllEvents()
        SetRaidTarget('party1', 8)
        assert(GetRaidTargetIndex('party1') == 8)
        assert(GetRaidTargetIndex('player') == nil)
        "#,
    )
    .unwrap();
}
