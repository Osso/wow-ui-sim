//! Merchant extended costs, purchases and buybacks through the original
//! MerchantFrame item buttons and purchase confirmation.
#![cfg(feature = "client-retail")]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_game_ui() -> WowLuaEnv {
    let ui = common::blizzard_ui_dir();
    let env = common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

const MERCHANT_CASES: &str = r#"
local HEARTHSTONE, SCROLL, CRYSTALS = 6948, 35230, 3089
local function crystals() return C_CurrencyInfo.GetCurrencyInfo(CRYSTALS).quantity end
local function count(item) return C_Item.GetItemCount(item) end
assert(not pcall(A_Admin.SetMerchantOfferCost, 1, 10, {{HEARTHSTONE, 0}}), "cost count")
A_Admin.SetMoney(1000)
A_Admin.SetMerchantItems({HEARTHSTONE, SCROLL})
A_Admin.SetMerchantOfferCost(1, 100)
A_Admin.SetMerchantOfferCost(2, 250, {{HEARTHSTONE, 1}}, {{CRYSTALS, 500}})
A_Admin.OpenMerchant()
assert(MerchantFrame:IsShown(), "MERCHANT_SHOW opens the merchant")
MerchantFrame_Update()
assert(GetMerchantItemCostInfo(1) == 0 and GetMerchantItemCostInfo(2) == 2)
local texture, value, link, currencyName = GetMerchantItemCostItem(2, 1)
assert(value == 1 and link:find("|Hitem:6948:", 1, true) and currencyName == nil)
texture, value, link, currencyName = GetMerchantItemCostItem(2, 2)
assert(value == 500 and currencyName == "Resonance Crystals" and link:find("|Hcurrency:3089:", 1, true))
assert(GetMerchantItemCostItem(2, 3) == nil and select(2, GetMerchantItemCostItem(2, 3)) == 0)
assert(MerchantItem2AltCurrencyFrameItem1:IsShown() and MerchantItem2AltCurrencyFrameItem2:IsShown())
assert(not MerchantItem1AltCurrencyFrameItem1:IsShown(), "money-only offer")
assert(GetMerchantItemMaxStack(1) == 1 and GetMerchantItemMaxStack(2) == 200)
local hearthstones, startCrystals = count(HEARTHSTONE), crystals()
MerchantItem1ItemButton:Click("RightButton")
assert(GetMoney() == 900 and count(HEARTHSTONE) == hearthstones + 1, "money purchase")
MerchantItem2ItemButton:Click("RightButton")
-- Unrefundable offers (the default) take the nonrefundable confirmation.
local confirm = StaticPopup_FindVisible("CONFIRM_PURCHASE_NONREFUNDABLE_ITEM")
assert(confirm, "extended cost asks for confirmation")
confirm.ButtonContainer.Button1:Click()
assert(GetMoney() == 650 and count(SCROLL) == 1 and count(HEARTHSTONE) == hearthstones)
assert(crystals() == startCrystals - 500, "currency spent")
BuyMerchantItem(2)
assert(GetMoney() == 650 and count(SCROLL) == 1, "too few crystals changes nothing")
assert(not pcall(A_Admin.AddBuybackItem, SCROLL, 0, 10, true, false), "quantity")
A_Admin.AddBuybackItem(SCROLL, 5, 300, true, false)
A_Admin.AddBuybackItem(HEARTHSTONE, 1, 900, true, true)
MerchantFrameTab2:Click()
MerchantItem1ItemButton:Click("RightButton")
assert(GetMoney() == 350 and count(SCROLL) == 6 and GetNumBuybackItems() == 1, "buyback")
BuybackItem(1)
assert(GetMoney() == 350 and GetNumBuybackItems() == 1, "unaffordable buyback stays")
MerchantRepairItemButton:Click()
assert(not InRepairMode(), "no repair cursor at a plain merchant")
A_Admin.OpenMerchant(true)
MerchantRepairItemButton:Click()
assert(InRepairMode(), "ShowRepairCursor")
MerchantRepairItemButton:Click()
assert(not InRepairMode(), "HideRepairCursor leaves repair mode")
"#;

#[test]
fn merchant_frame_buys_supplied_offers_and_buybacks() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(MERCHANT_CASES).expect("original merchant consumers");
    }
}
