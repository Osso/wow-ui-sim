use crate::support::env;

#[test]
fn test_c_container_get_num_slots_backpack() {
    let env = env();
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(0)")
        .unwrap();
    assert_eq!(slots, 16);
}

#[test]
fn test_c_container_get_num_slots_other_bag() {
    let env = env();
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(1)")
        .unwrap();
    assert_eq!(slots, 16);
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(5)")
        .unwrap();
    assert_eq!(slots, 0);
}

#[test]
fn test_c_container_get_item_id_empty_slot() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemID(0, 10) == nil")
        .unwrap();
    assert!(is_nil, "Unpopulated slot should be nil");
}

#[test]
fn test_c_container_add_bag_item_via_admin() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 5)").unwrap();
    let id: i64 = env
        .eval("return C_Container.GetContainerItemID(0, 1)")
        .unwrap();
    assert_eq!(id, 6948);
}

#[test]
fn test_c_container_get_item_info_after_add() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 3, 6948, 10)").unwrap();
    let stack: i32 = env
        .eval("local info = C_Container.GetContainerItemInfo(0, 3); return info.stackCount")
        .unwrap();
    assert_eq!(stack, 10);
}

#[test]
fn test_c_container_get_item_info_empty_slot_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemInfo(0, 10) == nil")
        .unwrap();
    assert!(is_nil, "Empty slot should return nil");
}

#[test]
fn test_c_container_remove_bag_item() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 1)").unwrap();
    env.exec("A_Admin.RemoveBagItem(0, 1)").unwrap();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemID(0, 1) == nil")
        .unwrap();
    assert!(is_nil, "Removed item should be nil");
}

#[test]
fn test_c_container_clear_bags() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 1)").unwrap();
    env.exec("A_Admin.AddBagItem(0, 2, 6948, 1)").unwrap();
    env.exec("A_Admin.ClearBags()").unwrap();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemID(0, 1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_container_free_slots_tracks_items() {
    let env = env();
    let (free, _): (i32, i32) = env
        .eval("return C_Container.GetContainerNumFreeSlots(0)")
        .unwrap();
    assert_eq!(
        free, 12,
        "Backpack with 4 default items should have 12 free slots"
    );
    env.exec("A_Admin.AddBagItem(0, 10, 6948, 1)").unwrap();
    env.exec("A_Admin.AddBagItem(0, 11, 6948, 1)").unwrap();
    let (free2, _): (i32, i32) = env
        .eval("return C_Container.GetContainerNumFreeSlots(0)")
        .unwrap();
    assert_eq!(free2, 10, "Should have 10 free after adding 2 more items");
}

#[test]
fn test_c_container_total_free_bag_slots_tracks_equipped_bags() {
    let env = env();
    env.exec("A_Admin.ClearBags()").unwrap();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 1)").unwrap();

    let total_free: i32 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();

    assert_eq!(
        total_free, 79,
        "Backpack plus four equipped bag slots should expose 80 total slots"
    );
}

#[test]
fn test_c_container_free_slot_list_returns_table() {
    let env = env();
    let (kind, count, first_free): (String, i32, i32) = env
        .eval(
            r#"
            local slots = C_Container.GetContainerFreeSlots(0)
            local count = 0
            local firstFree
            for _, slot in ipairs(slots) do
                count = count + 1
                firstFree = firstFree or slot
            end
            return type(slots), count, firstFree
            "#,
        )
        .unwrap();

    assert_eq!(kind, "table");
    assert_eq!(count, 12);
    assert_eq!(first_free, 5);
}

#[test]
fn test_c_container_has_item() {
    let env = env();
    let has: bool = env
        .eval("return C_Container.HasContainerItem(0, 10)")
        .unwrap();
    assert!(!has, "Empty slot should return false");
    env.exec("A_Admin.AddBagItem(0, 10, 6948, 1)").unwrap();
    let has: bool = env
        .eval("return C_Container.HasContainerItem(0, 10)")
        .unwrap();
    assert!(has, "Occupied slot should return true");
}

