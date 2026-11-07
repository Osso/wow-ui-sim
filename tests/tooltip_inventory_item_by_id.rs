//! WM94 §1 + Round 6 §GetInventoryItemByID: the original `SetInventoryItemByID`
//! accessor selects an owned instance (backpack first, then the equipment range)
//! and composes the ordinary item tooltip from that instance.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    #[test]
    fn tooltip_inventory_item_by_id_original_accessor_selects_owned_item() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.EquipItem(16, 231268)
            A_Admin.AddBagItem(0, 1, 1728, 1)
            A_Admin.AddBagItem(1, 1, 128934, 1)
            for _, case in ipairs({{1728, 'Teebu\'s Blazing Longsword'}, {128934, 'Shield of the Sea Queen'}}) do
                local item_id, name = case[1], case[2]
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetInventoryItemByID(item_id)
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetInventoryItemByID', 'inventory-item accessor missing')
                assert(data and data.type == Enum.TooltipDataType.Item and data.id == item_id)
                assert(#data.lines > 0, 'owned instance must compose the ordinary item lines')
                assert(data.lines[1].leftText == name, 'owned instance must keep its catalog identity')
                assert(GameTooltip:GetLeftLine(1):GetText() == name)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_inventory_item_by_id_searches_equipment_after_bags() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        // Equipped copies resolve too: the search reaches the equipment range.
        env.exec(
            r#"
            A_Admin.EquipItem(16, 231268)
            A_Admin.AddBagItem(0, 1, 1728, 1)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetInventoryItemByID(231268)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.id == 231268, 'equipped-only item must resolve')
            assert(data.lines[1].leftText == 'Blastfurious Machete')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_inventory_item_by_id_missing_owned_item_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.AddBagItem(0, 1, 1728, 1)
            A_Admin.EquipItem(16, 231268)
            assert(select('#', C_TooltipInfo.GetInventoryItemByID(6948)) == 0,
                'unowned item must take the missing route')
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetInventoryItemByID(6948)
            assert(GameTooltip:GetPrimaryTooltipData() == nil, 'unowned item must not show a tooltip')
            "#,
        )
        .unwrap();
    }

    // Round 6 §GetInventoryItemByID(c): the request carries the found item's own
    // GUID in both GUID inputs, and the shared item producer publishes it as the
    // top-level `guid` field of the resolved instance.
    #[test]
    fn tooltip_inventory_item_by_id_publishes_the_instance_guid() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.AddBagItem(0, 3, 1728, 1)
            local expected = C_Item.GetItemGUID({ bagID = 0, slotIndex = 3 })
            assert(expected == 'Item-0-3-1728', 'fixture GUID shape changed: ' .. tostring(expected))
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetInventoryItemByID(1728)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.guid == expected,
                'resolved instance must publish its own GUID, got ' .. tostring(data and data.guid))
            "#,
        )
        .unwrap();
    }

    // Round 6 §GetInventoryItemByID(a): one forward pass with the first
    // acceptable candidate winning; the backpack range precedes the equipment
    // range, so an equipped copy never outranks an earlier backpack copy.
    #[test]
    fn tooltip_inventory_item_by_id_prefers_the_backpack_over_the_equipment_range() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.EquipItem(16, 231268)
            A_Admin.AddBagItem(0, 2, 231268, 1)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetInventoryItemByID(231268)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.guid == 'Item-0-2-231268',
                'backpack copy must win over the equipped copy, got ' .. tostring(data and data.guid))
            "#,
        )
        .unwrap();
    }

    // Same forward pass: the backpack precedes the carried bags, and slots
    // inside one container are visited in ascending order.
    #[test]
    fn tooltip_inventory_item_by_id_prefers_the_backpack_and_the_lowest_slot() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.AddBagItem(1, 4, 1728, 1)
            A_Admin.AddBagItem(0, 6, 1728, 1)
            A_Admin.AddBagItem(0, 5, 1728, 1)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetInventoryItemByID(1728)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.guid == 'Item-0-5-1728',
                'backpack lowest slot must win, got ' .. tostring(data and data.guid))
            "#,
        )
        .unwrap();
    }

    // Round 6 §GetInventoryItemByID(a): the argument is read by the plain
    // signed reader. A negative id passes the reader with no semantic bound test
    // and then simply fails the inventory search; only the signed 32-bit range
    // is rejected.
    #[test]
    fn tooltip_inventory_item_by_id_reads_a_signed_identifier() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            A_Admin.AddBagItem(0, 1, 1728, 1)
            assert(select('#', C_TooltipInfo.GetInventoryItemByID(-1)) == 0,
                'a negative id passes the signed reader and misses')
            assert(select('#', C_TooltipInfo.GetInventoryItemByID(-2147483648)) == 0,
                'the signed range lower bound is accepted')
            assert(not pcall(C_TooltipInfo.GetInventoryItemByID, 2147483648),
                'the signed range upper bound is rejected')
            assert(not pcall(C_TooltipInfo.GetInventoryItemByID, 1.5),
                'the signed reader rejects a non-integer')
            "#,
        )
        .unwrap();
    }
}
