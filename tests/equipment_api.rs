//! Tests for equipment: equip/unequip items, query inventory slots.

use crate::common::inventory_slot_info_chunk;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_default_gear_head_slot() {
    let env = env();
    let id: i64 = env.eval("return GetInventoryItemID('player', 1)").unwrap();
    assert_eq!(id, 211993);
}

#[test]
fn test_empty_offhand_slot_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetInventoryItemID('player', 17) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_inventory_item_link_contains_name() {
    let env = env();
    let link: String = env
        .eval("return GetInventoryItemLink('player', 1)")
        .unwrap();
    assert!(link.contains("Entombed Seraph"), "link={link}");
    assert!(link.contains("|Hitem:211993"), "link={link}");
}

#[test]
fn test_inventory_item_link_nil_for_empty_slot() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetInventoryItemLink('player', 17) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_inventory_item_texture_returns_value() {
    let env = env();
    let tex: i64 = env
        .eval("return GetInventoryItemTexture('player', 1)")
        .unwrap();
    assert!(tex > 0);
}

#[test]
fn test_inventory_item_texture_nil_for_empty_slot() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetInventoryItemTexture('player', 17) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_inventory_item_texture_uses_ring_fallback_icon_for_missing_item_icon() {
    let env = env();
    env.exec("A_Admin.EquipItem(11, 999999)").unwrap();
    let tex: i64 = env
        .eval("return GetInventoryItemTexture('player', 11)")
        .unwrap();
    assert_eq!(tex, 133358);
}

#[test]
fn test_bag_slot_texture() {
    let env = env();
    let tex: String = env
        .eval("return tostring(GetInventoryItemTexture('player', 20))")
        .unwrap();
    assert!(tex.contains("INV_Misc_Bag_08"));
}

#[test]
fn test_profession_slot_texture_prefers_equipped_item() {
    let env = env();
    env.exec("A_Admin.EquipItem(20, 238017)").unwrap();
    let item_id: i64 = env.eval("return GetInventoryItemID('player', 20)").unwrap();
    let tex: i64 = env
        .eval("return GetInventoryItemTexture('player', 20)")
        .unwrap();
    assert_eq!(item_id, 238017);
    assert_eq!(tex, 7456228);
}

#[test]
fn test_admin_equip_item() {
    let env = env();
    env.exec("A_Admin.EquipItem(17, 229181)").unwrap();
    let id: i64 = env.eval("return GetInventoryItemID('player', 17)").unwrap();
    assert_eq!(id, 229181);
}

#[test]
fn test_admin_unequip_item() {
    let env = env();
    env.exec("A_Admin.EquipItem(17, 229181)").unwrap();
    env.exec("A_Admin.UnequipItem(17)").unwrap();
    let is_nil: bool = env
        .eval("return GetInventoryItemID('player', 17) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_inventory_slot_info() {
    let env = env();
    let (slot_id, tex_id): (i32, i32) =
        env.eval(&inventory_slot_info_chunk("return GetInventorySlotInfo('HeadSlot')")).unwrap();
    assert_eq!(slot_id, 1);
    assert!(tex_id > 0);
}

#[test]
fn test_all_default_gear_slots_populated() {
    let env = env();
    let count: i32 = env
        .eval(
            "local n = 0; \
             for _, s in ipairs({1,2,3,5,6,7,8,9,10,11,12,13,14,15,16}) do \
                 if GetInventoryItemID('player', s) then n = n + 1 end \
             end; \
             return n",
        )
        .unwrap();
    assert_eq!(count, 15);
}
