//! Tests for A_Admin spec and talent API.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetSpec / GetSpecialization
// ============================================================================

#[test]
fn test_set_spec_readable_via_get_specialization() {
    let env = env();
    let spec: i32 = env
        .eval(
            r#"
            A_Admin.SetSpec(3)
            return GetSpecialization()
            "#,
        )
        .unwrap();
    assert_eq!(spec, 3);
}

#[test]
fn test_inspect_specialization_returns_active_spec_id() {
    let env = env();
    let spec_id: i32 = env
        .eval(
            r#"
            A_Admin.SetSpec(3)
            return GetInspectSpecialization("player")
            "#,
        )
        .unwrap();
    assert_eq!(spec_id, 70);
}

#[test]
fn test_specialization_role_by_id_returns_role_token() {
    let env = env();
    let role: String = env.eval("return GetSpecializationRoleByID(70)").unwrap();
    assert_eq!(role, "DAMAGER");
}

#[test]
fn test_set_spec_one() {
    let env = env();
    let spec: i32 = env
        .eval(
            r#"
            A_Admin.SetSpec(1)
            return GetSpecialization()
            "#,
        )
        .unwrap();
    assert_eq!(spec, 1);
}

#[test]
fn test_set_spec_two() {
    let env = env();
    let spec: i32 = env
        .eval(
            r#"
            A_Admin.SetSpec(2)
            return GetSpecialization()
            "#,
        )
        .unwrap();
    assert_eq!(spec, 2);
}

#[test]
fn test_set_spec_overrides_previous() {
    let env = env();
    let spec: i32 = env
        .eval(
            r#"
            A_Admin.SetSpec(1)
            A_Admin.SetSpec(4)
            return GetSpecialization()
            "#,
        )
        .unwrap();
    assert_eq!(spec, 4);
}

#[test]
fn test_get_num_spec_groups_reports_active_player_group() {
    let env = env();
    let (num_groups, active_group): (i32, i32) = env
        .eval("return GetNumSpecGroups(false), C_SpecializationInfo.GetActiveSpecGroup(false)")
        .unwrap();

    assert_eq!(num_groups, 1);
    assert_eq!(active_group, 1);
}

#[test]
fn test_get_specialization_mastery_spells_returns_iterable_table() {
    let env = env();
    let (is_table, iterated): (bool, bool) = env
        .eval(
            r#"
            local masterySpells = C_SpecializationInfo.GetSpecializationMasterySpells(2)
            for _ in ipairs(masterySpells) do end
            return type(masterySpells) == "table", true
            "#,
        )
        .unwrap();

    assert!(is_table);
    assert!(iterated);
}

// ============================================================================
// SetTalentRank
// ============================================================================

// Node 100734 is a regular (type 0) node in the Paladin class talent tree (tree 994).
// It has no spec-set condition, so it is visible regardless of active spec.
// configID 1 is always valid (returned by C_Traits.GetConfigIDBySystemID).
const PALADIN_NODE_ID: i32 = 100734;

#[test]
fn test_set_talent_rank_does_not_error() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            A_Admin.SetTalentRank(100734, 2)
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn test_set_talent_rank_reflected_in_get_node_info() {
    let env = env();
    let ranks: i32 = env
        .eval(&format!(
            r#"
            A_Admin.SetTalentRank({node}, 2)
            local info = C_Traits.GetNodeInfo(1, {node})
            return info.ranksPurchased
            "#,
            node = PALADIN_NODE_ID,
        ))
        .unwrap();
    assert_eq!(ranks, 2);
}

#[test]
fn test_set_talent_rank_zero_clears_ranks() {
    let env = env();
    let ranks: i32 = env
        .eval(&format!(
            r#"
            A_Admin.SetTalentRank({node}, 3)
            A_Admin.SetTalentRank({node}, 0)
            local info = C_Traits.GetNodeInfo(1, {node})
            return info.ranksPurchased
            "#,
            node = PALADIN_NODE_ID,
        ))
        .unwrap();
    assert_eq!(ranks, 0);
}

#[test]
fn test_set_talent_rank_updates_current_rank() {
    let env = env();
    let (current, active): (i32, i32) = env
        .eval(&format!(
            r#"
            A_Admin.SetTalentRank({node}, 1)
            local info = C_Traits.GetNodeInfo(1, {node})
            return info.currentRank, info.activeRank
            "#,
            node = PALADIN_NODE_ID,
        ))
        .unwrap();
    assert_eq!(current, 1);
    assert_eq!(active, 1);
}

// ============================================================================
// SetTalentSelection
// ============================================================================

#[test]
fn test_set_talent_selection_does_not_error() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            A_Admin.SetTalentSelection(100754, 122583)
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

// ============================================================================
// ResetTalents
// ============================================================================

