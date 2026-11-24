//! Selected raid-manager load continuation after real arena aura cleanup.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

#[path = "stock_residual_roots/support.rs"]
mod support;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use rilua::Val;
use support::{assert_no_consumer_errors, assert_original_source};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

const CONSUMER_ERRORS: &[&str] = &[
    "ClearBlockedAuras",
    "CompactRaidFrameManager",
    "CompactRaidFrameContainer",
    "CompactArenaFrame",
    "dividerVerticalPool",
    "dividerHorizontalPool",
];

fn fixture_aura(id: i32) -> AuraInfo {
    AuraInfo {
        name: format!("Stock residual aura {id}"),
        spell_id: 99000 + id,
        icon: wow_ui_sim::lua_api::state::AuraIcon::FileId(134973),
        duration: 30.0,
        expiration_time: 45.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn seed_blockable_auras(env: &WowLuaEnv) {
    env.exec("A_Admin.ClearLossOfControl('player'); A_Admin.ClearLossOfControl('party1')")
        .expect("supplied no-loss-of-control fixture for both selected units");
    let mut state = env.state().borrow_mut();
    state.party_group_active = true;
    state.party_leader_index = None;
    state.player.buffs = vec![fixture_aura(101)];
    let party = state
        .party_members
        .first_mut()
        .expect("seeded party member");
    party.buffs = vec![fixture_aura(201), fixture_aura(202)];
    party.debuffs.clear();
}

#[test]
fn original_raid_manager_initializes_divider_pools_after_arena_cleanup() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    seed_blockable_auras(&env);
    // Load the real RaidFrame -> UnitFrame closure, then selected manager XML.
    // Do not clear the initial errors or supply missing pools after a failed load.
    let loaded =
        load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_CompactRaidFrames"], &[]);
    for addon in [
        "Blizzard_RaidFrame",
        "Blizzard_UnitFrame",
        "Blizzard_CompactRaidFrames",
    ] {
        assert!(loaded.iter().any(|name| name == addon), "missing {addon}");
    }
    assert_original_source(
        &env,
        "CompactRaidFrameManager_OnLoad",
        "Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua",
    );
    assert_original_source(
        &env,
        "CompactRaidFrameContainerMixin.TryUpdate",
        "Blizzard_CompactRaidFrames/Blizzard_CompactRaidFrameContainer.lua",
    );
    assert_original_source(
        &env,
        "ArenaUnitFrameDebuffMixin.Update",
        "Blizzard_UnitFrame/Mainline/CompactArenaFrame.lua",
    );
    assert_no_consumer_errors(&env, "initial raid-manager OnLoad", CONSUMER_ERRORS);
    env.exec(
        r#"
        local manager = CompactRaidFrameManager
        assert(manager.dividerVerticalPool and manager.dividerHorizontalPool,
            'initial unchanged manager OnLoad must reach both pool assignments')
        assert(CompactArenaFrame and #CompactArenaFrame.memberUnitFrames > 0)
        StockResidualDebuff = CompactArenaFrame.memberUnitFrames[1].DebuffFrame
        assert(StockResidualDebuff and StockResidualDebuff:GetParent() ==
            CompactArenaFrame.memberUnitFrames[1], 'must use the original arena member child')

        -- Replay the unchanged OnLoad only after proving its initial completion.
        CompactRaidFrameManager_OnLoad(manager)
        StockResidualVerticalPool = manager.dividerVerticalPool
        StockResidualHorizontalPool = manager.dividerHorizontalPool
        assert(manager:IsEventRegistered('GROUP_ROSTER_UPDATE'))
        assert(manager:IsEventRegistered('PLAYER_ENTERING_WORLD'))
        assert(IsInGroup() and UnitIsGroupLeader('player') and not IsInRaid())

        C_UnitAuras.AddBlockedAura('player', 101)
        for pass = 1, 2 do
            C_UnitAuras.AddBlockedAura('party1', 201)
            C_UnitAuras.AddBlockedAura('party1', 202)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil)
            assert(C_LossOfControl.GetActiveLossOfControlDataCountByUnit('party1') == 0)
            StockResidualDebuff:SetUnit('party1')
            StockResidualDebuff:Update()
            StockResidualDebuff:Update() -- repeat-clear must preserve visible aura rows
            assert(StockResidualDebuff.auraData == nil)
            assert(not StockResidualDebuff:IsShown())
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL').auraInstanceID == 201)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 2, 'HELPFUL').auraInstanceID == 202)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 3, 'HELPFUL') == nil)
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil,
                'selected-unit cleanup must not clear another unit block')
        end
        -- Reverse selection with both units blocked to reject hardcoded party1 cleanup.
        for pass = 1, 2 do
            C_UnitAuras.AddBlockedAura('party1', 201)
            C_UnitAuras.AddBlockedAura('party1', 202)
            C_UnitAuras.AddBlockedAura('player', 101)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil)
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil)
            assert(C_LossOfControl.GetActiveLossOfControlDataCountByUnit('player') == 0)
            StockResidualDebuff:SetUnit('player')
            StockResidualDebuff:Update()
            StockResidualDebuff:Update()
            assert(StockResidualDebuff.auraData == nil)
            assert(not StockResidualDebuff:IsShown())
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 101)
            assert(C_UnitAuras.GetAuraDataByIndex('player', 2, 'HELPFUL') == nil)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil,
                'clearing player must preserve populated party blocks')
        end

        -- Explicit SetUnit(nil)/Update acceptance remains a native-evidence gap.
        -- Do not turn this assigned-unit selector into an invented nil policy.
        "#,
    )
    .expect("original manager and assigned-unit arena cleanup");
    assert_no_consumer_errors(&env, "manager and arena cleanup", CONSUMER_ERRORS);

    // Preserve roster notification before the initial world-enter notification.
    env.fire_event("GROUP_ROSTER_UPDATE")
        .expect("real startup roster notification");
    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[Val::Bool(true), Val::Bool(false)],
    )
    .expect("real initial world-enter notification");
    assert_no_consumer_errors(&env, "roster then world-enter", CONSUMER_ERRORS);
    env.exec(
        r#"
        CompactRaidFrameManager_UpdateOptionsFlowContainer()
        assert(StockResidualVerticalPool:GetNumActive() > 0)
        assert(StockResidualHorizontalPool:GetNumActive() > 0)
        StockResidualDividers = {}
        for divider in StockResidualVerticalPool:EnumerateActive() do
            assert(divider:IsShown())
            table.insert(StockResidualDividers, divider)
        end
        for divider in StockResidualHorizontalPool:EnumerateActive() do
            assert(divider:IsShown())
            table.insert(StockResidualDividers, divider)
        end
        "#,
    )
    .expect("original options flow must acquire and display both divider types");

    env.state().borrow_mut().party_group_active = false;
    env.fire_event("GROUP_ROSTER_UPDATE")
        .expect("real leave-group roster notification");
    env.exec(
        r#"
        CompactRaidFrameManager_UpdateOptionsFlowContainer()
        assert(StockResidualVerticalPool:GetNumActive() == 0)
        assert(StockResidualHorizontalPool:GetNumActive() == 0)
        for _, divider in ipairs(StockResidualDividers) do
            assert(not divider:IsShown(), 'original ReleaseAll must hide obsolete dividers')
        end
        "#,
    )
    .expect("original options flow must release obsolete dividers after leaving group");
    env.state().borrow_mut().party_group_active = true;
    env.fire_event("GROUP_ROSTER_UPDATE")
        .expect("real rejoin roster notification");
    env.exec(
        r#"
        CompactRaidFrameManager_UpdateOptionsFlowContainer()
        assert(CompactRaidFrameManager.dividerVerticalPool == StockResidualVerticalPool)
        assert(CompactRaidFrameManager.dividerHorizontalPool == StockResidualHorizontalPool)
        assert(StockResidualVerticalPool:GetNumActive() > 0)
        assert(StockResidualHorizontalPool:GetNumActive() > 0)
        "#,
    )
    .expect("original pools must remain usable after release and reacquisition");
    assert_no_consumer_errors(
        &env,
        "divider acquisition/release/reacquisition",
        CONSUMER_ERRORS,
    );
    let state = env.state().borrow();
    assert_eq!(state.player.buffs[0].aura_instance_id, 101);
    assert_eq!(state.party_members[0].buffs.len(), 2);
}

#[test]
fn original_raid_manager_projects_supplied_ping_availability() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_CompactRaidFrames"], &[]);
    env.exec(
        r#"
        local dropdown = CompactRaidFrameManager.displayFrame.RestrictPingsDropdown
        local menu = dropdown:GetMenuDescription()
        assert(not menu or not menu:HasElements())
        "#,
    )
    .expect("disabled supplied fixture does not initialize the ping menu");
    for enabled in [true, false] {
        env.state().borrow_mut().forever_startup.ping_enabled = enabled;
        env.exec(
            r#"
            CompactRaidFrameManager_OnLoad(CompactRaidFrameManager)
            local dropdown = CompactRaidFrameManager.displayFrame.RestrictPingsDropdown
            dropdown:GenerateMenu()
            assert(dropdown:GetMenuDescription():HasElements())
            "#,
        )
        .expect("original OnLoad initializes and retains the enabled ping menu");
        assert_eq!(
            env.eval::<bool>("return C_Ping.IsPingSystemEnabled()")
                .unwrap(),
            enabled
        );
    }
}
