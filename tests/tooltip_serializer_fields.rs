#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};
use wow_ui_sim::screen::ScreenKind;

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

fn selected_tooltip_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    assert_selected_tooltip_addons(&ui);
    let env = tooltip_full_env_helpers::setup_full_env();
    assert_selected_tooltip_sources(&env);
    env
}

fn assert_selected_tooltip_addons(ui: &std::path::Path) {
    let selected = discover_blizzard_addons_for_screen(ui, ScreenKind::Game);
    for addon in [
        "Blizzard_SharedXMLGame",
        "Blizzard_GameTooltip",
        "Blizzard_UIPanels_Game",
    ] {
        assert!(
            selected.iter().any(|(name, _)| name == addon),
            "Retail game-screen selection must include {addon}"
        );
    }
}

fn assert_selected_tooltip_sources(env: &WowLuaEnv) {
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
}

// INFERRED (simulator mapping; R88aw): generated item flags/classification and the stored-price path
// supply the verified fixture gates. VERIFIED (R88p.p1): eligible rows carry empty text, default color,
// `price`, and `maxPrice`; missing quantities keep -1.
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
        assert(sellPriceLine, 'explicitly eligible positive price must use a type-11 line')
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
    assert(sellPriceLine, "explicitly eligible positive price must publish a SellPrice line")
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
        .expect("the original item accessor should publish eligible SellPrice rows");
}

fn set_item_tooltip_sell_price_unit_prices(env: &WowLuaEnv, prices: &[(u32, u16, u32)]) {
    env.state()
        .borrow_mut()
        .item_tooltip_sell_price_unit_prices
        .extend(prices.iter().map(|&(item_id, level, price)| ((item_id, level), price)));
}

/// INFERRED (70391 export); R88aw confirms the first three base records match 70688.
#[test]
fn generated_sell_price_metadata_preserves_export_values() {
    for (item_id, expected) in [
        (184318, (Some(64), Some(8192), Some(16384), Some(15), Some(4))),
        (
            164374,
            (Some(67_174_464), Some(90_176), Some(16_384), Some(15), Some(4)),
        ),
        (163924, (Some(98_368), Some(8_256), Some(16_384), Some(15), Some(4))),
        (
            198058,
            (Some(0), Some(2_147_512_320), Some(0), Some(7), Some(18)),
        ),
        (211995, (Some(0), Some(8_192), Some(0), Some(4), Some(4))),
    ] {
        let item = wow_ui_sim::items::get_item(item_id).expect("catalog item");
        assert_eq!(
            (
                item.flags_0,
                item.flags_1,
                item.flags_3,
                item.item_class,
                item.item_subclass,
            ),
            expected,
            "SellPrice gates for item {item_id}"
        );
    }
}

