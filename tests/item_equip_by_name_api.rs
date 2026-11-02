//! API-level WM57 tests for local equip requests and item locks.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_equip::OwnedItemEquipInputs;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

fn item(item_id: u32, identity: Option<u64>) -> BagItem {
    BagItem {
        owned_identity: identity,
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

#[test]
fn first_matching_locked_copy_prevents_fallthrough() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), item(7005, Some(31)));
        sim.bag_items.insert((0, 3), item(7005, Some(32)));
        sim.item_equip.inputs.insert(
            31,
            OwnedItemEquipInputs {
                native_flags: 2048,
                ..Default::default()
            },
        );
    }
    env.exec(
        r#"
        local first = { bagID = 0, slotIndex = 2 }
        local second = { bagID = 0, slotIndex = 3 }
        assert(C_Item.IsLocked(first) and not C_Item.IsLocked(second))
        C_Item.EquipItemByName('Skinning Knife')
        assert(C_Item.IsLocked(first) and not C_Item.IsLocked(second))
        assert(GetCursorInfo() == nil)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().item_equip.requests.is_empty());
}

#[test]
fn direct_request_retains_cursor_and_lock_follows_the_owned_item() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert((0, 2), item(7005, Some(71)));
    }
    env.exec(
        r#"
        C_Item.PickupItem(6948)
        C_Item.EquipItemByName('item:7005', 16)
        local kind, id = GetCursorInfo()
        assert(kind == 'item' and id == 6948)
        assert(C_Item.IsLocked({ bagID = 0, slotIndex = 2 }))
        ClearCursor()
        assert(C_Container.GetContainerItemInfo(0, 2).isLocked)
        assert(not pcall(C_Item.EquipItemByName, 7005, 20))
    "#,
    )
    .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        assert_eq!(sim.item_equip.requests.len(), 1);
        assert_eq!(sim.item_equip.requests[0].destination, Some(16));
        assert!(sim.player.equipped_items.is_empty());
        let moved = sim.bag_items.remove(&(0, 2)).unwrap();
        sim.bag_items.insert((0, 5), moved);
        sim.bag_items.insert((0, 2), item(7005, Some(72)));
    }
    env.exec(
        r#"
        assert(not C_Container.GetContainerItemInfo(0, 2).isLocked)
        assert(C_Container.GetContainerItemInfo(0, 5).isLocked)
        assert(C_Item.IsLocked({ bagID = 0, slotIndex = 5 }))
    "#,
    )
    .unwrap();
}

#[test]
fn unbound_bop_and_ineligible_boe_items_do_not_open_a_binding_prompt() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        // 211993 has Bonding 1; 25 has Bonding 2 in the 70391 source data.
        sim.bag_items.insert((0, 1), item(211993, Some(11)));
        sim.bag_items.insert((0, 2), item(25, Some(12)));
        sim.item_equip.inputs.insert(
            12,
            OwnedItemEquipInputs {
                confirmation_eligible: Some(false),
                ..Default::default()
            },
        );
    }
    env.exec(
        r#"
        C_Item.EquipItemByName(211993)
        assert(not CursorHasItem() and C_Container.GetContainerItemInfo(0, 1).isLocked)
        C_Item.EquipItemByName(25)
        assert(not CursorHasItem() and C_Container.GetContainerItemInfo(0, 2).isLocked)
    "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.item_equip.requests.len(), 2);
    assert!(sim.item_equip.pending_confirmation.is_none());
}

#[test]
fn automatic_class_six_clears_without_request() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 1), item(24412, Some(11)));
    }
    env.exec(
        r#"
        C_Item.EquipItemByName(24412)
        assert(GetCursorInfo() == nil)
        assert(not C_Container.GetContainerItemInfo(0, 1).isLocked)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().item_equip.requests.is_empty());
}