#[test]
fn test_c_container_get_item_link_after_add() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 1)").unwrap();
    let link: String = env
        .eval("return C_Container.GetContainerItemLink(0, 1)")
        .unwrap();
    assert!(
        link.contains("Hearthstone"),
        "Link should contain item name"
    );
}

#[test]
fn test_c_container_item_link_matches_retail_parse_shape() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948, 1)").unwrap();
    let (item_id, item_context, bonus_count_is_safe): (String, String, bool) = env
        .eval(
            r#"
            local link = C_Container.GetContainerItemLink(0, 1)
            local _, _, itemID, _enchantID, _gemID1, _gemID2, _gemID3, _gemID4,
                _suffixID, _uniqueID, _linkLevel, _specializationID, _modifiersMask,
                itemContext, rest = strsplit(":", link, 15)
            local numBonusIDs = nil
            if rest ~= nil then
                numBonusIDs = strsplit(":", rest, 2)
            end
            return itemID, itemContext, numBonusIDs == "" or tonumber(numBonusIDs) ~= nil
            "#,
        )
        .unwrap();

    assert_eq!(item_id, "6948");
    assert_eq!(item_context, "");
    assert!(
        bonus_count_is_safe,
        "the first variable item-link field must be empty or numeric"
    );
}

#[test]
fn test_c_container_get_item_cooldown_zero_when_ready() {
    let env = env();
    let (start, duration, enable): (f64, f64, i32) = env
        .eval("return C_Container.GetItemCooldown(6948)")
        .unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1);
}

#[test]
fn test_c_container_get_container_item_cooldown_zero_when_ready() {
    let env = env();
    let item_id: i32 = env
        .eval("return C_Container.GetContainerItemID(0, 1)")
        .unwrap();
    assert_eq!(item_id, 6948);
    let (start, duration, enable): (f64, f64, i32) = env
        .eval("return C_Container.GetContainerItemCooldown(0, 1)")
        .unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1);
}

#[test]
fn test_c_container_backpack_disabled_flags_share_bag_slot_state() {
    let env = env();
    let states: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local autosortInitially = C_Container.GetBackpackAutosortDisabled()
            local sellJunkInitially = C_Container.GetBackpackSellJunkDisabled()

            C_Container.SetBagSlotFlag(0, 1, true)
            local autosortAfterSet = C_Container.GetBackpackAutosortDisabled()
            local sellJunkAfterAutosortSet = C_Container.GetBackpackSellJunkDisabled()

            C_Container.SetBagSlotFlag(0, 64, true)
            local autosortAfterBothSet = C_Container.GetBackpackAutosortDisabled()
            local sellJunkAfterSet = C_Container.GetBackpackSellJunkDisabled()

            C_Container.SetBagSlotFlag(0, 1, false)
            C_Container.SetBagSlotFlag(0, 64, false)
            return autosortInitially,
                   sellJunkInitially,
                   autosortAfterSet,
                   sellJunkAfterAutosortSet,
                   autosortAfterBothSet,
                   sellJunkAfterSet,
                   C_Container.GetBackpackAutosortDisabled(),
                   C_Container.GetBackpackSellJunkDisabled()
            "#,
        )
        .unwrap();

    assert_eq!(
        states,
        (false, false, true, false, true, true, false, false)
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn test_c_container_default_shims_have_safe_shapes() {
    let env = env();
    let (quest_ok, filtered, battle_pay, actions_ok): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local questInfo = C_Container.GetContainerItemQuestInfo(0, 1)
            local actionsOk =
                pcall(C_Container.UseContainerItem, 0, 1)
                and pcall(C_Container.PickupContainerItem, 0, 1)
                and pcall(C_Container.SplitContainerItem, 0, 1, 1)
            assert(C_Container.GetContainerItemID(1, 1) == nil)
            return type(questInfo) == "table"
                       and questInfo.isQuestItem == false
                       and questInfo.questID == nil
                       and questInfo.isActive == false,
                   C_Container.IsContainerFiltered(0),
                   C_Container.IsBattlePayItem(1, 1),
                   actionsOk
            "#,
        )
        .unwrap();

    assert!(quest_ok);
    assert!(!filtered);
    assert!(!battle_pay);
    assert!(actions_ok);
}

