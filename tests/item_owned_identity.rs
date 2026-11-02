//! Owned identities survive ordinary simulator inventory transfers.
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, CursorInfo};

#[test]
fn identity_survives_bag_cursor_equipment_and_backpack_transfers() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert(
            (0, 1),
            BagItem {
                item_id: 7005,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
                owned_identity: Some(71),
            },
        );
    }
    env.exec("C_Container.PickupContainerItem(0, 1)").unwrap();
    assert!(matches!(
        env.state().borrow().cursor_item,
        Some(CursorInfo::Item {
            owned_identity: Some(71),
            ..
        })
    ));
    env.exec("EquipCursorItem(16)").unwrap();
    assert_eq!(
        env.state().borrow().player.equipped_items[&16].owned_identity,
        Some(71)
    );
    env.exec("PickupInventoryItem(16); PutItemInBackpack()")
        .unwrap();
    assert_eq!(
        env.state().borrow().bag_items[&(0, 1)].owned_identity,
        Some(71)
    );
}

#[test]
fn equipped_bag_alias_does_not_duplicate_identity_on_the_cursor() {
    use wow_ui_sim::lua_api::state::EquippedItem;
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let bag = sim.bag_info.get_mut(&1).unwrap();
        bag.item_id = Some(21841);
        bag.inventory_slot = Some(21);
        bag.owned_identity = Some(71);
        sim.player.equipped_items.insert(
            21,
            EquippedItem {
                item_id: 21841,
                enchant_id: 0,
                gem_ids: [0; 3],
                applied_transmog: None,
                owned_identity: Some(71),
            },
        );
    }
    env.exec("PickupInventoryItem(21)").unwrap();
    {
        let sim = env.state().borrow();
        assert_eq!(sim.bag_info[&1].owned_identity, None);
        assert!(matches!(
            sim.cursor_item,
            Some(CursorInfo::Item {
                owned_identity: Some(71),
                ..
            })
        ));
    }
    env.exec("EquipCursorItem(21)").unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.player.equipped_items[&21].owned_identity, Some(71));
    assert_eq!(sim.bag_info[&1].owned_identity, Some(71));
}
