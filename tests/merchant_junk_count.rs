#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::c_merchant_frame::junk::junk_stack_quantity;
use wow_ui_sim::items::ItemInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

const EMPTY_READS: &str = r#"
    assert(C_MerchantFrame.GetNumJunkItems() == 0, "expected numeric zero junk count")
    assert(select('#', C_MerchantFrame.GetNumJunkItems()) == 1)
"#;

fn poor_item() -> ItemInfo {
    ItemInfo {
        name: "Test worn scrap",
        quality: 0,
        item_level: 1,
        required_level: 0,
        inventory_type: 0,
        sell_price: 7,
        stackable: 20,
        bonding: 0,
        expansion_id: 0,
        icon_file_data_id: 134400,
        stat_percent_editor: [0; 10],
        stat_modifier_bonus_stat: [0; 10],
    }
}

#[test]
fn merchant_junk_count_positive_stack_quantities_and_metadata() {
    let poor = poor_item();
    assert_eq!(junk_stack_quantity((0, 1), 3, Some(&poor), false), 3);
    assert_eq!(junk_stack_quantity((4, 16), 7, Some(&poor), false), 7);
    let counts = [(0, 1, 3), (1, 2, 7)].map(|(bag, slot, quantity)| {
        junk_stack_quantity((bag, slot), quantity, Some(&poor), false)
    });
    assert_eq!(counts.into_iter().sum::<u64>(), 10);

    let unsellable = ItemInfo {
        sell_price: 0,
        ..poor.clone()
    };
    let common = ItemInfo {
        quality: 1,
        ..poor.clone()
    };
    assert_eq!(junk_stack_quantity((0, 1), 3, Some(&unsellable), false), 0);
    assert_eq!(junk_stack_quantity((0, 1), 3, Some(&common), false), 0);
    assert_eq!(junk_stack_quantity((0, 1), 3, None, false), 0);
    for quantity in [0, -3] {
        assert_eq!(junk_stack_quantity((0, 1), quantity, Some(&poor), false), 0);
    }
}

#[test]
fn merchant_junk_count_excludes_flagged_bags_banks_and_invalid_slots() {
    let poor = poor_item();
    assert_eq!(junk_stack_quantity((0, 1), 3, Some(&poor), true), 0);
    assert_eq!(junk_stack_quantity((4, 16), 3, Some(&poor), true), 0);
    for location in [
        (-4, 1),
        (-1, 1),
        (5, 1),
        (6, 1),
        (0, 0),
        (0, -1),
        (0, 17),
        (4, 17),
    ] {
        assert_eq!(
            junk_stack_quantity(location, 3, Some(&poor), false),
            0,
            "{location:?}"
        );
    }
}

#[test]
fn merchant_junk_count_live_catalog_api_and_bootstrap() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(EMPTY_READS).unwrap();
    env.exec(
        r#"
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 2840, 3)
        A_Admin.AddBagItem(4, 16, 2852, 1)
        A_Admin.AddBagItem(-1, 1, 2840, 5)
        A_Admin.AddBagItem(0, 17, 2840, 5)
        A_Admin.AddBagItem(0, 2, 4294967295, 8)
        assert(C_Item.GetItemInfo(2840) == 'Copper Bar')
        assert(select(3, C_Item.GetItemInfo(2840)) == 1)
        assert(C_MerchantFrame.GetNumJunkItems() == 0)
        C_Container.SetBagSlotFlag(0, 64, true)
        assert(C_Container.GetBackpackSellJunkDisabled())
        assert(C_MerchantFrame.GetNumJunkItems() == 0)
        C_Container.SetBagSlotFlag(0, 64, false)
        assert(not C_Container.GetBackpackSellJunkDisabled())
    "#,
    )
    .unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(EMPTY_READS).unwrap();
    assert_eq!(env.state().borrow().bag_items[&(0, 1)].stack_count, 3);
    let other = WowLuaEnv::new().unwrap();
    other.exec(EMPTY_READS).unwrap();
    assert!(!other.state().borrow().bag_items.contains_key(&(0, 17)));
}
