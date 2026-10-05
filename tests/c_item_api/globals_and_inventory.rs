use crate::common::inventory_slot_info_chunk;
use crate::support::env;

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

#[test]
fn test_legacy_get_item_info() {
    let env = env();
    let name: String = env.eval("return GetItemInfo(1)").unwrap();
    assert_eq!(name, "Unknown");
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

#[test]
fn test_legacy_get_container_item_id() {
    let env = env();
    let item_id: i32 = env.eval("return GetContainerItemID(0, 1)").unwrap();
    assert_eq!(item_id, 6948);
}

#[test]
fn test_legacy_get_container_item_link() {
    let env = env();
    let link: String = env.eval("return GetContainerItemLink(0, 1)").unwrap();
    assert!(link.contains("Hitem:6948"));
}

#[test]
fn test_get_inventory_slot_info() {
    let env = env();
    let (slot, texture, check_relic): (i32, i32, bool) = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("HeadSlot")"#))
        .unwrap();
    assert_eq!(slot, 1);
    assert_eq!(texture, 136516);
    assert!(!check_relic);
}

#[test]
fn test_get_inventory_slot_info_mainhand() {
    let env = env();
    let (slot, texture): (i32, i32) = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("MainHandSlot")"#))
        .unwrap();
    assert_eq!(slot, 16);
    assert_eq!(texture, 136518);
}

#[test]
fn test_get_inventory_slot_info_bag_slots() {
    let env = env();
    let slot: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("Bag0Slot")"#))
        .unwrap();
    assert_eq!(slot, 20);
    let slot: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("Bag1Slot")"#))
        .unwrap();
    assert_eq!(slot, 21);
    let slot: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("Bag2Slot")"#))
        .unwrap();
    assert_eq!(slot, 22);
    let slot: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("Bag3Slot")"#))
        .unwrap();
    assert_eq!(slot, 23);
}

#[test]
fn test_get_inventory_slot_info_returns_three_values() {
    let env = env();
    let count: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return select('#', GetInventorySlotInfo("HeadSlot"))"#))
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_get_inventory_slot_info_bag_slot_texture() {
    let env = env();
    let (slot, texture): (i32, i32) = env
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("Bag0Slot")"#))
        .unwrap();
    assert_eq!(slot, 20);
    assert_eq!(texture, 136511);
}

#[test]
fn test_get_inventory_item_texture_bag_slots() {
    let env = env();
    let tex: String = env
        .eval(r#"return GetInventoryItemTexture("player", 20)"#)
        .unwrap();
    assert!(tex.contains("INV_Misc_Bag_08"));
    let has_tex: bool = env
        .eval(r#"return GetInventoryItemTexture("player", 1) ~= nil"#)
        .unwrap();
    assert!(has_tex, "Equipped slot should return a texture");
    let is_nil: bool = env
        .eval(r#"return GetInventoryItemTexture("player", 19) == nil"#)
        .unwrap();
    assert!(is_nil, "Empty slot should return nil");
}

#[test]
fn test_get_inventory_item_link_empty_slot() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return GetInventoryItemLink("player", 19) == nil"#)
        .unwrap();
    assert!(is_nil, "Empty slot should return nil");
    let has_link: bool = env
        .eval(r#"return GetInventoryItemLink("player", 1) ~= nil"#)
        .unwrap();
    assert!(has_link, "Equipped slot should return a link");
}

#[test]
fn test_get_inventory_item_broken() {
    let env = env();
    let broken: bool = env
        .eval(r#"return GetInventoryItemBroken("player", 20)"#)
        .unwrap();
    assert!(!broken);
}

#[test]
fn test_get_inventory_item_durability() {
    let env = env();
    let (durability, max_durability): (i32, i32) =
        env.eval(r#"return GetInventoryItemDurability(1)"#).unwrap();
    assert_eq!((durability, max_durability), (100, 100));

    let empty_slot_is_nil: bool = env
        .eval(r#"return GetInventoryItemDurability(19) == nil"#)
        .unwrap();
    assert!(empty_slot_is_nil, "Empty slot should return nil");
}

#[test]
fn test_get_inventory_item_cooldown() {
    let env = env();
    let (start, dur, enabled): (f64, f64, i32) = env
        .eval(r#"return GetInventoryItemCooldown("player", 20)"#)
        .unwrap();
    assert_eq!((start, dur, enabled), (0.0, 0.0, 1));
}

#[test]
fn test_get_spell_link() {
    let env = env();
    let link: String = env.eval("return GetSpellLink(100)").unwrap();
    assert!(link.contains("Hspell:100"));
}

#[test]
fn test_get_spell_icon() {
    let env = env();
    let icon: i32 = env.eval("return GetSpellIcon(100)").unwrap();
    assert!(icon > 0);
}

#[test]
fn test_get_spell_icon_unknown() {
    let env = env();
    let icon: i32 = env.eval("return GetSpellIcon(999999999)").unwrap();
    assert_eq!(icon, 136243);
}

#[test]
fn test_get_spell_cooldown() {
    let env = env();
    let (start, duration, enabled): (f64, f64, i32) =
        env.eval("return GetSpellCooldown(100)").unwrap();
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

#[test]
fn test_item_button_icon_has_anchors() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local btn = CreateFrame("ItemButton", "TestBagBtn", UIParent)
            btn:SetSize(30, 30)
            local icon = btn.icon
            if not icon then return "no_icon" end
            local n = icon:GetNumPoints()
            if n == 0 then return "no_anchors" end
            local parts = {}
            for i = 1, n do
                local pt, rel, relPt, x, y = icon:GetPoint(i)
                parts[#parts+1] = pt .. ">" .. (rel and rel:GetName() or "nil") .. "." .. relPt
            end
            return n .. ":" .. table.concat(parts, ",")
            "#,
        )
        .unwrap();
    assert!(result.contains("2:"), "Expected 2 anchors, got: {}", result);
    assert!(
        result.contains("TOPLEFT"),
        "Missing TOPLEFT anchor: {}",
        result
    );
    assert!(
        result.contains("BOTTOMRIGHT"),
        "Missing BOTTOMRIGHT anchor: {}",
        result
    );
}

#[test]
fn test_item_button_icon_identity() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local btn = CreateFrame("ItemButton", "TestIBIdentity", UIParent)
            btn:SetSize(30, 30)
            local icon = btn.icon
            if not icon then return "no_icon" end
            local name = icon:GetName() or "anonymous"
            local numPts = icon:GetNumPoints()
            local globalIcon = _G["TestIBIdentityIconTexture"]
            local globalName = globalIcon and (globalIcon:GetName() or "anon-global") or "no_global"
            local globalPts = globalIcon and globalIcon:GetNumPoints() or -1
            local same = (globalIcon == icon) and "same" or "different"
            return name .. "|pts=" .. numPts .. "|global=" .. globalName .. "|gpts=" .. globalPts .. "|" .. same
            "#,
        )
        .unwrap();
    eprintln!("icon identity: {}", result);
    assert!(
        result.contains("pts=2"),
        "Icon should have 2 anchors: {}",
        result
    );
}
