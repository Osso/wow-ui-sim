//! Integration tests for `src/lua_api/globals/inventory_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, CursorInfo, CursorItemOrigin, EquippedItem};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── PickupContainerItem ───────────────────────────────────────────────────────

#[test]
fn pickup_container_item_moves_bag_item_to_cursor() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.bag_items.insert(
            (0, 5),
            BagItem {
                item_id: 6948, // Hearthstone
                stack_count: 1,
                hyperlink: None,
            },
        );
    }
    env.exec("PickupContainerItem(0, 5)").unwrap();
    let st = env.state().borrow();
    assert!(!st.bag_items.contains_key(&(0, 5)), "item must leave bag");
    match &st.cursor_item {
        Some(CursorInfo::Item {
            item_id,
            stack_count,
            origin,
        }) => {
            assert_eq!(*item_id, 6948);
            assert_eq!(*stack_count, 1);
            assert!(matches!(origin, CursorItemOrigin::Bag { bag: 0, slot: 5 }));
        }
        other => panic!("cursor should carry the Hearthstone, got {other:?}"),
    }
}

#[test]
fn pickup_container_item_empty_slot_is_noop() {
    let env = env();
    env.exec("PickupContainerItem(0, 5)").unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
}

// ── PickupInventoryItem ───────────────────────────────────────────────────────

#[test]
fn pickup_inventory_item_moves_equipped_slot_to_cursor() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.equipped_items.insert(
            16, // main hand
            EquippedItem {
                item_id: 19019, // Thunderfury
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
    }
    env.exec("PickupInventoryItem(16)").unwrap();
    let st = env.state().borrow();
    assert!(!st.player.equipped_items.contains_key(&16));
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item {
            item_id: 19019,
            origin: CursorItemOrigin::Equipped { slot: 16 },
            ..
        })
    ));
}

// Cached EasyFishing 1.7.6 EquipBagPole uses these three calls, with no
// EquipCursorItem call between them. Exercise the legacy entry point too.
#[test]
fn fishing_cursor_transfer_preserves_both_items() {
    for pickup in ["C_Container.PickupContainerItem", "PickupContainerItem"] {
        let env = fishing_inventory(true);
        env.exec(&format!(
            "{pickup}(0, 5); PickupInventoryItem(16); \
             if CursorHasItem() then {pickup}(0, 5) end"
        ))
        .unwrap();
        let st = env.state().borrow();
        assert_eq!(
            st.player.equipped_items.get(&16).map(|item| item.item_id),
            Some(6256),
            "{pickup}"
        );
        let bag_item = st
            .bag_items
            .get(&(0, 5))
            .expect("displaced weapon returns to bag");
        assert_eq!((bag_item.item_id, bag_item.stack_count), (19019, 1));
        assert_eq!(st.bag_items.len(), 1);
        assert!(
            st.cursor_item.is_none(),
            "{pickup} must clear cursor after returning weapon"
        );
    }
}

#[test]
fn fishing_cursor_transfer_equips_empty_destination() {
    for pickup in ["C_Container.PickupContainerItem", "PickupContainerItem"] {
        let env = fishing_inventory(false);
        env.exec(&format!(
            "{pickup}(0, 5); PickupInventoryItem(16); \
             if CursorHasItem() then {pickup}(0, 5) end"
        ))
        .unwrap();
        let st = env.state().borrow();
        assert_eq!(
            st.player.equipped_items.get(&16).map(|item| item.item_id),
            Some(6256),
            "{pickup}"
        );
        assert!(st.bag_items.is_empty());
        assert!(st.cursor_item.is_none());
    }
}

#[test]
fn fishing_cursor_transfer_swaps_and_drops_bag_stacks() {
    for pickup in ["C_Container.PickupContainerItem", "PickupContainerItem"] {
        let env = fishing_inventory(false);
        env.state().borrow_mut().bag_items.insert(
            (0, 6),
            BagItem {
                item_id: 2589,
                stack_count: 7,
                hyperlink: None,
            },
        );
        env.exec(&format!("{pickup}(0, 5); {pickup}(0, 6)"))
            .unwrap();
        {
            let st = env.state().borrow();
            let placed = st
                .bag_items
                .get(&(0, 6))
                .expect("pole replaces cloth stack");
            assert_eq!((placed.item_id, placed.stack_count), (6256, 1), "{pickup}");
            assert!(matches!(
                st.cursor_item,
                Some(CursorInfo::Item {
                    item_id: 2589,
                    stack_count: 7,
                    origin: CursorItemOrigin::Bag { bag: 0, slot: 6 },
                })
            ));
        }
        env.exec(&format!("{pickup}(0, 5)")).unwrap();
        let st = env.state().borrow();
        let cloth = st
            .bag_items
            .get(&(0, 5))
            .expect("held stack drops into empty slot");
        assert_eq!((cloth.item_id, cloth.stack_count), (2589, 7));
        assert_eq!(st.bag_items.len(), 2);
        assert!(st.cursor_item.is_none());
    }
}

