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
        assertSource(MerchantItemButton_OnEnter,
            "Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua")
        "#,
    )
    .expect("selected Retail tooltip consumers should load from their original files");
    env
}

#[test]
fn selected_item_accessor_publishes_backed_sell_price_on_line_11() {
    let env = selected_tooltip_env();
    env.exec(
        r#"
        local itemID = 159
        local expectedPrice = select(11, C_Item.GetItemInfo(itemID))
        assert(type(expectedPrice) == "number" and expectedPrice > 0,
            "fixture item must have a supplied sell price")

        local displayedPrice
        GameTooltip_OnTooltipAddMoney = function(_, price)
            displayedPrice = price
        end
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetItemByID(itemID)

        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and data.type == Enum.TooltipDataType.Item,
            "the original SetItemByID accessor must publish item TooltipData")
        local sellPriceLine
        for _, line in ipairs(data.lines) do
            if line.type == Enum.TooltipDataLineType.SellPrice then
                sellPriceLine = line
                break
            end
        end
        assert(sellPriceLine, "backed sell price must use a type-11 line")
        assert(sellPriceLine.price == expectedPrice,
            "SellPrice line must use the existing item sell-price value")
        assert(displayedPrice == expectedPrice,
            "the original SellPrice line consumer must receive the published price")
        "#,
    )
    .expect("the original item accessor should publish the catalog sell price");
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
        assert(GameTooltip:ProcessInfo({tooltipData = data}))
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
