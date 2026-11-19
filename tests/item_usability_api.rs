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
                owned_identity: None,
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

#[test]
fn request_load_item_data_is_noop_without_request_model() {
    let ui = crate::common::blizzard_addon_closure::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_closure::new_blizzard_addon_env(&ui);
    let loaded = crate::common::blizzard_addon_closure::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_ObjectAPI"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_ObjectAPI"));
    env.exec(
        r#"
        local location = ItemLocation:CreateFromEquipmentSlot(1)
        assert(select('#', C_Item.RequestLoadItemData(location)) == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn item_usability_defaults_unsupplied_eligibility_to_accepted() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.insert(
            (0, 1),
            BagItem {
                item_id: 1434,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
                owned_identity: None,
            },
        );
        sim.player.power_type = 0;
        sim.player.power = 100;
    }
    let sufficient: (bool, bool) = env.eval("return C_Item.IsUsableItem(1434)").unwrap();
    assert_eq!(sufficient, (true, false));

    env.state().borrow_mut().player.power = 0;
    let insufficient: (bool, bool) = env.eval("return C_Item.IsUsableItem(1434)").unwrap();
    assert_eq!(insufficient, (false, true));
}
