//! Retail/Forever capability features over shared simulator state (api-gaps-9).

#[cfg(any(feature = "player-combat-query", feature = "raid-target-removal"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "player-combat-query")]
#[test]
fn player_is_in_combat_follows_supplied_combat_state() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(PlayerIsInCombat() == false)
        A_Admin.SetInCombat(true)
        assert(PlayerIsInCombat() == true)
        A_Admin.SetInCombat(false)
        assert(PlayerIsInCombat() == false)
        "#,
    )
    .unwrap();
    env.exec("A_Admin.SetInCombat(true)").unwrap();
    assert!(!other.eval::<bool>("return PlayerIsInCombat()").unwrap());
}

#[cfg(feature = "raid-target-removal")]
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