#[test]
fn test_c_item_upgrade_supplied_false_preserves_boolean_shape() {
    let env = env();
    if wow_ui_sim::client_profile::ACTIVE != wow_ui_sim::client_profile::ClientProfile::Wrath {
        env.exec(
            "A_Admin.AddBagItem(0, 1, 6948); A_Admin.SetItemUpgradeEligibility({bagID=0, slotIndex=1}, false)",
        )
        .unwrap();
    }
    let can_upgrade: bool = env
        .eval(
            r#"
            local location = { bagID = 0, slotIndex = 1 }
            return C_ItemUpgrade.CanUpgradeItem(location)
            "#,
        )
        .unwrap();

    assert!(!can_upgrade);
}

#[test]
fn test_c_container_default_stack_count_is_one() {
    let env = env();
    env.exec("A_Admin.AddBagItem(0, 1, 6948)").unwrap();
    let stack: i32 = env
        .eval("local info = C_Container.GetContainerItemInfo(0, 1); return info.stackCount")
        .unwrap();
    assert_eq!(stack, 1);
}

#[test]
fn test_c_item_get_item_cooldown_zero_when_ready() {
    let env = env();
    let (start, duration, enable): (f64, f64, bool) =
        env.eval("return C_Item.GetItemCooldown(6948)").unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert!(enable);
}

#[test]
fn test_default_backpack_hearthstone_slot1() {
    let env = env();
    let (id, stack, name): (i64, i32, String) = env
        .eval(
            r#"
            local info = C_Container.GetContainerItemInfo(0, 1)
            return info.itemID, info.stackCount, info.hyperlink
            "#,
        )
        .unwrap();
    assert_eq!(id, 6948);
    assert_eq!(stack, 1);
    assert!(name.contains("Hearthstone"));
}

#[test]
fn test_default_backpack_water_slot2() {
    let env = env();
    let (id, stack): (i64, i32) = env
        .eval(
            r#"
            local info = C_Container.GetContainerItemInfo(0, 2)
            return info.itemID, info.stackCount
            "#,
        )
        .unwrap();
    assert_eq!(id, 159);
    assert_eq!(stack, 5);
}

#[test]
fn test_default_backpack_bread_slot3() {
    let env = env();
    let (id, stack): (i64, i32) = env
        .eval(
            r#"
            local info = C_Container.GetContainerItemInfo(0, 3)
            return info.itemID, info.stackCount
            "#,
        )
        .unwrap();
    assert_eq!(id, 4540);
    assert_eq!(stack, 5);
}

#[test]
fn test_default_backpack_skinning_knife_slot4() {
    let env = env();
    let (id, stack, icon): (i64, i32, i64) = env
        .eval(
            r#"
            local info = C_Container.GetContainerItemInfo(0, 4)
            return info.itemID, info.stackCount, info.iconFileID
            "#,
        )
        .unwrap();
    assert_eq!(id, 7005);
    assert_eq!(stack, 1);
    assert_eq!(icon, 135637);
}

#[test]
fn test_c_container_set_bag_portrait_texture_uses_equipped_bag_icon() {
    let env = env();
    let texture: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local portrait = frame:CreateTexture(nil, "ARTWORK")
            C_Container.SetBagPortraitTexture(portrait, Enum.BagIndex.Bag_1)
            return portrait:GetTextureFilePath()
            "#,
        )
        .unwrap();
    assert_eq!(texture, "Interface\\Icons\\INV_Misc_Bag_08");
}