#[test]
fn fishing_cursor_transfer_namespace_preserves_pickup_and_empty_slot_behavior() {
    let env = fishing_inventory(false);
    env.exec("C_Container.PickupContainerItem(0, 6)").unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
    env.exec("C_Container.PickupContainerItem(0, 5)").unwrap();
    let st = env.state().borrow();
    assert!(st.bag_items.is_empty());
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item {
            item_id: 6256,
            stack_count: 1,
            origin: CursorItemOrigin::Bag { bag: 0, slot: 5 },
        })
    ));
}

fn fishing_inventory(equipped: bool) -> WowLuaEnv {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.bag_items.clear();
        st.bag_items.insert(
            (0, 5),
            BagItem {
                item_id: 6256,
                stack_count: 1,
                hyperlink: None,
            },
        );
        st.player.equipped_items.remove(&16);
        if equipped {
            st.player.equipped_items.insert(
                16,
                EquippedItem {
                    item_id: 19019,
                    enchant_id: 0,
                    gem_ids: [0; 3],
                },
            );
        }
    }
    env
}

// ── PickupMerchantItem ────────────────────────────────────────────────────────

#[test]
fn pickup_merchant_item_synthesizes_item_on_cursor() {
    let env = env();
    env.exec("PickupMerchantItem(3)").unwrap();
    let st = env.state().borrow();
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item {
            item_id: 100_003,
            origin: CursorItemOrigin::Merchant { index: 3 },
            ..
        })
    ));
}

#[test]
fn put_item_in_backpack_returns_false_without_item_cursor() {
    let env = env();

    let returned: bool = env
        .eval("return PutItemInBackpack() and true or false")
        .unwrap();
    assert!(
        !returned,
        "PutItemInBackpack should be falsey when there is nothing on the cursor"
    );
}

#[test]
fn put_item_in_backpack_moves_item_cursor_into_backpack() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 6948,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }

    let returned: bool = env
        .eval("return PutItemInBackpack() and true or false")
        .unwrap();
    assert!(
        returned,
        "PutItemInBackpack should report success for item cursors"
    );

    let st = env.state().borrow();
    assert!(st.cursor_item.is_none(), "cursor item should be consumed");
    assert_eq!(
        st.bag_items.get(&(0, 1)).map(|item| item.item_id),
        Some(6948),
        "first backpack slot should receive the moved item"
    );
}

#[test]
fn cursor_has_item_tracks_item_cursor_only() {
    let env = env();

    let empty: bool = env.eval("return CursorHasItem()").unwrap();
    assert!(!empty, "empty cursor should not report an item");

    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 100,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    let item: bool = env.eval("return CursorHasItem()").unwrap();
    assert!(item, "item cursor should report true");

    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Spell { spell_id: 12345 });
    }
    let spell: bool = env.eval("return CursorHasItem()").unwrap();
    assert!(!spell, "spell cursor should not report true");
}

// ── EquipCursorItem ───────────────────────────────────────────────────────────

#[test]
fn equip_cursor_item_writes_to_equipped_slot() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        // Default player has main-hand already populated; clear it so we can
        // observe the "empty slot" equip path.
        st.player.equipped_items.remove(&16);
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 19019,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    env.exec("EquipCursorItem(16)").unwrap();
    let st = env.state().borrow();
    assert_eq!(
        st.player
            .equipped_items
            .get(&16)
            .map(|e| e.item_id)
            .unwrap_or_default(),
        19019
    );
    assert!(
        st.cursor_item.is_none(),
        "cursor must clear after equip when slot was empty"
    );
}

#[test]
fn equip_cursor_item_swaps_with_existing_slot() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.equipped_items.insert(
            16,
            EquippedItem {
                item_id: 100,
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 200,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    env.exec("EquipCursorItem(16)").unwrap();
    let st = env.state().borrow();
    assert_eq!(
        st.player.equipped_items.get(&16).map(|e| e.item_id),
        Some(200)
    );
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item { item_id: 100, .. })
    ));
}

