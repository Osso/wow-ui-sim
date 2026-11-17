#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

fn selected_tooltip_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let selected = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for addon in ["Blizzard_SharedXMLGame", "Blizzard_UIPanels_Game"] {
        assert!(
            selected.iter().any(|(name, _)| name == addon),
            "Retail game-screen selection must include {addon}"
        );
    }

    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        local function assertSource(func, suffix)
            local source = debug.getinfo(func, "S").source:gsub("\\", "/")
            assert(source:find(suffix, 1, true), source)
        end
        assertSource(TooltipDataRules.FinalizeItemTooltip,
            "Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua")
        assertSource(TooltipDataRules.BattlePet,
            "Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua")
        assertSource(TooltipDataRules.SellPrice,
            "Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua")
        assertSource(GameTooltip.SetItemByID,
            "Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua")
        assertSource(GameTooltip_OnTooltipAddMoney,
            "Blizzard_GameTooltip/Mainline/GameTooltip.lua")
        assertSource(MerchantItemButton_OnEnter,
            "Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua")
        "#,
    )
    .expect("selected Retail tooltip consumers should load from their original files");
    env
}

// INFERRED: catalog `sell_price` stands in for the initial unit-price result.
// VERIFIED (R88p.p1): eligible type-11 rows carry empty text, a default color,
// `price`, and `maxPrice`; absent quantity inputs leave `maxPrice` at -1.
const RETAIL_SELL_PRICE_LUA: &str = r#"
local cases = {
    {itemID = 184318, price = select(11, C_Item.GetItemInfo(184318))},
    {itemID = 164374, price = select(11, C_Item.GetItemInfo(164374))},
    {itemID = 163924, price = select(11, C_Item.GetItemInfo(163924))},
}
assert(cases[1].price == 1 and cases[2].price == 50 and cases[3].price == 0,
    'fixtures must have two distinct positive prices and one zero price')
assert(cases[1].price ~= cases[2].price,
    'the two backed positive prices must differ')

local displayedPrice, displayedMaxPrice
GameTooltip.isShopping = false
GameTooltip_OnTooltipAddMoney = function(_, price, maxPrice)
    displayedPrice = price
    displayedMaxPrice = maxPrice
end
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

for _, case in ipairs(cases) do
    displayedPrice, displayedMaxPrice = nil, nil
    GameTooltip:SetItemByID(case.itemID)
    local data = GameTooltip:GetPrimaryTooltipData()
    assert(data and data.type == Enum.TooltipDataType.Item,
        'the original SetItemByID accessor must publish item TooltipData')
    local sellPriceLine
    for _, line in ipairs(data.lines) do
        if line.type == Enum.TooltipDataLineType.SellPrice then
            sellPriceLine = line
            break
        end
    end
    if case.price > 0 then
        assert(sellPriceLine, 'backed positive price must use a type-11 line')
        assert(sellPriceLine.price == case.price,
            'each SellPrice line must use this item’s backed price')
        assert(sellPriceLine.leftText == '',
            'VERIFIED SellPrice rows publish an empty leftText')
        assert(sellPriceLine.leftColor ~= nil,
            'VERIFIED SellPrice rows publish the resolved default leftColor')
        assert(sellPriceLine.maxPrice == -1,
            'without prepared quantity inputs maxPrice remains -1')
        assert(displayedPrice == case.price and displayedMaxPrice == -1,
            'the original SellPrice consumer must receive both row prices')
    else
        assert(sellPriceLine == nil,
            'an initial zero price must not create a type-11 line')
        assert(displayedPrice == nil and displayedMaxPrice == nil,
            'an initial zero price must not reach the money consumer')
    end
end
"#;

const RETAIL_PREPARED_SELL_PRICE_SETUP: &str = r#"
assert(C_Item.GetItemInfo(184318) and C_Item.GetItemInfo(164374),
    "prepared-quantity cases must use backed catalog items")
GameTooltip.isShopping = false
GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
__sellPriceConsumer = {}
GameTooltip_OnTooltipAddMoney = function(_, price, maxPrice)
    __sellPriceConsumer.price = price
    __sellPriceConsumer.maxPrice = maxPrice
end
function __assertRetailPreparedSellPrice(itemID, currentQuantity, preparedMaxQuantity)
    __sellPriceConsumer.price, __sellPriceConsumer.maxPrice = nil, nil
    GameTooltip:SetItemByID(itemID)
    local data = GameTooltip:GetPrimaryTooltipData()
    assert(data and data.type == Enum.TooltipDataType.Item,
        "SetItemByID must publish item TooltipData")
    local sellPriceLine
    for _, line in ipairs(data.lines) do
        if line.type == Enum.TooltipDataLineType.SellPrice then
            sellPriceLine = line
            break
        end
    end
    assert(sellPriceLine, "backed positive price must publish a SellPrice line")
    local expectedMaxPrice = -1
    if preparedMaxQuantity and preparedMaxQuantity > currentQuantity then
        expectedMaxPrice = (sellPriceLine.price * preparedMaxQuantity) % 4294967296
    end
    assert(__sellPriceConsumer.price == sellPriceLine.price
        and __sellPriceConsumer.maxPrice == expectedMaxPrice,
        "original SellPrice consumer must receive prepared maxPrice")
    assert(sellPriceLine.maxPrice == expectedMaxPrice,
        "SellPrice line maxPrice must match prepared quantity")
    return sellPriceLine.price, sellPriceLine.maxPrice
