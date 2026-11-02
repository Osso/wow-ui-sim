//! Integration tests for `C_Cursor.GetCursorItem` — the call surfaced by
//! `Blizzard_AzeriteRespecUI` slot OnClick/OnReceiveDrag at
//! `Blizzard_AzeriteRespecUI.lua:153,163`. Drives the cursor-state model
//! defined in `src/lua_api/state_types/runtime.rs` (`CursorInfo::Item` +
//! `CursorItemOrigin`).

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

/// Inline `ItemLocationMixin` matching
/// `vendor/wow-ui-source/.../Blizzard_ObjectAPI/Mainline/ItemLocation.lua`.
/// `WowLuaEnv::new()` doesn't load Blizzard_ObjectAPI, so tests that exercise
/// mixin methods install the canonical definition first.
const ITEM_LOCATION_MIXIN_BOOTSTRAP: &str = r#"
    ItemLocationMixin = {}
    function ItemLocationMixin:GetBagAndSlot()
        return self.bagID, self.slotIndex
    end
    function ItemLocationMixin:GetEquipmentSlot()
        return self.equipmentSlotIndex
    end
    function ItemLocationMixin:IsBagAndSlot()
        return self.bagID ~= nil and self.slotIndex ~= nil
    end
    function ItemLocationMixin:IsEquipmentSlot()
        return self.equipmentSlotIndex ~= nil
    end
    function ItemLocationMixin:IsEqualTo(other)
        if not other then return false end
        if self:IsBagAndSlot() then
            local ob, os = other:GetBagAndSlot()
            return self.bagID == ob and self.slotIndex == os
        end
        if self:IsEquipmentSlot() then
            return self.equipmentSlotIndex == other:GetEquipmentSlot()
        end
        return false
    end
"#;

#[test]
fn namespace_is_present() {
    let env = WowLuaEnv::new().expect("env");
    let ns_type: String = env.eval("return type(C_Cursor)").unwrap();
    assert_eq!(ns_type, "table");
    let fn_type: String = env.eval("return type(C_Cursor.GetCursorItem)").unwrap();
    assert_eq!(fn_type, "function");
}

#[test]
fn returns_nil_when_cursor_is_empty() {
    let env = WowLuaEnv::new().expect("env");
    let kind: String = env.eval("return type(C_Cursor.GetCursorItem())").unwrap();
    assert_eq!(kind, "nil");
}

#[test]
fn returns_nil_for_non_item_payloads() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Spell { spell_id: 1234 });
    let kind: String = env.eval("return type(C_Cursor.GetCursorItem())").unwrap();
    assert_eq!(kind, "nil");
}

#[test]
fn returns_nil_for_merchant_origin() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Merchant { index: 3 },
        applied_transmog: None,
    });
    let kind: String = env.eval("return type(C_Cursor.GetCursorItem())").unwrap();
    assert_eq!(kind, "nil");
}

#[test]
fn returns_nil_for_unknown_origin() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Unknown,
        applied_transmog: None,
    });
    let kind: String = env.eval("return type(C_Cursor.GetCursorItem())").unwrap();
    assert_eq!(kind, "nil");
}

#[test]
fn bag_origin_returns_bag_and_slot_shape() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 2, slot: 5 },
        applied_transmog: None,
    });
    let bag_id: f64 = env.eval("return C_Cursor.GetCursorItem().bagID").unwrap();
    let slot_index: f64 = env
        .eval("return C_Cursor.GetCursorItem().slotIndex")
        .unwrap();
    assert!((bag_id - 2.0).abs() < 1e-6);
    assert!((slot_index - 5.0).abs() < 1e-6);
    let equipment_kind: String = env
        .eval("return type(C_Cursor.GetCursorItem().equipmentSlotIndex)")
        .unwrap();
    assert_eq!(equipment_kind, "nil");
}

#[test]
fn equipped_origin_returns_equipment_slot_shape() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Equipped { slot: 7 },
        applied_transmog: None,
    });
    let equipment_slot: f64 = env
        .eval("return C_Cursor.GetCursorItem().equipmentSlotIndex")
        .unwrap();
    assert!((equipment_slot - 7.0).abs() < 1e-6);
    let bag_kind: String = env
        .eval("return type(C_Cursor.GetCursorItem().bagID)")
        .unwrap();
    assert_eq!(bag_kind, "nil");
}

#[test]
fn bag_origin_exposes_item_location_mixin_methods() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(ITEM_LOCATION_MIXIN_BOOTSTRAP).unwrap();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 0, slot: 4 },
        applied_transmog: None,
    });
    let is_bag: bool = env
        .eval("return C_Cursor.GetCursorItem():IsBagAndSlot()")
        .unwrap();
    assert!(is_bag);
    let is_equipment: bool = env
        .eval("return C_Cursor.GetCursorItem():IsEquipmentSlot()")
        .unwrap();
    assert!(!is_equipment);
    let bag_id: f64 = env
        .eval("local b, _ = C_Cursor.GetCursorItem():GetBagAndSlot(); return b")
        .unwrap();
    let slot_idx: f64 = env
        .eval("local _, s = C_Cursor.GetCursorItem():GetBagAndSlot(); return s")
        .unwrap();
    assert!((bag_id - 0.0).abs() < 1e-6);
    assert!((slot_idx - 4.0).abs() < 1e-6);
}

#[test]
fn equipped_origin_exposes_item_location_mixin_methods() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(ITEM_LOCATION_MIXIN_BOOTSTRAP).unwrap();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Equipped { slot: 5 },
        applied_transmog: None,
    });
    let is_equipment: bool = env
        .eval("return C_Cursor.GetCursorItem():IsEquipmentSlot()")
        .unwrap();
    assert!(is_equipment);
    let is_bag: bool = env
        .eval("return C_Cursor.GetCursorItem():IsBagAndSlot()")
        .unwrap();
    assert!(!is_bag);
    let slot_idx: f64 = env
        .eval("return C_Cursor.GetCursorItem():GetEquipmentSlot()")
        .unwrap();
    assert!((slot_idx - 5.0).abs() < 1e-6);
}

#[test]
fn item_locations_compare_equal_via_mixin() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(ITEM_LOCATION_MIXIN_BOOTSTRAP).unwrap();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 158041,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 1, slot: 9 },
        applied_transmog: None,
    });
    let equal: bool = env
        .eval(
            r#"
            local a = C_Cursor.GetCursorItem()
            local b = C_Cursor.GetCursorItem()
            return a:IsEqualTo(b)
        "#,
        )
        .unwrap();
    assert!(equal);
}
