//! API-level coverage: no pinned retail Blizzard consumer calls IsUsableItem.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

#[test]
fn item_usability_checks_ownership_eligibility_and_spell_power() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.bag_items.clear();
        sim.player
            .equipped_items
            .retain(|_, item| item.item_id != 1434);
    }
    env.exec("local a,b=C_Item.IsUsableItem(1434); assert(not a and not b)")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.bag_items.insert(
            (0, 1),
            BagItem {
                item_id: 1434,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
        sim.item_use_eligibility.insert((1434, 13424), false);
        sim.player.power_type = 0;
        sim.player.power = 100;
    }
    env.exec("local a,b=C_Item.IsUsableItem(1434); assert(not a and not b)")
        .unwrap();
    env.state()
        .borrow_mut()
        .item_use_eligibility
        .insert((1434, 13424), true);
    env.exec("local a,b=C_Item.IsUsableItem(1434); assert(a and not b)")
        .unwrap();
    env.state().borrow_mut().player.power = 20;
    env.exec("local a,b=C_Item.IsUsableItem(1434); assert(not a and b); assert(select('#', C_Item.IsUsableItem(1434)) == 2)").unwrap();
}