end
"#;

const RETAIL_PREPARED_SELL_PRICE_CASES: &str = r#"
local p1, m1 = __assertRetailPreparedSellPrice(184318, 2, 5)
local p2, m2 = __assertRetailPreparedSellPrice(164374, 3, 100000000)
assert(p1 ~= p2 and m1 ~= m2,
    "distinct backed items and prepared quantities must produce distinct prices")
"#;

#[test]
fn selected_item_accessor_publishes_backed_sell_price_on_line_11() {
    let env = selected_tooltip_env();
    env.exec(RETAIL_SELL_PRICE_LUA)
        .expect("the original item accessor should publish native SellPrice rows");
}

fn set_item_tooltip_sell_price_quantities(env: &WowLuaEnv, quantities: &[(u32, u32, u32)]) {
    let mut sim = env.state().borrow_mut();
    sim.item_tooltip_sell_price_quantities = quantities
        .iter()
        .map(|&(item_id, current, prepared_max)| (item_id, (current, prepared_max)))
        .collect();
}

#[test]
fn selected_item_accessor_uses_prepared_quantity_for_max_price() {
    let env = selected_tooltip_env();
    env.exec(RETAIL_PREPARED_SELL_PRICE_SETUP)
        .expect("the SellPrice consumer probe should load");

    set_item_tooltip_sell_price_quantities(&env, &[(184318, 2, 5), (164374, 3, 100_000_000)]);
    env.exec(RETAIL_PREPARED_SELL_PRICE_CASES)
        .expect("the original SellPrice consumer should receive prepared maxPrice");

    set_item_tooltip_sell_price_quantities(&env, &[]);
    env.exec("__assertRetailPreparedSellPrice(184318, nil, nil)")
        .expect("missing prepared quantities must preserve maxPrice -1");

    set_item_tooltip_sell_price_quantities(&env, &[(184318, 5, 5)]);
    env.exec("__assertRetailPreparedSellPrice(184318, 5, 5)")
        .expect("non-increasing prepared quantities must preserve maxPrice -1");
}

#[test]
fn repair_cost_consumer_reads_the_top_level_item_field() {
    let env = selected_tooltip_env();
    env.exec(
        r#"
        local observedRepairCost
        InRepairMode = function() return true end
        REPAIR_COST = "Repair Cost"
        GameTooltip_AddMoneyLine = function(_, amount)
            observedRepairCost = amount
        end
        local data = {
            type = Enum.TooltipDataType.Item,
            repairCost = 731,
            lines = {},
        }
        assert(GameTooltip:ProcessInfo({tooltipData = data}))
        assert(observedRepairCost == 731,
            "FinalizeItemTooltip must read top-level repairCost")
        "#,
    )
    .expect("the selected original item rule should read top-level repairCost");
}

#[test]
fn battle_pet_consumer_reads_top_level_tooltip_fields() {
    let env = selected_tooltip_env();
    env.exec(
        r#"
        local observed
        BattlePetToolTip_Show = function(...)
            observed = {...}
        end
        local data = {
            type = Enum.TooltipDataType.BattlePet,
            battlePetSpeciesID = 41,
            battlePetLevel = 7,
            battlePetBreedQuality = 3,
            battlePetMaxHealth = 521,
            battlePetPower = 47,
            battlePetSpeed = 39,
            battlePetName = "Probe Pet",
            lines = {},
        }
        GameTooltip:ProcessInfo({tooltipData = data})
        assert(type(observed) == "table", "BattlePet rule must consume the tooltip data")
        assert(#observed == 7, "BattlePet rule must forward seven top-level fields")
        assert(observed[1] == 41 and observed[2] == 7 and observed[3] == 3)
        assert(observed[4] == 521 and observed[5] == 47 and observed[6] == 39)
        assert(observed[7] == "Probe Pet")
        "#,
    )
    .expect("the selected original battle-pet rule should read top-level fields");
}

#[test]
fn sell_price_consumer_reads_price_fields_from_a_type_11_line() {
    let env = selected_tooltip_env();
    env.exec(
        r#"
        local observedPrice, observedMaxPrice
        GameTooltip.isShopping = false
        GameTooltip_OnTooltipAddMoney = function(_, price, maxPrice)
            observedPrice = price
            observedMaxPrice = maxPrice
        end
        local data = {
            type = Enum.TooltipDataType.Item,
            lines = {{
                type = Enum.TooltipDataLineType.SellPrice,
                price = 113,
                maxPrice = 227,
            }},
        }
        assert(GameTooltip:ProcessInfo({tooltipData = data}))
        assert(observedPrice == 113 and observedMaxPrice == 227,
            "the original SellPrice rule must read line-level price fields")
        "#,
    )
    .expect("the selected original SellPrice rule should consume line-level fields");
}
