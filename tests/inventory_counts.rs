//! Integration tests for `src/lua_api/globals/inventory_counts.rs`.

use wow_ui_sim::c_api::bag_info::BagInfo;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, CursorInfo, CursorItemOrigin};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── GetContainerNumFreeSlots ──────────────────────────────────────────────────

#[test]
fn get_container_num_free_slots_backpack_reports_free_and_bag_type() {
    let env = env();
    // The seeded backpack has some items already — free = 16 - occupied.
    let (free, bag_type, occupied): (i32, i32, i32) = env
        .eval(
            r#"
            local free, bagType = GetContainerNumFreeSlots(0)
            local totalSlots = C_Container.GetContainerNumSlots(0)
            return free, bagType, totalSlots - free
            "#,
        )
        .unwrap();
    assert_eq!(bag_type, 0, "normal backpack reports bagType 0");
    assert!(free >= 0);
    assert!(
        occupied > 0,
        "seeded backpack should have at least one occupied slot"
    );
}

#[test]
fn get_container_num_free_slots_uses_seeded_bag_capacity() {
    let env = env();
    let (free, bag_type): (i32, i32) = env.eval("return GetContainerNumFreeSlots(3)").unwrap();
    assert_eq!(free, 16);
    assert_eq!(bag_type, 0);
}

// ── Modeled container capacities ─────────────────────────────────────────────

fn captured_bag(num_slots: i32, inventory_slot: Option<i32>) -> BagInfo {
    BagInfo {
        num_slots,
        family: 4,
        name: Some("Captured reagent bag".to_string()),
        inventory_slot,
        item_id: Some(194019),
        hyperlink: Some("|Hitem:194019|h[Captured reagent bag]|h".to_string()),
    }
}

#[test]
fn container_default_bags_retain_seeded_capacity_name_and_inventory_mapping() {
    let env = env();
    let (backpack, bag, bank, bank_bag, reagent_bag, name, inventory): (
        i32,
        i32,
        i32,
        i32,
        i32,
        String,
        i32,
    ) = env
        .eval(
            "return C_Container.GetContainerNumSlots(0), C_Container.GetContainerNumSlots(3), \
         C_Container.GetContainerNumSlots(-1), C_Container.GetContainerNumSlots(-4), \
         C_Container.GetContainerNumSlots(5), C_Container.GetBagName(0), \
         C_Container.ContainerIDToInventoryID(3)",
        )
        .unwrap();
    assert_eq!(
        (backpack, bag, bank, bank_bag, reagent_bag),
        (16, 16, 28, 7, 0)
    );
    assert_eq!(name, "Backpack");
    assert_eq!(inventory, 23);
    let reagent_inventory: i32 = env
        .eval("return C_Container.ContainerIDToInventoryID(5)")
        .unwrap();
    assert_eq!(
        reagent_inventory, 25,
        "unequipped reagent bags still have an inventory-slot identity"
    );
}

#[test]
fn captured_reagent_bag_drives_all_capacity_queries_and_metadata() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.bag_info.insert(5, captured_bag(36, Some(77)));
        state.bag_items.insert(
            (5, 1),
            BagItem {
                item_id: 6948,
                stack_count: 1,
                hyperlink: None,
            },
        );
        state.bag_items.insert(
            (5, 36),
            BagItem {
                item_id: 2589,
                stack_count: 2,
                hyperlink: None,
            },
        );
    }
    let (capacity, free, legacy_free, family, first, last, slots, name, inventory): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        String,
        i32,
    ) = env
        .eval(
            "local slots = C_Container.GetContainerFreeSlots(5); \
         local free, family = C_Container.GetContainerNumFreeSlots(5); \
         return C_Container.GetContainerNumSlots(5), free, GetContainerNumFreeSlots(5), \
         family, slots[1], slots[#slots], #slots, C_Container.GetBagName(5), \
         C_Container.ContainerIDToInventoryID(5)",
        )
        .unwrap();
    assert_eq!((capacity, free, legacy_free, family), (36, 34, 34, 4));
    assert_eq!((first, last, slots), (2, 35, 34));
    assert_eq!((name.as_str(), inventory), ("Captured reagent bag", 77));
}

#[test]
fn captured_bag_item_link_preserves_bonus_context_across_container_queries() {
    let env = env();
    let captured_link = "|Hitem:6948::::::::70:::::1:12345|h[Captured Hearthstone]|h";
    env.state().borrow_mut().bag_items.insert(
        (5, 8),
        BagItem {
            item_id: 6948,
            stack_count: 1,
            hyperlink: Some(captured_link.to_string()),
        },
    );
    let (item_link, info_link): (String, String) = env
        .eval("return C_Container.GetContainerItemLink(5, 8), C_Container.GetContainerItemInfo(5, 8).hyperlink")
        .unwrap();
    assert_eq!(item_link, captured_link);
    assert_eq!(info_link, captured_link);
}

