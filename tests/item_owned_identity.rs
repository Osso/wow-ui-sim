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
