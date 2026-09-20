//! Integration tests for `src/lua_api/globals/inventory_probes.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── IsInventoryItemLocked ─────────────────────────────────────────────────────

#[test]
fn is_inventory_item_locked_false_without_cursor_item() {
    let env = env();
    let b: bool = env.eval("return IsInventoryItemLocked(16)").unwrap();
    assert!(!b);
}

#[test]
fn is_inventory_item_locked_true_when_cursor_holds_that_slot() {
    let env = env();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        item_id: 1,
        stack_count: 1,
        origin: CursorItemOrigin::Equipped { slot: 16 },
    });
    let b: bool = env.eval("return IsInventoryItemLocked(16)").unwrap();
    assert!(b);
    // Different slot should still be false.
    let b: bool = env.eval("return IsInventoryItemLocked(5)").unwrap();
    assert!(!b);
}

#[test]
fn get_inventory_item_durability_returns_full_durability_for_equipped_items() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.equipped_items.insert(
            16,
            wow_ui_sim::lua_api::state::EquippedItem {
                item_id: 19019,
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
    }

    let (current, max): (i32, i32) = env
        .eval("return GetInventoryItemDurability(16)")
        .expect("GetInventoryItemDurability should be callable");
    assert_eq!((current, max), (100, 100));
}

#[test]
fn get_inventory_item_durability_returns_nil_for_empty_slots() {
    let env = env();
    let has_value: bool = env
        .eval("return GetInventoryItemDurability(99) ~= nil")
        .expect("GetInventoryItemDurability should be callable");
    assert!(!has_value);
}

// ── IsEquippableItem / IsConsumableItem ───────────────────────────────────────

#[test]
fn is_equippable_item_reads_state_set() {
    let env = env();
    env.state().borrow_mut().equippable_items.insert(19019);
    let b: bool = env.eval("return IsEquippableItem(19019)").unwrap();
    assert!(b);
    let b: bool = env.eval("return IsEquippableItem(6948)").unwrap();
    assert!(!b);
}

#[test]
fn is_consumable_item_reads_state_set() {
    let env = env();
    env.state().borrow_mut().consumable_items.insert(6948);
    let b: bool = env.eval("return IsConsumableItem(6948)").unwrap();
    assert!(b);
    let b: bool = env.eval("return IsConsumableItem(19019)").unwrap();
    assert!(!b);
}

// ── CanLootUnit ───────────────────────────────────────────────────────────────

#[test]
fn can_loot_unit_requires_dead_enemy_target() {
    let env = env();
    let b: bool = env.eval(r#"return CanLootUnit("target")"#).unwrap();
    assert!(!b, "with no target the probe is false");

    // Admin test helper: seed an enemy target at 0 HP.
    use wow_ui_sim::lua_api::state::TargetInfo;
    env.state().borrow_mut().current_target = Some(TargetInfo {
        unit_id: "target".into(),
        name: "Quilboar".into(),
        health: 0,
        health_max: 100,
        power: 0,
        power_max: 0,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_player: false,
        is_enemy: true,
        guid: "Creature-0-0".into(),
        level: 10,
        class_index: 1,
        classification: "normal".into(),
        creature_type: "Humanoid".into(),
        reaction: 2,
        interaction: Default::default(),
    });
    let b: bool = env.eval(r#"return CanLootUnit("target")"#).unwrap();
    assert!(b);
}

// ── CanMerchant ───────────────────────────────────────────────────────────────

#[test]
fn can_merchant_tracks_merchant_frame_flag() {
    let env = env();
    let b: bool = env.eval("return CanMerchant()").unwrap();
    assert!(!b);
    env.state().borrow_mut().merchant_frame_open = true;
    let b: bool = env.eval("return CanMerchant()").unwrap();
    assert!(b);
}

// ── CanInspect ────────────────────────────────────────────────────────────────

#[test]
fn can_inspect_true_for_player_and_party_members() {
    let env = env();
    env.state().borrow_mut().party_group_active = true;
    let b_player: bool = env.eval(r#"return CanInspect("player")"#).unwrap();
    let b_party: bool = env.eval(r#"return CanInspect("party1")"#).unwrap();
    assert!(b_player);
    assert!(b_party);
}

#[test]
fn can_inspect_false_for_empty_target() {
    let env = env();
    env.state().borrow_mut().current_target = None;
    let b: bool = env.eval(r#"return CanInspect("target")"#).unwrap();
    assert!(!b);
}

#[test]
fn notify_inspect_queues_inspect_ready_for_player_guid() {
    let env = env();
    env.exec(r#"NotifyInspect("player")"#).unwrap();

    let has_event = env.state().borrow().events.pending().iter().any(|event| {
        event.name == "INSPECT_READY"
            && event.args.iter().any(|arg| {
                matches!(
                    arg,
                    wow_ui_sim::event::EventArg::String(guid)
                        if guid == "Player-0000-00000001"
                )
            })
    });

    assert!(has_event);
}

#[test]
fn notify_inspect_ignores_uninspectable_target() {
    let env = env();
    env.state().borrow_mut().current_target = None;
    let before = env.state().borrow().events.pending().len();
    env.exec(r#"NotifyInspect("target")"#).unwrap();
    let after = env.state().borrow().events.pending().len();
    assert_eq!(after, before);
}
