//! Round 6 `GetBuybackItem`: the original merchant buyback hover reaches the
//! getter, the index is one-based and truncating, and the live buyback count
//! bounds it.
#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;

fn buyback_fixture() -> WowLuaEnv {
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
    env
}

/// `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:717`
/// (`MerchantItemButton_OnEnter`) and the shared handler mapping at
/// `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:545`.
#[test]
fn tooltip_buyback_item_original_hover_selects_the_entry() {
    let env = buyback_fixture();
    env.exec(
        r#"
        assert(type(MerchantItemButton_OnEnter) == 'function', 'merchant consumer missing')
        MerchantFrame.selectedTab = 2
        local button = CreateFrame('Button', nil, UIParent)
        for id, name in ipairs({'Hearthstone', 'Copper Bar'}) do
            button:SetID(id)
            MerchantItemButton_OnEnter(button)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetBuybackItem', 'buyback getter missing')
            assert(data and data.type == Enum.TooltipDataType.Item, 'buyback data type')
            assert(#data.lines > 0, 'buyback item must compose the item lines')
            assert(data.lines[1].leftText == name, 'buyback name mismatch')
            assert(GameTooltip:GetLeftLine(1):GetText() == name)
        end
        "#,
    )
    .unwrap();
}

/// Round 6 §GetBuybackItem(a): the reader accepts any number in 0..=2^32-1,
/// subtracts 1.0 and truncates toward zero before the live-count bound test.
#[test]
fn tooltip_buyback_item_index_truncates_without_an_integral_test() {
    let env = buyback_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetBuybackItem
        assert(getter(1).lines[1].leftText == 'Hearthstone', 'index 1')
        assert(getter(1.9).lines[1].leftText == 'Hearthstone', '1.9 must truncate to entry 1')
        assert(getter(2.9).lines[1].leftText == 'Copper Bar', '2.9 must truncate to entry 2')
        assert(getter(0.5).lines[1].leftText == 'Hearthstone', '0.5 must truncate toward zero to entry 1')
        "#,
    )
    .unwrap();
}

/// Round 6 §GetBuybackItem(a,b): index 0, past-count indices and a 13th entry
/// take the same miss arm, which returns no values.
#[test]
fn tooltip_buyback_item_miss_arm_returns_no_values() {
    let env = buyback_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetBuybackItem
        assert(select('#', getter(0)) == 0, 'index 0 must miss')
        assert(select('#', getter(3)) == 0, 'past-count index must miss')
        assert(select('#', getter(13)) == 0, 'a 13th buyback entry must miss')
        assert(select('#', getter(4294967295)) == 0, 'top of the domain must miss')
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetBuybackItem(3)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'miss must not publish a tooltip')
        "#,
    )
    .unwrap();
}

/// Round 6 §GetBuybackItem(a): the argument error carries the getter's own usage
/// text, and an out-of-domain number carries the reader's range reason.
#[test]
fn tooltip_buyback_item_argument_errors_carry_native_text() {
    let env = buyback_fixture();
    env.exec(
        r#"
        local getter = C_TooltipInfo.GetBuybackItem
        local usage = 'Usage: local data = C_TooltipInfo.GetBuybackItem(index)'
        local range = 'outside of expected range 0 to 4294967295'
        local function error_text(...)
            local ok, err = pcall(getter, ...)
            assert(not ok, 'expected an argument error')
            return err
        end
        assert(error_text(-1):find(range, 1, true), 'negative index must report the range')
        assert(error_text(4294967296):find(range, 1, true), 'above the domain must report the range')
        assert(error_text(0/0):find(range, 1, true), 'non-finite must report the range')
        assert(error_text('1'):find(usage, 1, true), 'wrong type must report the usage text')
        assert(error_text():find(usage, 1, true), 'missing argument must report the usage text')
        "#,
    )
    .unwrap();
}