#[test]
fn test_reset_talents_clears_node_ranks() {
    let env = env();
    let ranks: i32 = env
        .eval(&format!(
            r#"
            A_Admin.SetTalentRank({node}, 3)
            A_Admin.ResetTalents()
            local info = C_Traits.GetNodeInfo(1, {node})
            return info.ranksPurchased
            "#,
            node = PALADIN_NODE_ID,
        ))
        .unwrap();
    assert_eq!(ranks, 0, "ResetTalents should clear all node ranks to 0");
}

#[test]
fn test_reset_talents_does_not_error() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            A_Admin.SetTalentRank(100734, 2)
            A_Admin.SetTalentRank(100754, 1)
            A_Admin.ResetTalents()
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn test_reset_talents_allows_re_setting_ranks() {
    let env = env();
    let ranks: i32 = env
        .eval(&format!(
            r#"
            A_Admin.SetTalentRank({node}, 3)
            A_Admin.ResetTalents()
            A_Admin.SetTalentRank({node}, 1)
            local info = C_Traits.GetNodeInfo(1, {node})
            return info.ranksPurchased
            "#,
            node = PALADIN_NODE_ID,
        ))
        .unwrap();
    assert_eq!(ranks, 1);
}

#[test]
fn test_trait_config_mapping_tracks_active_loadout() {
    let env = env();
    let (active_before, by_tree_before, by_system_before, active_after, by_tree_after, by_system_after):
        (i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local activeBefore = C_ClassTalents.GetActiveConfigID()
            local treeID = C_Traits.GetConfigInfo(activeBefore).treeIDs[1]
            local byTreeBefore = C_Traits.GetConfigIDByTreeID(treeID)
            local bySystemBefore = C_Traits.GetConfigIDBySystemID(1)

            local configs = C_ClassTalents.GetConfigIDsBySpecID(66)
            assert(C_ClassTalents.LoadConfig(configs[2], true) == Enum.LoadConfigResult.Ready)

            local activeAfter = C_ClassTalents.GetActiveConfigID()
            local switchedTreeID = C_Traits.GetConfigInfo(activeAfter).treeIDs[1]
            local byTreeAfter = C_Traits.GetConfigIDByTreeID(switchedTreeID)
            local bySystemAfter = C_Traits.GetConfigIDBySystemID(1)

            return activeBefore, byTreeBefore, bySystemBefore, activeAfter, byTreeAfter, bySystemAfter
            "#,
        )
        .unwrap();

    assert_eq!(by_tree_before, active_before);
    assert_eq!(by_system_before, active_before);
    assert_ne!(active_after, active_before);
    assert_eq!(by_tree_after, active_after);
    assert_eq!(by_system_after, active_after);
}

#[test]
fn test_can_purchase_rank_tracks_live_node_gating() {
    let env = env();
    let (node_id, entry_id, can_before, can_after): (i32, i32, bool, bool) = env
        .eval(
            r#"
            local configID = C_ClassTalents.GetActiveConfigID()
            local treeID = C_Traits.GetConfigInfo(configID).treeIDs[1]

            for _, nodeID in ipairs(C_Traits.GetTreeNodes(treeID)) do
                local nodeInfo = C_Traits.GetNodeInfo(configID, nodeID)
                if nodeInfo and nodeInfo.canPurchaseRank and nodeInfo.entryIDs and nodeInfo.entryIDs[1] then
                    local entryID = nodeInfo.entryIDs[1]
                    local canBefore = C_Traits.CanPurchaseRank(configID, nodeID, entryID)
                    for _ = 1, nodeInfo.totalMaxRanks do
                        assert(C_Traits.PurchaseRank(configID, nodeID))
                    end
                    local canAfter = C_Traits.CanPurchaseRank(configID, nodeID, entryID)
                    return nodeID, entryID, canBefore, canAfter
                end
            end

            error("expected at least one purchasable node in the active config")
            "#,
        )
        .unwrap();

    assert!(node_id > 0, "expected a live purchasable node");
    assert!(entry_id > 0, "expected a live purchasable entry");
    assert!(
        can_before,
        "purchasable node should report true before spending"
    );
    assert!(!can_after, "maxed node should stop reporting purchasable");
}

