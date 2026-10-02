#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::{
    OutfitEntry, PendingTransmogCost, ViewedOutfitSlotInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn set_snapshot(env: &WowLuaEnv, cost: u64, modifier_flags: u32) {
    env.state().borrow_mut().pending_transmog_cost = Some(PendingTransmogCost {
        cost,
        modifier_flags,
    });
}

fn assert_pair(env: &WowLuaEnv, cost: u64, modifier_flags: u32) {
    env.exec(&format!(
        r#"
        local function inspect(...)
            assert(select('#', ...) == 2, 'expected exactly two returns')
            local cost, flags = ...
            assert(type(cost) == 'number' and type(flags) == 'number')
            assert(not issecretvalue(cost) and not issecretvalue(flags))
            assert(cost == {cost} and flags == {modifier_flags})
        end
        inspect(C_TransmogOutfitInfo.GetPendingTransmogCost())
        "#,
    ))
    .unwrap();
}

fn assert_absent(env: &WowLuaEnv) {
    env.exec("assert(select('#', C_TransmogOutfitInfo.GetPendingTransmogCost()) == 0)")
        .unwrap();
}

#[test]
fn pending_transmog_cost_default_has_zero_returns() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(env.state().borrow().pending_transmog_cost, None);
    assert_absent(&env);
}

#[test]
fn pending_transmog_cost_positive_cost_and_combined_flags_are_public_numbers() {
    let env = WowLuaEnv::new().unwrap();
    // Test data: 14 = 8 | 4 | 2; no native enum chronology or discount claim.
    set_snapshot(&env, 123_450, 14);
    assert_pair(&env, 123_450, 14);
}

#[test]
fn pending_transmog_cost_zero_values_never_mean_absence() {
    let env = WowLuaEnv::new().unwrap();
    for (cost, flags) in [(0, 0), (0, 14), (123_450, 0)] {
        set_snapshot(&env, cost, flags);
        assert_pair(&env, cost, flags);
    }
}

#[test]
fn pending_transmog_cost_preserves_full_u32_flags_and_unknown_bits() {
    let env = WowLuaEnv::new().unwrap();
    for flags in [0x8000_0000, u32::MAX] {
        set_snapshot(&env, 123_450, flags);
        assert_pair(&env, 123_450, flags);
    }
}

#[test]
fn pending_transmog_cost_repeated_reads_do_not_derive_or_mutate_owned_state() {
    let env = WowLuaEnv::new().unwrap();
    let slot = ViewedOutfitSlotInfo {
        transmog_id: 190_001,
        display_type: 1,
        is_transmogrified: true,
        has_pending: true,
        is_pending_collected: true,
        can_transmogrify: true,
        warning: 0,
        warning_text: String::new(),
        error: 0,
        error_text: String::new(),
        texture: Some(135_771),
        sheathe_category: 2,
    };
    {
        let mut state = env.state().borrow_mut();
        state.transmog_outfit_catalog.entries.push(OutfitEntry {
            outfit_id: 91,
            name: "Fixture outfit".into(),
            situation_categories: vec!["Raid".into()],
            icon: 135_771,
            is_event_outfit: false,
            is_disabled: true,
            player_facing_outfit_index: 7,
        });
        state.transmog_outfit_locks.insert(91);
        state.equipped_outfit_locked = true;
        state.outfit_situations_enabled = true;
        state.viewed_outfit_slots.insert((16, 1, 0), slot.clone());
    }
    env.exec(
        r#"
        C_TransmogOutfitInfo.ChangeToOutfit(91, false)
        C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(16, 2, 2)
        "#,
    )
    .unwrap();
    // Adjacent inputs do not create a cost snapshot.
    assert_absent(&env);
    set_snapshot(&env, 123_450, 14);
    for _ in 0..3 {
        assert_pair(&env, 123_450, 14);
    }
    env.exec(
        r#"
        assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 91)
        local pending = rawget(C_TransmogOutfitInfo, '__pendingSheatheCategories')
        assert(pending['16:2'] == 2)
        local count = 0
        for _ in pairs(pending) do count = count + 1 end
        assert(count == 1)
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(
        state.pending_transmog_cost,
        Some(PendingTransmogCost {
            cost: 123_450,
            modifier_flags: 14
        })
    );
    assert_eq!(state.viewed_outfit_slots.len(), 1);
    assert_eq!(state.viewed_outfit_slots.get(&(16, 1, 0)), Some(&slot));
    assert_eq!(state.transmog_outfit_locks.len(), 1);
    assert!(state.transmog_outfit_locks.contains(&91));
    assert!(state.equipped_outfit_locked && state.outfit_situations_enabled);
    assert_eq!(state.transmog_outfit_catalog.entries.len(), 1);
    let entry = &state.transmog_outfit_catalog.entries[0];
    assert_eq!(entry.outfit_id, 91);
    assert_eq!(entry.name, "Fixture outfit");
    assert_eq!(entry.situation_categories, ["Raid"]);
    assert_eq!(entry.icon, 135_771);
    assert!(!entry.is_event_outfit && entry.is_disabled);
    assert_eq!(entry.player_facing_outfit_index, 7);
}

