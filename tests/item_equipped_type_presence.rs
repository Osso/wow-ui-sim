//! API-level bounds for WM29 player presence and the selected equipment range.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::EquippedItem;

#[test]
fn equipped_item_type_requires_player_and_ignores_slots_after_eighteen() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(
            1,
            EquippedItem {
                item_id: 211993,
                enchant_id: 0,
                gem_ids: [0; 3],
                applied_transmog: None,
            },
        );
        sim.player.present = false;
    }
    env.exec("assert(not C_Item.IsEquippedItemType('INVTYPE_HEAD'))")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.present = true;
        let item = sim.player.equipped_items.remove(&1).unwrap();
        sim.player.equipped_items.insert(19, item);
    }
    env.exec("assert(not C_Item.IsEquippedItemType('INVTYPE_HEAD'))")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let item = sim.player.equipped_items.remove(&19).unwrap();
        sim.player.equipped_items.insert(18, item);
    }
    env.exec("assert(C_Item.IsEquippedItemType('INVTYPE_HEAD'))")
        .unwrap();
}
