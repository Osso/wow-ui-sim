//! Original TooltipDataHandler accessor over generated ItemChildEquipment rows.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::BagItem;

#[test]
fn tooltip_bag_child_original_accessor_selects_catalog_child_not_parent() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    for (slot, parent, child, equip_slot) in [(1, 127417, 1728, 16), (2, 127829, 127830, 17)] {
        let item = wow_ui_sim::items::get_item(child).expect("generated child has item metadata");
        env.state().borrow_mut().bag_items.insert(
            (0, slot),
            BagItem {
                owned_identity: None,
                item_id: parent,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
        env.exec(&format!(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetBagItemChild(0, {slot}, {equip_slot})
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetBagItemChild', 'bag-child accessor missing')
            assert(data and data.id == {child}, 'must select child, not parent or constant')
            assert(data.type == Enum.TooltipDataType.Item and #data.lines > 0)
            assert(type(data.dataInstanceID) == 'number')
            assert(data.lines[1].leftText == {name:?}, 'must use catalog item lines')
            assert(GameTooltip:GetLeftLine(1):GetText() == {name:?})
            local color = data.lines[1].leftColor
            assert(color and color.a == nil and type(color.GetRGB) == 'function')
            local r, g, b = color:GetRGB()
            for _, channel in ipairs({{r, g, b}}) do
                local byte = math.floor(channel * 255 + 0.5)
                assert(math.abs(channel - byte * 0.003921568859368563) < 0.0000001)
            end
            "#,
            name = item.name,
        ))
        .unwrap();
    }
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetBagItemChild(0, 1, 17)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'wrong equip slot must not select first child')
        GameTooltip:SetBagItemChild(0, 99, 16)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'missing bag item must not fabricate data')
        "#,
    )
    .unwrap();
}