#[test]
fn pending_transmog_cost_live_replacement_and_clear_have_no_sticky_result() {
    let env = WowLuaEnv::new().unwrap();
    set_snapshot(&env, 123_450, 14);
    assert_pair(&env, 123_450, 14);
    set_snapshot(&env, 765, 2);
    assert_pair(&env, 765, 2);
    env.state().borrow_mut().pending_transmog_cost = None;
    assert_absent(&env);
    assert_absent(&env);
    set_snapshot(&env, 0, 0);
    assert_pair(&env, 0, 0);
}

#[test]
fn pending_transmog_cost_environments_do_not_share_snapshots() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    set_snapshot(&first, 123_450, 14);
    assert_pair(&first, 123_450, 14);
    assert_absent(&second);
    set_snapshot(&second, 765, 2);
    assert_pair(&second, 765, 2);
    first.state().borrow_mut().pending_transmog_cost = None;
    assert_absent(&first);
    assert_pair(&second, 765, 2);
}

#[test]
fn pending_transmog_cost_safe_integer_upper_boundary_is_exact() {
    let env = WowLuaEnv::new().unwrap();
    set_snapshot(&env, 9_007_199_254_740_991, 14);
    assert_pair(&env, 9_007_199_254_740_991, 14);
}

#[test]
fn pending_transmog_cost_out_of_domain_errors_without_rounding_and_recovers() {
    let env = WowLuaEnv::new().unwrap();
    for cost in [9_007_199_254_740_992, 9_007_199_254_740_993, u64::MAX] {
        set_snapshot(&env, cost, 14);
        env.exec(
            r#"
            local ok, err = pcall(function()
                return C_TransmogOutfitInfo.GetPendingTransmogCost()
            end)
            assert(not ok and type(err) == 'string' and #err > 0)
            "#,
        )
        .unwrap();
        assert_eq!(
            env.state().borrow().pending_transmog_cost,
            Some(PendingTransmogCost {
                cost,
                modifier_flags: 14
            })
        );
        set_snapshot(&env, 123_450, 14);
        assert_pair(&env, 123_450, 14);
    }
    env.state().borrow_mut().pending_transmog_cost = None;
    assert_absent(&env);
}

#[test]
fn pending_transmog_cost_secure_and_tainted_public_reads_preserve_context() {
    let env = WowLuaEnv::new().unwrap();
    set_snapshot(&env, 123_450, 14);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            local secure = issecure()
            local function inspect(...)
                assert(select('#', ...) == 2)
                local cost, flags = ...
                assert(type(cost) == 'number' and type(flags) == 'number')
                assert(not issecretvalue(cost) and not issecretvalue(flags))
                assert(cost == 123450 and flags == 14)
            end
            inspect(C_TransmogOutfitInfo.GetPendingTransmogCost())
            assert(debug.getstacktaint() == before and issecure() == secure)
        end
        assert(issecure() and debug.getstacktaint() == nil)
        probe()
        local function addon()
            assert(not issecure() and debug.getstacktaint() == 'PendingCostFixture')
            probe()
        end
        debug.setobjecttaint(addon, 'PendingCostFixture')
        addon()
        assert(issecure() and debug.getstacktaint() == nil)
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().pending_transmog_cost,
        Some(PendingTransmogCost {
            cost: 123_450,
            modifier_flags: 14
        })
    );
}
