//! Tests for C_Item, C_Container, C_EncodingUtil, and related global functions (c_item_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// C_Item.GetItemInfo
// ============================================================================

#[test]
fn test_c_item_get_item_info_returns_nil() {
    let env = env();
    // Item 42 doesn't exist in the DB, should return nil
    let is_nil: bool = env
        .eval("return C_Item.GetItemInfo(42) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_item_get_item_info_returns_data() {
    let env = env();
    // Item 6948 (Hearthstone) exists in the DB
    let name: String = env
        .eval("local info = C_Item.GetItemInfo(6948); return info.itemName")
        .unwrap();
    assert_eq!(name, "Hearthstone");
}

// ============================================================================
// C_Item.GetItemInfoInstant
// ============================================================================

#[test]
fn test_c_item_get_item_info_instant_by_id() {
    let env = env();
    let (item_id, item_type, item_sub_type): (i64, String, String) = env
        .eval("return C_Item.GetItemInfoInstant(12345)")
        .unwrap();
    assert_eq!(item_id, 12345);
    assert_eq!(item_type, "Miscellaneous");
    assert_eq!(item_sub_type, "Junk");
}

#[test]
fn test_c_item_get_item_info_instant_by_link() {
    let env = env();
    let item_id: i64 = env
        .eval(r#"return C_Item.GetItemInfoInstant("|cffffffff|Hitem:54321::::::::80:::::|h[Test]|h|r")"#)
        .unwrap();
    assert_eq!(item_id, 54321);
}

#[test]
fn test_c_item_get_item_info_instant_invalid() {
    let env = env();
    let count: i32 = env
        .eval("return select('#', C_Item.GetItemInfoInstant(nil))")
        .unwrap();
    assert_eq!(count, 0);
}

// ============================================================================
// C_Item.GetItemIDForItemInfo
// ============================================================================

#[test]
fn test_c_item_get_item_id_for_item_info_integer() {
    let env = env();
    let id: i64 = env
        .eval("return C_Item.GetItemIDForItemInfo(999)")
        .unwrap();
    assert_eq!(id, 999);
}

#[test]
fn test_c_item_get_item_id_for_item_info_link() {
    let env = env();
    let id: i64 = env
        .eval(r#"return C_Item.GetItemIDForItemInfo("|cffffffff|Hitem:42::::::::80:::::|h[X]|h|r")"#)
        .unwrap();
    assert_eq!(id, 42);
}

#[test]
fn test_c_item_get_item_id_for_item_info_invalid() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return C_Item.GetItemIDForItemInfo("not a link") == nil"#)
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// C_Item.GetItemIconByID / GetItemQualityByID / GetItemLink
// ============================================================================

#[test]
fn test_c_item_get_item_icon_by_id() {
    let env = env();
    let icon: i32 = env.eval("return C_Item.GetItemIconByID(1)").unwrap();
    assert_eq!(icon, 134400);
}

#[test]
fn test_c_item_get_item_quality_by_id() {
    let env = env();
    let quality: i32 = env
        .eval("return C_Item.GetItemQualityByID(1)")
        .unwrap();
    assert_eq!(quality, 1);
}

#[test]
fn test_c_item_get_item_link() {
    let env = env();
    let link: String = env.eval("return C_Item.GetItemLink(6948)").unwrap();
    assert!(link.contains("Hitem:6948"));
    assert!(link.contains("[Hearthstone]"));
}

// ============================================================================
// C_Item.GetItemNameByID
// ============================================================================

#[test]
fn test_c_item_get_item_name_by_id() {
    let env = env();
    let name: String = env.eval("return C_Item.GetItemNameByID(6948)").unwrap();
    assert_eq!(name, "Hearthstone");
}

#[test]
fn test_c_item_get_item_name_by_id_unknown() {
    let env = env();
    let name: String = env.eval("return C_Item.GetItemNameByID(42)").unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// C_Item.GetItemSubClassInfo
// ============================================================================

#[test]
fn test_c_item_get_item_sub_class_info_weapon() {
    let env = env();
    let name: String = env
        .eval("return C_Item.GetItemSubClassInfo(2, 7)")
        .unwrap();
    assert_eq!(name, "One-Handed Swords");
}

#[test]
fn test_c_item_get_item_sub_class_info_armor() {
    let env = env();
    let name: String = env
        .eval("return C_Item.GetItemSubClassInfo(4, 4)")
        .unwrap();
    assert_eq!(name, "Plate");
}

#[test]
fn test_c_item_get_item_sub_class_info_unknown() {
    let env = env();
    let name: String = env
        .eval("return C_Item.GetItemSubClassInfo(99, 99)")
        .unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// C_Item.GetItemClassInfo
// ============================================================================

#[test]
fn test_c_item_get_item_class_info() {
    let env = env();
    let name: String = env.eval("return C_Item.GetItemClassInfo(2)").unwrap();
    assert_eq!(name, "Weapon");
}

// ============================================================================
// C_Item.GetDetailedItemLevelInfo
// ============================================================================

#[test]
fn test_c_item_get_detailed_item_level_info() {
    let env = env();
    // Non-existent item returns 0
    let (a, b, c): (i32, i32, i32) = env
        .eval("return C_Item.GetDetailedItemLevelInfo(42)")
        .unwrap();
    assert_eq!((a, b, c), (0, 0, 0));
}

#[test]
fn test_c_item_get_detailed_item_level_info_real() {
    let env = env();
    // Hearthstone (6948) has a real item level
    let (level, _, _): (i32, i32, i32) = env
        .eval("return C_Item.GetDetailedItemLevelInfo(6948)")
        .unwrap();
    assert!(level > 0);
}

// ============================================================================
// C_Item.GetItemCount
// ============================================================================

#[test]
fn test_c_item_get_item_count() {
    let env = env();
    let count: i32 = env.eval("return C_Item.GetItemCount(12345)").unwrap();
    assert_eq!(count, 0);
}

// ============================================================================
// C_Container
// ============================================================================

#[test]
fn test_c_container_get_num_slots_backpack() {
    let env = env();
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(0)")
        .unwrap();
    assert_eq!(slots, 16);
}

#[test]
fn test_c_container_get_num_slots_other_bag() {
    let env = env();
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(1)")
        .unwrap();
    assert_eq!(slots, 0);
}

#[test]
fn test_c_container_get_item_id_populated_slot() {
    let env = env();
    let id: i64 = env
        .eval("return C_Container.GetContainerItemID(0, 1)")
        .unwrap();
    assert_eq!(id, 6948, "Slot 1 should contain Hearthstone");
}

#[test]
fn test_c_container_get_item_id_empty_slot() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemID(0, 2) == nil")
        .unwrap();
    assert!(is_nil, "Slot 2 should be empty");
}

// ============================================================================
// C_EncodingUtil
// ============================================================================

#[test]
fn test_c_encoding_util_compress_decompress() {
    let env = env();
    let result: String = env
        .eval(r#"return C_EncodingUtil.CompressString("hello")"#)
        .unwrap();
    assert_eq!(result, "hello");

    let result: String = env
        .eval(r#"return C_EncodingUtil.DecompressString("hello")"#)
        .unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_c_encoding_util_base64() {
    let env = env();
    let encoded: String = env
        .eval(r#"return C_EncodingUtil.EncodeBase64("data")"#)
        .unwrap();
    assert_eq!(encoded, "data");

    let decoded: String = env
        .eval(r#"return C_EncodingUtil.DecodeBase64("data")"#)
        .unwrap();
    assert_eq!(decoded, "data");
}

// ============================================================================
// Legacy global functions
// ============================================================================

#[test]
fn test_legacy_get_item_info() {
    let env = env();
    let is_nil: bool = env.eval("return GetItemInfo(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_legacy_get_item_id() {
    let env = env();
    let id: i32 = env
        .eval(r#"return GetItemID("|cffffffff|Hitem:777::::::::80:::::|h[X]|h|r")"#)
        .unwrap();
    assert_eq!(id, 777);
}

#[test]
fn test_legacy_get_item_id_nil() {
    let env = env();
    let is_nil: bool = env.eval("return GetItemID(nil) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_legacy_get_item_class_info() {
    let env = env();
    let name: String = env.eval("return GetItemClassInfo(4)").unwrap();
    assert_eq!(name, "Armor");
}

#[test]
fn test_legacy_get_container_num_slots() {
    let env = env();
    let slots: i32 = env.eval("return GetContainerNumSlots(0)").unwrap();
    assert_eq!(slots, 16);
}

// ============================================================================
// Inventory slot functions
// ============================================================================

#[test]
fn test_get_inventory_slot_info() {
    let env = env();
    let slot: i32 = env
        .eval(r#"return GetInventorySlotInfo("HeadSlot")"#)
        .unwrap();
    assert_eq!(slot, 1);
}

#[test]
fn test_get_inventory_slot_info_mainhand() {
    let env = env();
    let slot: i32 = env
        .eval(r#"return GetInventorySlotInfo("MainHandSlot")"#)
        .unwrap();
    assert_eq!(slot, 16);
}

#[test]
fn test_get_inventory_item_link_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return GetInventoryItemLink("player", 1) == nil"#)
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// Spell functions (in c_item_api)
// ============================================================================

#[test]
fn test_get_spell_link() {
    let env = env();
    let link: String = env.eval("return GetSpellLink(100)").unwrap();
    assert!(link.contains("Hspell:100"));
}

#[test]
fn test_get_spell_icon() {
    let env = env();
    // Spell 100 exists with a real icon; just verify it returns a positive number
    let icon: i32 = env.eval("return GetSpellIcon(100)").unwrap();
    assert!(icon > 0);
}

#[test]
fn test_get_spell_icon_unknown() {
    let env = env();
    // Non-existent spell falls back to default icon
    let icon: i32 = env.eval("return GetSpellIcon(999999999)").unwrap();
    assert_eq!(icon, 136243);
}

#[test]
fn test_get_spell_cooldown() {
    let env = env();
    let (start, duration, enabled): (f64, f64, i32) = env
        .eval("return GetSpellCooldown(100)")
        .unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enabled, 1);
}

#[test]
fn test_is_spell_known() {
    let env = env();
    let known: bool = env.eval("return IsSpellKnown(100)").unwrap();
    assert!(!known);
}