#[test]
fn account_conversion_retains_cursor_and_clear_restores_lock() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 1), item(7005, Some(42)));
        sim.item_equip.inputs.insert(
            42,
            OwnedItemEquipInputs {
                account_conversion: true,
                ..Default::default()
            },
        );
    }
    env.exec(r#"
        local seen = {}
        local listener = CreateFrame('Frame')
        for _, name in ipairs({'ITEM_LOCK_CHANGED', 'ITEM_LOCKED', 'ITEM_UNLOCKED', 'CONVERT_TO_BIND_TO_ACCOUNT_CONFIRM'}) do
            listener:RegisterEvent(name)
        end
        listener:SetScript('OnEvent', function(_, name) seen[#seen + 1] = name end)
        C_Item.EquipItemByName(7005)
        assert(CursorHasItem() and C_Container.GetContainerItemInfo(0, 1).isLocked)
        assert(table.concat(seen, ',') == 'ITEM_LOCK_CHANGED,ITEM_LOCKED,CONVERT_TO_BIND_TO_ACCOUNT_CONFIRM')
        ClearCursor()
        assert(not C_Container.GetContainerItemInfo(0, 1).isLocked)
        assert(seen[4] == 'ITEM_LOCK_CHANGED' and seen[5] == 'ITEM_UNLOCKED')
    "#).unwrap();
    assert!(env.state().borrow().item_equip.requests.is_empty());
}

#[test]
fn refused_staging_processes_the_previous_stored_item() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 1), item(7005, Some(42)));
        sim.bag_items.insert((0, 2), item(6948, Some(43)));
        sim.item_equip.inputs.insert(
            42,
            OwnedItemEquipInputs {
                account_conversion: true,
                ..Default::default()
            },
        );
    }
    env.exec("C_Item.EquipItemByName(7005); assert(CursorHasItem())")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.item_equip.pickup_control_enabled = false;
        sim.item_equip
            .inputs
            .get_mut(&42)
            .unwrap()
            .account_conversion = false;
    }
    env.exec(
        r#"
        C_Item.EquipItemByName(6948)
        assert(GetCursorInfo() == nil)
        assert(C_Container.GetContainerItemInfo(0, 1).isLocked)
        assert(not C_Container.GetContainerItemInfo(0, 2).isLocked)
    "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.item_equip.requests.len(), 1);
    assert_eq!(sim.item_equip.requests[0].identity, 42);
}

#[test]
fn refused_staging_processes_retained_identity_even_when_cursor_mode_is_empty() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 1), item(7005, Some(42)));
        sim.bag_items.insert((0, 2), item(6948, Some(43)));
        sim.item_equip.inputs.insert(
            42,
            OwnedItemEquipInputs {
                account_conversion: true,
                ..Default::default()
            },
        );
    }
    env.exec("C_Item.EquipItemByName(7005); assert(CursorHasItem())")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        // Supplied mode-0 state with retained owned identity (WM44-B).
        sim.cursor_item = None;
        sim.item_equip.pickup_control_enabled = false;
        sim.item_equip
            .inputs
            .get_mut(&42)
            .unwrap()
            .account_conversion = false;
    }
    env.exec("C_Item.EquipItemByName(6948)").unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.item_equip.requests.len(), 1);
    assert_eq!(sim.item_equip.requests[0].identity, 42);
}

#[test]
fn replacing_the_selected_item_during_cursor_clear_does_not_transfer_its_identity() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), item(7005, None));
    }
    env.exec(
        r#"
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ACTIONBAR_HIDEGRID')
        listener:SetScript('OnEvent', function()
            -- Replace the selected item with a distinct copy of the same ID.
            A_Admin.AddBagItem(0, 2, 7005, 1)
        end)
        C_Item.PickupItem(6948)
        C_Item.EquipItemByName(7005)
        assert(GetCursorInfo() == nil)
        assert(not C_Container.GetContainerItemInfo(0, 2).isLocked)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().item_equip.requests.is_empty());
}

#[test]
fn carried_bag_contents_take_priority_over_equipment() {
    use wow_ui_sim::lua_api::state::EquippedItem;
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_info.get_mut(&1).unwrap().item_id = Some(21841);
        sim.bag_items.insert((1, 3), item(7005, Some(91)));
        sim.player.equipped_items.insert(
            16,
            EquippedItem {
                owned_identity: Some(92),
                item_id: 7005,
                enchant_id: 0,
                gem_ids: [0; 3],
                applied_transmog: None,
            },
        );
    }
    env.exec(
        r#"
        C_Item.EquipItemByName(7005)
        assert(C_Container.GetContainerItemInfo(1, 3).isLocked)
        assert(not C_Item.IsLocked({ equipmentSlotIndex = 16 }))
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().item_equip.requests[0].identity, 91);
}
