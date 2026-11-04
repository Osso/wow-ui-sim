#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;

#[test]
fn original_buyback_setter_uses_the_selected_buyback_item() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().merchant_buyback_items = vec![
        BuybackItem {
            item_id: 6948,
            quantity: 1,
            price: 10,
            num_available: 1,
            is_usable: true,
            is_bound: false,
        },
        BuybackItem {
            item_id: 2840,
            quantity: 2,
            price: 20,
            num_available: 1,
            is_usable: true,
            is_bound: false,
        },
    ];
    env.exec(
        r#"
        for slot, name in ipairs({'Hearthstone', 'Copper Bar'}) do
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetBuybackItem(slot)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetBuybackItem', 'GetBuybackItem info missing')
            assert(data and data.lines[1].leftText == name)
            assert(GameTooltip:GetLeftLine(1):GetText() == name)
        end
        "#,
    )
    .unwrap();
}