// ── DeleteCursorItem ──────────────────────────────────────────────────────────

#[test]
fn delete_cursor_item_clears_cursor() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 42,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    env.exec("DeleteCursorItem()").unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
}

#[test]
fn clear_cursor_clears_cursor() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 42,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    env.exec("ClearCursor()").unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
}

// ── PlaceAction ───────────────────────────────────────────────────────────────

#[test]
fn place_action_writes_spell_id_to_action_bar_slot() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Spell { spell_id: 12345 });
    }
    env.exec("PlaceAction(7)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.action_bars.get(&7).copied(), Some(12345));
    assert!(st.cursor_item.is_none());
}

#[test]
fn place_action_with_item_cursor_is_noop() {
    let env = env();
    // Snapshot the pre-call value of slot 7 so we can assert it is untouched.
    let before = env.state().borrow().action_bars.get(&7).copied();
    {
        let mut st = env.state().borrow_mut();
        st.cursor_item = Some(CursorInfo::Item {
            item_id: 999,
            stack_count: 1,
            origin: CursorItemOrigin::Unknown,
        });
    }
    env.exec("PlaceAction(7)").unwrap();
    let st = env.state().borrow();
    assert_eq!(
        st.action_bars.get(&7).copied(),
        before,
        "items cannot be placed on action bars"
    );
    assert!(
        st.cursor_item.is_some(),
        "cursor must keep the item when place is refused"
    );
}

// ── PickupBagFromSlot (alias) ─────────────────────────────────────────────────

#[test]
fn pickup_bag_from_slot_aliases_pickup_inventory_item() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.equipped_items.insert(
            20,
            EquippedItem {
                item_id: 5571, // small red pouch
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
    }
    env.exec("PickupBagFromSlot(20)").unwrap();
    let st = env.state().borrow();
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Item {
            item_id: 5571,
            origin: CursorItemOrigin::Equipped { slot: 20 },
            ..
        })
    ));
}

// ── PickupPlayerMoney / DropCursorMoney / GetCursorMoney ──────────────────────

#[test]
fn pickup_player_money_moves_copper_from_player_to_cursor() {
    let env = env();
    env.state().borrow_mut().player.money = 10_000;
    env.exec("PickupPlayerMoney(2500)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.player.money, 7_500);
    assert!(matches!(
        st.cursor_item,
        Some(CursorInfo::Money { copper: 2500 })
    ));
}

#[test]
fn pickup_player_money_above_balance_is_noop() {
    let env = env();
    env.state().borrow_mut().player.money = 100;
    env.exec("PickupPlayerMoney(500)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.player.money, 100);
    assert!(st.cursor_item.is_none());
}

#[test]
fn drop_cursor_money_returns_copper_to_player() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.money = 5_000;
        st.cursor_item = Some(CursorInfo::Money { copper: 1_500 });
    }
    env.exec("DropCursorMoney()").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.player.money, 6_500);
    assert!(st.cursor_item.is_none());
}

#[test]
fn drop_cursor_money_with_extra_arg_is_no_error() {
    // CoinPickupFrame.xml fires `<OnLoad function="DropCursorMoney"/>`,
    // which calls DropCursorMoney(self). The frame argument must be tolerated.
    let env = env();
    env.exec("DropCursorMoney(CoinPickupFrame)").unwrap();
    assert!(env.state().borrow().cursor_item.is_none());
}

#[test]
fn drop_cursor_money_leaves_non_money_cursor_alone() {
    let env = env();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        item_id: 6948,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 0, slot: 0 },
    });
    env.exec("DropCursorMoney()").unwrap();
    assert!(matches!(
        env.state().borrow().cursor_item,
        Some(CursorInfo::Item { item_id: 6948, .. })
    ));
}

#[test]
fn get_cursor_money_reads_cursor_payload() {
    let env = env();
    let zero: f64 = env.eval("return GetCursorMoney()").unwrap();
    assert_eq!(zero as u64, 0);

    env.state().borrow_mut().cursor_item = Some(CursorInfo::Money { copper: 4_242 });
    let copper: f64 = env.eval("return GetCursorMoney()").unwrap();
    assert_eq!(copper as u64, 4_242);
}

#[test]
fn get_cursor_info_reports_money_cursor() {
    let env = env();
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Money { copper: 777 });
    let (kind, copper): (String, f64) = env.eval("return GetCursorInfo()").unwrap();
    assert_eq!(kind, "money");
    assert_eq!(copper as u64, 777);
}