#[test]
fn test_staged_changes_expose_purchases_and_costs() {
    let env = env();
    let (purchase_node, cost_id, cost_amount): (i32, i32, i32) = env
        .eval(
            r#"
            local configID = C_ClassTalents.GetActiveConfigID()
            local treeID = C_Traits.GetConfigInfo(configID).treeIDs[1]

            local purchaseNodeID = nil
            for _, nodeID in ipairs(C_Traits.GetTreeNodes(treeID)) do
                local nodeInfo = C_Traits.GetNodeInfo(configID, nodeID)
                if nodeInfo and nodeInfo.canPurchaseRank and nodeInfo.entryIDs and nodeInfo.entryIDs[1] then
                    purchaseNodeID = nodeID
                    break
                end
            end

            assert(purchaseNodeID, "expected a staged purchase candidate")
            assert(C_Traits.PurchaseRank(configID, purchaseNodeID))
            assert(C_Traits.ConfigHasStagedChanges(configID), "staged purchases should become visible")

            local purchases = C_Traits.GetStagedChanges(configID)
            assert(purchases and tContains(purchases, purchaseNodeID), "purchase node should be listed")

            local costs = C_Traits.GetStagedChangesCost(configID)
            assert(costs and costs[1] and costs[1].ID and costs[1].amount, "staged purchase costs should include trait currency rows")

            return purchaseNodeID, costs[1].ID, costs[1].amount
            "#,
        )
        .unwrap();

    assert!(purchase_node > 0);
    assert!(cost_id > 0, "expected a staged trait currency cost id");
    assert!(
        cost_amount > 0,
        "expected staged purchase costs to consume talent currency"
    );
}

#[test]
fn test_staged_changes_expose_refunds() {
    let env = env();
    let refund_node: i32 = env
        .eval(
            r#"
            local configID = C_ClassTalents.GetActiveConfigID()
            local treeID = C_Traits.GetConfigInfo(configID).treeIDs[1]

            for _, nodeID in ipairs(C_Traits.GetTreeNodes(treeID)) do
                local nodeInfo = C_Traits.GetNodeInfo(configID, nodeID)
                if nodeInfo and nodeInfo.canRefundRank then
                    assert(C_Traits.RefundRank(configID, nodeID))
                    assert(C_Traits.ConfigHasStagedChanges(configID), "staged refunds should become visible")

                    local _, refunds = C_Traits.GetStagedChanges(configID)
                    assert(refunds and tContains(refunds, nodeID), "refund node should be listed")
                    return nodeID
                end
            end

            error("expected a staged refund candidate")
            "#,
        )
        .unwrap();

    assert!(refund_node > 0);
}

#[test]
fn test_staged_changes_expose_selection_swaps() {
    let env = env();
    let swap_node: i32 = env
        .eval(
            r#"
            local configID = C_ClassTalents.GetActiveConfigID()
            local treeID = C_Traits.GetConfigInfo(configID).treeIDs[1]

            for _, nodeID in ipairs(C_Traits.GetTreeNodes(treeID)) do
                local nodeInfo = C_Traits.GetNodeInfo(configID, nodeID)
                if nodeInfo and nodeInfo.ranksPurchased and nodeInfo.ranksPurchased > 0 and nodeInfo.entryIDs and #nodeInfo.entryIDs > 1 and nodeInfo.activeEntry and nodeInfo.activeEntry.entryID then
                    for _, entryID in ipairs(nodeInfo.entryIDs) do
                        if entryID ~= nodeInfo.activeEntry.entryID then
                            assert(C_Traits.SetSelection(configID, nodeID, entryID))
                            assert(C_Traits.ConfigHasStagedChanges(configID), "staged selection swaps should become visible")

                            local _, _, swaps = C_Traits.GetStagedChanges(configID)
                            assert(swaps and tContains(swaps, nodeID), "selection swap node should be listed")
                            return nodeID
                        end
                    end
                end
            end

            error("expected a staged selection swap candidate")
            "#,
        )
        .unwrap();

    assert!(swap_node > 0);
}

// ============================================================================
// C_SpecializationInfo.SetSpecialization — cast-based flow
// ============================================================================

#[test]
fn c_spec_set_specialization_starts_cast_and_defers_active_index() {
    let env = env();
    env.state().borrow_mut().player.active_spec_index = 1;

    let ok: bool = env
        .eval("return C_SpecializationInfo.SetSpecialization(2)")
        .unwrap();
    assert!(ok, "SetSpecialization should return true");

    let state = env.state().borrow();
    assert_eq!(
        state.player.pending_spec_change,
        Some(2),
        "pending_spec_change should be queued"
    );
    assert_eq!(
        state.player.active_spec_index, 1,
        "active_spec_index must NOT change until cast completes \
         (otherwise the UI's grey overlay never clears)"
    );
    assert!(
        state.casting.is_some(),
        "a cast must be in flight so the UI's PLAYER_SPECIALIZATION_CHANGED \
         dismissal path runs"
    );
}

#[test]
fn c_spec_set_specialization_same_spec_is_noop() {
    let env = env();
    env.state().borrow_mut().player.active_spec_index = 2;

    let ok: bool = env
        .eval("return C_SpecializationInfo.SetSpecialization(2)")
        .unwrap();
    assert!(ok);

    let state = env.state().borrow();
    assert_eq!(state.player.pending_spec_change, None);
    assert!(state.casting.is_none());
}