#[test]
fn zero_capacity_overrides_seeded_bag_without_stale_free_slots() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.bag_info.insert(3, captured_bag(0, None));
        state.bag_items.remove(&(3, 1));
    }
    let (capacity, free, legacy_free, free_list, name, inventory): (
        i32,
        i32,
        i32,
        i32,
        String,
        Option<i32>,
    ) = env
        .eval(
            "return C_Container.GetContainerNumSlots(3), \
         C_Container.GetContainerNumFreeSlots(3), GetContainerNumFreeSlots(3), \
         #C_Container.GetContainerFreeSlots(3), C_Container.GetBagName(3), \
         C_Container.ContainerIDToInventoryID(3)",
        )
        .unwrap();
    assert_eq!((capacity, free, legacy_free, free_list), (0, 0, 0, 0));
    assert_eq!((name.as_str(), inventory), ("Captured reagent bag", None));
}

#[test]
fn total_free_slots_use_captured_capacities_including_bag_five() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.bag_items.clear();
        state.bag_info.insert(2, captured_bag(0, None));
        state.bag_info.insert(5, captured_bag(36, Some(77)));
        state.bag_items.insert(
            (5, 36),
            BagItem {
                item_id: 6948,
                stack_count: 1,
                hyperlink: None,
            },
        );
    }
    let total: i32 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(total, 16 * 4 + 35);
}

#[test]
fn backpack_placement_obeys_captured_zero_and_expanded_capacity() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.bag_items.clear();
        state.bag_info.insert(0, captured_bag(0, Some(19)));
        state.cursor_item = Some(CursorInfo::Item {
            item_id: 6948,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    let placed: bool = env
        .eval("return PutItemInBackpack() and true or false")
        .unwrap();
    assert!(!placed);
    assert!(env.state().borrow().bag_items.is_empty());
    {
        let mut state = env.state().borrow_mut();
        state.bag_info.insert(0, captured_bag(36, Some(19)));
        for slot in 1..=16 {
            state.bag_items.insert(
                (0, slot),
                BagItem {
                    item_id: 2589,
                    stack_count: 1,
                    hyperlink: None,
                },
            );
        }
    }
    let placed: bool = env
        .eval("return PutItemInBackpack() and true or false")
        .unwrap();
    assert!(placed);
    assert_eq!(
        env.state()
            .borrow()
            .bag_items
            .get(&(0, 17))
            .map(|item| item.item_id),
        Some(6948)
    );
}

// ── GetNumLootItems ───────────────────────────────────────────────────────────

#[test]
fn get_num_loot_items_zero_when_no_loot_window() {
    let env = env();
    let n: i32 = env.eval("return GetNumLootItems()").unwrap();
    assert_eq!(n, 0);
}

#[test]
fn get_num_loot_items_counts_seeded_slots() {
    use wow_ui_sim::lua_api::state::BagItem;

    let env = env();
    env.state().borrow_mut().loot_slots = vec![
        BagItem {
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
        },
        BagItem {
            item_id: 19019,
            stack_count: 1,
            hyperlink: None,
        },
        BagItem {
            item_id: 36942,
            stack_count: 1,
            hyperlink: None,
        },
    ];
    let n: i32 = env.eval("return GetNumLootItems()").unwrap();
    assert_eq!(n, 3);
}

// ── GetMerchantNumItems ───────────────────────────────────────────────────────

#[test]
fn get_merchant_num_items_zero_when_merchant_closed() {
    let env = env();
    let n: i32 = env.eval("return GetMerchantNumItems()").unwrap();
    assert_eq!(n, 0);
}

#[test]
fn get_merchant_num_items_counts_seeded_items() {
    let env = env();
    env.state().borrow_mut().merchant_items = vec![159, 160, 4536, 4540];
    let n: i32 = env.eval("return GetMerchantNumItems()").unwrap();
    assert_eq!(n, 4);
}

// ── GetNumAuctionItems ────────────────────────────────────────────────────────

#[test]
fn get_num_auction_items_list_reports_browse_size() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.auction_browse_results.clear();
        state.auction_browse_items = vec![6948; 12];
    }
    let (n, total): (i32, i32) = env.eval(r#"return GetNumAuctionItems("list")"#).unwrap();
    assert_eq!(n, 12);
    assert_eq!(total, 12);
}

#[test]
fn get_num_auction_items_owner_and_bidder_always_zero() {
    let env = env();
    env.state().borrow_mut().auction_browse_items = vec![6948; 5];
    let (owner_n, owner_total): (i32, i32) =
        env.eval(r#"return GetNumAuctionItems("owner")"#).unwrap();
    let (bidder_n, bidder_total): (i32, i32) =
        env.eval(r#"return GetNumAuctionItems("bidder")"#).unwrap();
    assert_eq!((owner_n, owner_total), (0, 0));
    assert_eq!((bidder_n, bidder_total), (0, 0));
}

#[test]
fn get_num_auction_items_unknown_list_type_reports_zero() {
    let env = env();
    env.state().borrow_mut().auction_browse_items = vec![6948; 3];
    let (n, total): (i32, i32) = env.eval("return GetNumAuctionItems()").unwrap();
    assert_eq!((n, total), (0, 0));
}
