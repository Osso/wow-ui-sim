//! Forever runtime consumer of the merchant buyback cursor global.
//!
//! The pinned Forever cache loads `[Family]\MerchantFrame.lua`
//! (`Blizzard_UIPanels_Game.toc:110`, `AllowLoadGameType mainline`; the
//! WowForever profile accepts `mainline`). `MerchantItemButton_OnEnter` calls
//! `ShowBuybackSellCursor(button:GetID())` at
//! `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:767`. No pinned
//! `Blizzard_APIDocumentationGenerated` file declares that name, so the
//! contract comes from the consumer call site plus the Retail 12.1.5/70688
//! native evidence in
//! `handoff/native-evidence-70688/WM86-merchant-cursor.md`.
//!
//! The Hearthstone fixture (item 6948) follows the sibling
//! `tests/tooltip_data_buyback_consumer.rs`.

#![cfg(feature = "client-wowforever")]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;

fn fixture() -> wow_ui_sim::lua_api::WowLuaEnv {
    let env = tooltip_full_env_helpers::setup_full_env();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_frame_open = true;
        state.player.money = 100;
        state.merchant_buyback_items = vec![BuybackItem {
            item_id: 6948,
            quantity: 1,
            price: 50,
            num_available: 1,
            is_usable: true,
            is_bound: false,
        }];
    }
    env
}

fn hover_first_buyback_button(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        assert(type(MerchantItemButton_OnEnter) == 'function',
            'Forever MerchantFrame must expose MerchantItemButton_OnEnter')
        MerchantFrame.selectedTab = 2
        local button = CreateFrame('Button', nil, UIParent)
        button:SetID(1)
        MerchantItemButton_OnEnter(button)
        "#,
    )
    .expect("the original Forever buyback hover path must run");
}

#[test]
fn forever_merchant_buyback_hover_selects_the_buy_cursor() {
    let env = fixture();
    hover_first_buyback_button(&env);
    assert_eq!(
        env.state().borrow().cursor_storage.presentation_display,
        3,
        "affordable buyback hover must select UICursorType Buy"
    );

    env.state().borrow_mut().player.money = 10;
    hover_first_buyback_button(&env);
    assert_eq!(
        env.state().borrow().cursor_storage.presentation_display,
        49,
        "unaffordable buyback hover must select UICursorType UnableBuy"
    );
}
