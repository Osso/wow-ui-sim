//! API-level coverage: no pinned Blizzard consumer calls IsEquippedItemType.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::EquippedItem;

prefork_full_ui_case! {
fn equipped_item_type_matches_equipped_item_categories(env: &WowLuaEnv) {
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(1, EquippedItem {
            item_id: 211993,
            enchant_id: 0,
            gem_ids: [0; 3],
            applied_transmog: None,
            owned_identity: None,
        });
    }
    env.exec(r#"
        assert(C_Item.IsEquippedItemType('INVTYPE_HEAD'))
        assert(C_Item.IsEquippedItemType('armor'))
        assert(C_Item.IsEquippedItemType('Plate'))
        assert(not C_Item.IsEquippedItemType('Weapon'))
        assert(not C_Item.IsEquippedItemType('not an item type'))
        assert(not pcall(C_Item.IsEquippedItemType))
        assert(not pcall(C_Item.IsEquippedItemType, nil))
        assert(not pcall(C_Item.IsEquippedItemType, true))
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(16, EquippedItem {
            item_id: 128872,
            enchant_id: 0,
            gem_ids: [0; 3],
            applied_transmog: None,
            owned_identity: None,
        });
    }
    env.exec(r#"
        assert(C_Item.IsEquippedItemType('INVTYPE_WEAPONMAINHAND'))
        assert(C_Item.IsEquippedItemType('main hand'))
        assert(C_Item.IsEquippedItemType('Weapon'))
        assert(C_Item.IsEquippedItemType('One-Handed Swords'))
        assert(C_Item.IsEquippedItemType('Melee Weapon'))
        assert(not C_Item.IsEquippedItemType('INVTYPE_WEAPON'))
        assert(not C_Item.IsEquippedItemType('Plate'))
    "#).unwrap();
    env.state().borrow_mut().player.equipped_items.clear();
    env.exec("assert(not C_Item.IsEquippedItemType('INVTYPE_HEAD'))").unwrap();
}
}
