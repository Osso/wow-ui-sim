#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;

const EMPTY_READS: &str = r#"
    assert(GetNumBuybackItems() == 0)
    assert(select('#', GetNumBuybackItems()) == 1)
    for _, index in ipairs({-1, 0, 1, 13, 4294967296}) do
        assert(GetBuybackItemInfo(index) == nil)
        assert(select('#', GetBuybackItemInfo(index)) == 0)
        assert(GetBuybackItemLink(index) == nil)
        assert(select('#', GetBuybackItemLink(index)) == 0)
    end
"#;

fn populate(env: &WowLuaEnv) {
    env.state().borrow_mut().merchant_buyback_items = vec![
        BuybackItem {
            item_id: 2840,
            quantity: 3,
            price: 30,
            num_available: -1,
            is_usable: true,
            is_bound: false,
        },
        BuybackItem {
            item_id: 2852,
            quantity: 1,
            price: 79,
            num_available: 1,
            is_usable: false,
            is_bound: true,
        },
    ];
}

#[test]
fn merchant_buyback_empty_and_bootstrap() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(EMPTY_READS).unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(EMPTY_READS).unwrap();
}

#[test]
fn merchant_buyback_populated_metadata_and_slot_order() {
    let env = WowLuaEnv::new().unwrap();
    populate(&env);
    let reads = r#"
        assert(GetNumBuybackItems() == 2)
        local name, icon, price, quantity, available, usable, bound = GetBuybackItemInfo(1)
        assert(select('#', GetBuybackItemInfo(1)) == 7)
        assert(name == 'Copper Bar' and icon == 133216)
        assert(price == 30 and quantity == 3 and available == -1)
        assert(usable == true and bound == false)
        assert(GetBuybackItemLink(1) == '|cnIQ1:|Hitem:2840::::::::80:70:::::::::|h[Copper Bar]|h|r')
        name, icon, price, quantity, available, usable, bound = GetBuybackItemInfo(2)
        assert(name == 'Copper Chain Pants' and icon == 134583)
        assert(price == 79 and quantity == 1 and available == 1)
        assert(usable == false and bound == true)
        assert(GetBuybackItemLink(2) == '|cnIQ1:|Hitem:2852::::::::80:70:::::::::|h[Copper Chain Pants]|h|r')
        assert(select('#', GetBuybackItemLink(2)) == 1)
        assert(GetBuybackItemInfo(0) == nil and GetBuybackItemInfo(3) == nil)
        assert(GetBuybackItemLink(0) == nil and GetBuybackItemLink(3) == nil)
    "#;
    env.exec(reads).unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(reads).unwrap();
    env.state().borrow_mut().merchant_buyback_items.remove(0);
    env.exec("assert(GetNumBuybackItems() == 1); assert(GetBuybackItemInfo(1) == 'Copper Chain Pants'); assert(GetBuybackItemLink(2) == nil)").unwrap();
}

#[test]
fn merchant_buyback_inventory_capability_and_environment_independence() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    populate(&first);
    {
        let mut state = first.state().borrow_mut();
        state.merchant_items = vec![6948];
        state.merchant_frame_open = true;
        state.merchant_repair_capable = true;
    }
    first.exec("assert(GetMerchantNumItems() == 1); assert(GetNumBuybackItems() == 2); assert(CanMerchantRepair()); CloseMerchant(); assert(not CanMerchantRepair()); assert(GetNumBuybackItems() == 2)").unwrap();
    second.exec(EMPTY_READS).unwrap();
    assert_eq!(first.state().borrow().merchant_items, vec![6948]);
    assert!(first.state().borrow().merchant_repair_capable);
    assert!(second.state().borrow().merchant_items.is_empty());
    first.state().borrow_mut().merchant_buyback_items.clear();
    first.exec(EMPTY_READS).unwrap();
}

#[test]
fn merchant_buyback_rejects_invalid_arguments_and_unknown_metadata() {
    let env = WowLuaEnv::new().unwrap();
    populate(&env);
    env.exec(
        r#"
        for _, read in ipairs({GetBuybackItemInfo, GetBuybackItemLink}) do
            assert(not pcall(read))
            assert(not pcall(read, {}))
            assert(not pcall(read, 'invalid'))
        end
        assert(GetNumBuybackItems() == 2)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().merchant_buyback_items[0].item_id = u32::MAX;
    env.exec(
        r#"
        for _, read in ipairs({GetBuybackItemInfo, GetBuybackItemLink}) do
            local ok, message = pcall(read, 1)
            assert(not ok and tostring(message):find('unknown buyback item 4294967295', 1, true))
        end
        assert(GetNumBuybackItems() == 2)
        assert(GetBuybackItemInfo(2) == 'Copper Chain Pants')
    "#,
    )
    .unwrap();
}