// INFERRED (70391 export) with VERIFIED R88aw exclusion mask: preserve the high-bit word and reject its set mask.
#[test]
fn signed_item_flag_word_suppresses_sell_price() {
    const SELL_PRICE_EXCLUSION_MASK: u32 = 4096;
    let item = wow_ui_sim::items::get_item(198058).expect("catalog item");
    assert_eq!(
        (
            item.sell_price,
            item.inventory_type,
            item.item_class,
            item.item_subclass,
        ),
        (2_500, 0, Some(7), Some(18)),
    );
    assert!(item
        .flags_1
        .is_some_and(|flags| flags & SELL_PRICE_EXCLUSION_MASK != 0));
    let env = selected_tooltip_env();
    env.exec(r#"
        local data = C_TooltipInfo.GetItemByID(198058)
        for _, line in ipairs(data.lines) do
            assert(line.type ~= Enum.TooltipDataLineType.SellPrice)
        end
    "#).expect("the signed ItemSparse word's exclusion bit suppresses the row");
}

// VERIFIED (Retail 12.1.5.70688, R88aw): merchant and hidden item links set preview.
#[test]
fn merchant_and_hidden_item_links_apply_preview_gate() {
    let env = selected_tooltip_env();
    env.state().borrow_mut().merchant_items.push(184318);
    env.exec(r#"
        local function hasSellPrice(data)
            for _, line in ipairs(data.lines) do
                if line.type == Enum.TooltipDataLineType.SellPrice then
                    return true
                end
            end
            return false
        end
        assert(hasSellPrice(C_TooltipInfo.GetItemByID(184318)))
        assert(not hasSellPrice(C_TooltipInfo.GetHyperlink('item:184318', nil, nil, true)))
        assert(hasSellPrice(C_TooltipInfo.GetHyperlink('item:184318', nil, nil, false)))
        assert(not hasSellPrice(C_TooltipInfo.GetMerchantItem(1)))
    "#).expect("each producer must pass its native preview input");
}

// INFERRED (simulator mapping): an unknown calculated price needs an explicit host value.
#[test]
fn calculated_unit_price_requires_explicit_host_input() {
    let env = selected_tooltip_env();
    let check_no_sell_price = r#"
        local data = C_TooltipInfo.GetItemByID(211995)
        for _, line in ipairs(data.lines) do
            assert(line.type ~= Enum.TooltipDataLineType.SellPrice)
        end
    "#;
    env.exec(check_no_sell_price)
        .expect("an unknown calculated price must not imply a row");

    set_item_tooltip_sell_price_unit_prices(&env, &[(211995, 571, 4242)]);
    env.exec(r#"
        local data = C_TooltipInfo.GetItemByID(211995)
        local sellPriceLine
        for _, line in ipairs(data.lines) do
            if line.type == Enum.TooltipDataLineType.SellPrice then
                sellPriceLine = line
                break
            end
        end
        assert(sellPriceLine and sellPriceLine.price == 4242)
    "#).expect("an explicit calculated price should publish a type-11 line");
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

// INFERRED (simulator mapping): missing bag-instance status is unresolved and does not permit a row.
#[test]
fn bag_item_instance_keeps_prepared_max_price_at_minus_one() {
    let env = selected_tooltip_env();
    seed_bag_sell_price_instance(&env);
    assert_active_status_suppresses_sell_price(&env);
    allow_bag_instance_sell_price(&env);
    assert_instance_sell_price_uses_minus_one_max(&env);
    clear_bag_status_and_assert_sell_price_missing(&env);
}

fn seed_bag_sell_price_instance(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.bag_items.insert(
        (0, 1),
        BagItem {
            owned_identity: None,
            item_id: 184318,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
    sim.bag_item_tooltip_sell_price_statuses.insert(
        (0, 1),
        wow_ui_sim::c_api::item_tooltip::SellPriceStatusInput::StatusRecord {
            flags: 0,
            recorded_time: 1_000,
            client_clock: 1_000,
        },
    );
    sim.item_tooltip_sell_price_quantities
        .insert(184318, (1, 5));
}

fn assert_active_status_suppresses_sell_price(env: &WowLuaEnv) {
    env.exec(r#"
        local data = C_TooltipInfo.GetBagItem(0, 1)
        for _, line in ipairs(data.lines) do
            assert(line.type ~= Enum.TooltipDataLineType.SellPrice)
        end
    "#).expect("an active status record suppresses the SellPrice row");
}

fn allow_bag_instance_sell_price(env: &WowLuaEnv) {
    if let Some(wow_ui_sim::c_api::item_tooltip::SellPriceStatusInput::StatusRecord { flags, .. }) =
        env.state()
            .borrow_mut()
            .bag_item_tooltip_sell_price_statuses
            .get_mut(&(0, 1))
    {
        *flags = 1;
    }
}

fn assert_instance_sell_price_uses_minus_one_max(env: &WowLuaEnv) {
    env.exec(r#"
        local data = C_TooltipInfo.GetBagItem(0, 1)
        local sellPriceLine
        for _, line in ipairs(data.lines) do
            if line.type == Enum.TooltipDataLineType.SellPrice then
                sellPriceLine = line
                break
            end
        end
        assert(sellPriceLine and sellPriceLine.price == 1)
        assert(sellPriceLine.maxPrice == -1)
    "#).expect("item-instance rows bypass prepared maxPrice after status passes");
}

fn clear_bag_status_and_assert_sell_price_missing(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .bag_item_tooltip_sell_price_statuses
        .clear();
    env.exec(r#"
        local data = C_TooltipInfo.GetBagItem(0, 1)
        for _, line in ipairs(data.lines) do
            assert(line.type ~= Enum.TooltipDataLineType.SellPrice)
        end
    "#).expect("missing instance status input must not imply the row is eligible");
}

// INFERRED (simulator mapping): bag location represents the live instance passed to native status processing.
#[test]
fn no_status_record_marks_the_instance_before_later_sell_price_rows() {
    const ITEM_STATUS_FLAG: u32 = 4096;
    const HOST_UNIT_PRICE: u32 = 4242;
    let item_id = 7005;
    let item = wow_ui_sim::items::get_item(item_id).expect("catalog item");
    assert!(item.flags_0.is_some_and(|flags| flags & ITEM_STATUS_FLAG != 0));
    let env = selected_tooltip_env();
    seed_unrecorded_bag_item(&env, item_id, item.item_level, HOST_UNIT_PRICE);
    assert_first_unrecorded_request_suppresses_sell_price(&env);
    assert_instance_status_marker_set(&env);
    assert_later_unrecorded_request_publishes_price(&env);
}

fn seed_unrecorded_bag_item(
    env: &WowLuaEnv,
    item_id: u32,
    item_level: u16,
    host_unit_price: u32,
) {
    let mut sim = env.state().borrow_mut();
    sim.bag_items.insert(
        (0, 1),
        BagItem {
            owned_identity: None,
            item_id,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
    sim.item_tooltip_sell_price_unit_prices
        .insert((item_id, item_level), host_unit_price);
    sim.bag_item_tooltip_sell_price_statuses.insert(
        (0, 1),
        wow_ui_sim::c_api::item_tooltip::SellPriceStatusInput::NoStatusRecord {
            instance_marker_flags: 0,
        },
    );
}

fn assert_first_unrecorded_request_suppresses_sell_price(env: &WowLuaEnv) {
    env.exec(r#"
        local data = C_TooltipInfo.GetBagItem(0, 1)
        for _, line in ipairs(data.lines) do
            assert(line.type ~= Enum.TooltipDataLineType.SellPrice)
        end
    "#).expect("the first unrecorded instance request suppresses SellPrice");
}

fn assert_instance_status_marker_set(env: &WowLuaEnv) {
    const INSTANCE_STATUS_MARKER: u32 = 2;
    assert!(matches!(
        env.state()
            .borrow()
            .bag_item_tooltip_sell_price_statuses
            .get(&(0, 1)),
        Some(wow_ui_sim::c_api::item_tooltip::SellPriceStatusInput::NoStatusRecord {
            instance_marker_flags: INSTANCE_STATUS_MARKER
        })
    ));
}

fn assert_later_unrecorded_request_publishes_price(env: &WowLuaEnv) {
    env.exec(r#"
        local data = C_TooltipInfo.GetBagItem(0, 1)
        local sellPriceLine
        for _, line in ipairs(data.lines) do
            if line.type == Enum.TooltipDataLineType.SellPrice then
                sellPriceLine = line
                break
            end
        end
        assert(sellPriceLine and sellPriceLine.price == 4242)
    "#).expect("a set instance marker no longer suppresses SellPrice");
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
