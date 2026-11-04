#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::BagItem;

#[test]
fn original_loot_hover_uses_the_selected_loot_item() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().loot_slots = vec![
        BagItem {
            owned_identity: None,
            item_id: 6948,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
        BagItem {
            owned_identity: None,
            item_id: 2840,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    ];
    env.exec(
        r#"
        assert(type(LootFrameItemElementMixin.OnEnter) == 'function')
        local button = CreateFrame('Button', nil, UIParent)
        button.HighlightNameFrame = CreateFrame('Frame', nil, button)
        function button:GetItemSlotType() return Enum.LootSlotType.Item end
        function button:GetSlotIndex() return self.slot end
        for slot, name in ipairs({'Hearthstone', 'Copper Bar'}) do
            button.slot = slot
            LootFrameItemElementMixin.OnEnter(button)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetLootItem')
            assert(data and data.lines[1].leftText == name)
            assert(GameTooltip:GetLeftLine(1):GetText() == name)
        end
        "#,
    )
    .unwrap();
}
