//! WM94 §1: the original `SetInventoryItemByID` accessor selects an owned
//! instance (bag first, then the equipment range) and composes the ordinary
//! item tooltip from it.

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
}
