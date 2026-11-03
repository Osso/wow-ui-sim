//! WM65/WM67 cursor item slot compatibility for `C_PaperDollInfo.CanCursorCanGoInSlot`.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_child::ItemChildEquipment;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, CursorInfo, CursorItemOrigin};

fn item(item_id: u32) -> BagItem {
    BagItem {
        owned_identity: None,
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

fn env_with_bag_item(item_id: u32) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 1), item(item_id));
    }
    env.exec("C_Container.PickupContainerItem(0, 1)").unwrap();
    env
}

fn can_go(env: &WowLuaEnv, slot: &str) -> bool {
    env.eval(&format!(
        "return C_PaperDollInfo.CanCursorCanGoInSlot({slot})"
    ))
    .unwrap()
}

#[test]
fn only_an_owned_item_cursor_can_pass() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().bag_items.clear();
    assert!(!can_go(&env, "16"));
    // C_Item.PickupItem installs ActionBar mode 6, not an owned item.
    env.exec("C_Item.PickupItem(25)").unwrap();
    assert!(!can_go(&env, "16"));
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Money { copper: 5 });
    assert!(!can_go(&env, "16"));
    env.exec("ClearCursor()").unwrap();
    env.state().borrow_mut().bag_items.insert((0, 1), item(25));
    env.exec("C_Container.PickupContainerItem(0, 1)").unwrap();
    assert!(can_go(&env, "16"));
    env.state().borrow_mut().player.present = false;
    assert!(!can_go(&env, "16"));
}

#[test]
fn inventory_type_masks_select_public_slots() {
    // Item 25: class 2, InventoryType 21 (WEAPONMAINHAND).
    let env = env_with_bag_item(25);
    assert!(can_go(&env, "16"));
    assert!(!can_go(&env, "17"));
    assert!(!can_go(&env, "1"));
    // Item 876: InventoryType 14 (SHIELD).
    let env = env_with_bag_item(876);
    assert!(can_go(&env, "17"));
    assert!(!can_go(&env, "16"));
}

#[test]
fn slot_argument_is_decremented_before_truncation() {
    let env = env_with_bag_item(25);
    assert!(can_go(&env, "16.9"));
    assert!(!can_go(&env, "17.5"));
    // Public 0 becomes native all-ones, which accepts an ordinary item.
    assert!(can_go(&env, "0"));
    assert!(can_go(&env, "36"));
    assert!(can_go(&env, "4294967295"));
    for rejected in ["nil", "-1", "4294967296", "{}"] {
        let call = format!("return pcall(C_PaperDollInfo.CanCursorCanGoInSlot, {rejected})");
        let accepted: bool = env.eval(&call).unwrap();
        assert!(!accepted, "{rejected} should raise");
    }
}

#[test]
fn excluded_native_slots_reject_before_mask_matching() {
    let env = env_with_bag_item(25);
    for public in [
        "18", "87", "88", "91", "92", "94", "95", "96", "98", "99", "100",
    ] {
        assert!(!can_go(&env, public), "public slot {public}");
    }
    for public in ["89", "93", "97", "101"] {
        assert!(can_go(&env, public), "public slot {public}");
    }
}

#[test]
fn child_equipment_row_rejects_its_native_slot() {
    let env = env_with_bag_item(25);
    env.state().borrow_mut().item_children.insert(
        25,
        vec![ItemChildEquipment {
            slot_index: 15,
            child_item_id: 6948,
        }],
    );
    assert!(!can_go(&env, "16"));
    env.state().borrow_mut().item_children.get_mut(&25).unwrap()[0].slot_index = 40;
    assert!(can_go(&env, "16"));
    assert!(!can_go(&env, "41"));
}

#[test]
fn two_handed_offhand_requires_weapon_subclass_and_marker() {
    // Item 756: class 2, subclass 1, InventoryType 17 (2HWEAPON).
    let env = env_with_bag_item(756);
    assert!(can_go(&env, "16"));
    assert!(!can_go(&env, "17"));
    env.state().borrow_mut().item_equip.offhand_two_hand_marker = true;
    assert!(can_go(&env, "17"));
    // Item 6174: class 2, subclass 14 (not in the allowed set).
    let env = env_with_bag_item(6174);
    env.state().borrow_mut().item_equip.offhand_two_hand_marker = true;
    assert!(can_go(&env, "16"));
    assert!(!can_go(&env, "17"));
}

#[test]
fn bag_masks_reserve_the_last_bag_slot_for_reagent_bags() {
    // Item 804: class 1, subclass 0, InventoryType 18.
    let env = env_with_bag_item(804);
    assert!(can_go(&env, "31"));
    assert!(can_go(&env, "34"));
    assert!(!can_go(&env, "35"));
    // Item 176398: subclass 11 reagent bag.
    let env = env_with_bag_item(176398);
    assert!(can_go(&env, "35"));
    assert!(!can_go(&env, "31"));
}

#[test]
fn nonempty_container_is_refused_outside_bag_ranges() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        let bag = sim.bag_info.get_mut(&1).unwrap();
        bag.item_id = Some(804);
        bag.inventory_slot = Some(21);
        sim.bag_items.insert((1, 3), item(6948));
        sim.cursor_item = Some(CursorInfo::Item {
            owned_identity: None,
            item_id: 804,
            stack_count: 1,
            origin: CursorItemOrigin::Equipped { slot: 21 },
            applied_transmog: None,
        });
    }
    assert!(can_go(&env, "32"));
    assert!(can_go(&env, "64"));
    assert!(!can_go(&env, "0"));
    assert!(!can_go(&env, "70"));
    env.state().borrow_mut().bag_items.clear();
    assert!(can_go(&env, "0"));
}

#[test]
fn profession_items_require_a_listed_profession_slot() {
    // Item 778: class 19 subclass 5 (Profession row 6, key 5), type 29.
    let env = env_with_bag_item(778);
    assert!(can_go(&env, "26"));
    assert!(!can_go(&env, "20"));
    // Item 198243: class 19 subclass 5, type 30.
    let env = env_with_bag_item(198243);
    assert!(can_go(&env, "27"));
    assert!(!can_go(&env, "21"));
    // Item 191227: Blacksmithing tool; Blacksmithing is the first primary.
    let env = env_with_bag_item(191227);
    assert!(can_go(&env, "20"));
    assert!(!can_go(&env, "23"));
    // Without Blacksmithing, neither primary matches its skill line.
    env.state()
        .borrow_mut()
        .crafting
        .unlearned_profession_ids
        .insert(164);
    assert!(!can_go(&env, "20"));
    // Item 194125: Mining tool; Mining is the second primary.
    let env = env_with_bag_item(194125);
    assert!(can_go(&env, "23"));
    assert!(!can_go(&env, "20"));
    // The profession filter also applies at native 35 and above.
    let env = env_with_bag_item(191227);
    assert!(!can_go(&env, "36"));
}
