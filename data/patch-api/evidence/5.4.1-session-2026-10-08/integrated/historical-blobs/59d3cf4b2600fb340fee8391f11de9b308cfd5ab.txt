//! Tooltip-focused WoW API coverage tests extracted from wow_api.rs.

#[path = "wow_api_tooltip_helpers.rs"]
mod wow_api_tooltip_helpers;
#[path = "wow_api_tooltip_units.rs"]
mod wow_api_tooltip_units;

use super::*;
use wow_api_tooltip_helpers::{
    assert_item_tooltip_has_name_level_and_slot, seed_c_tooltip_info_test_state,
    tooltip_shape_checks_match_handler,
};

#[test]
fn test_c_tooltip_info_exists() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(C_TooltipInfo)").unwrap();
    assert_eq!(ty, "table");
}

#[test]
fn test_c_tooltip_info_shapes_match_tooltip_data_handler_expectations() {
    let env = WowLuaEnv::new().unwrap();
    seed_c_tooltip_info_test_state(&env);
    let all_shapes_match = tooltip_shape_checks_match_handler(&env);
    assert!(
        all_shapes_match,
        "every implemented C_TooltipInfo getter should return tooltip data tables that TooltipDataHandlerMixin can consume",
    );
}

#[test]
fn test_c_tooltip_info_get_trait_entry_returns_real_tooltip_lines() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local nodeIDs = C_Traits.GetTreeNodes(790)
            local entryID
            for _, nodeID in ipairs(nodeIDs) do
                local nodeInfo = C_Traits.GetNodeInfo(1, nodeID)
                if nodeInfo and nodeInfo.entryIDs and nodeInfo.entryIDs[1] then
                    entryID = nodeInfo.entryIDs[1]
                    break
                end
            end

            if not entryID then
                return false
            end

            local tooltip = C_TooltipInfo.GetTraitEntry(entryID, 1)
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Spell or not tooltip.lines then
                return false
            end

            local firstLine = tooltip.lines[1]
            return firstLine
                and firstLine.type == Enum.TooltipDataLineType.SpellName
                and type(firstLine.leftText) == "string"
                and firstLine.leftText ~= ""
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetTraitEntry should expose at least a real spell-name line",
    );
}

#[test]
fn test_c_tooltip_info_get_item_by_id_exposes_colored_name_and_binding_line() {
    let env = WowLuaEnv::new().unwrap();
    let has_full_item_tooltip_colors: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetItemByID(229181)
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Item or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            if not nameLine
                or nameLine.type ~= Enum.TooltipDataLineType.ItemName
                or nameLine.leftText ~= "Ordained Forge Maul"
                or type(nameLine.leftColor) ~= "table"
            then
                return false
            end

            local r, g, b = nameLine.leftColor:GetRGB()
            if math.abs(r - 0.64) > 0.01 or math.abs(g - 0.21) > 0.01 or math.abs(b - 0.93) > 0.01 then
                return false
            end

            local itemLevelLine = tooltip.lines[2]
            if not itemLevelLine
                or itemLevelLine.type ~= Enum.TooltipDataLineType.ItemLevel
                or type(itemLevelLine.leftText) ~= "string"
                or not string.find(itemLevelLine.leftText, "Item Level", 1, true)
                or type(itemLevelLine.leftColor) ~= "table"
            then
                return false
            end

            local hR, hG, hB = HIGHLIGHT_FONT_COLOR:GetRGB()
            local levelR, levelG, levelB = itemLevelLine.leftColor:GetRGB()
            if math.abs(levelR - hR) > 0.01 or math.abs(levelG - hG) > 0.01 or math.abs(levelB - hB) > 0.01 then
                return false
            end

            local bindingLine = tooltip.lines[4]
            if not bindingLine
                or bindingLine.type ~= Enum.TooltipDataLineType.ItemBinding
                or bindingLine.leftText ~= ITEM_BIND_ON_PICKUP
                or type(bindingLine.leftColor) ~= "table"
            then
                return false
            end

            local bindingR, bindingG, bindingB = bindingLine.leftColor:GetRGB()
            return math.abs(bindingR - hR) <= 0.01
                and math.abs(bindingG - hG) <= 0.01
                and math.abs(bindingB - hB) <= 0.01
            "#,
        )
        .unwrap();
    assert!(
        has_full_item_tooltip_colors,
        "C_TooltipInfo.GetItemByID should expose colored item-level and binding lines",
    );
}

#[test]
fn test_c_tooltip_info_get_item_by_id_returns_item_tooltip_lines() {
    let env = WowLuaEnv::new().unwrap();
    assert_item_tooltip_has_name_level_and_slot(
        &env,
        "C_TooltipInfo.GetItemByID(211992)",
        "Entombed Seraph's Greaves",
        "Legs",
        "C_TooltipInfo.GetItemByID should expose item, item-level, and equip-slot lines",
    );
}

#[test]
fn test_c_tooltip_info_get_item_by_id_includes_stat_lines_for_gear() {
    let env = WowLuaEnv::new().unwrap();
    let has_colored_stat_lines: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetItemByID(211995)
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Item or not tooltip.lines then
                return false
            end

            local foundStrength = false
            local foundStamina = false
            local foundGreenStatColor = false
            for _, line in ipairs(tooltip.lines) do
                local text = line.leftText
                if type(text) == "string" and string.sub(text, 1, 1) == "+" then
                    if string.find(text, ITEM_MOD_STRENGTH_SHORT, 1, true) then
                        foundStrength = true
                    end
                    if string.find(text, ITEM_MOD_STAMINA_SHORT, 1, true) then
                        foundStamina = true
                    end
                    if line.leftColor and type(line.leftColor.g) == "number" and type(line.leftColor.r) == "number" and line.leftColor.g > line.leftColor.r then
                        foundGreenStatColor = true
                    end
                end
            end

            return foundStrength and foundStamina and foundGreenStatColor
            "#,
        )
        .unwrap();

    assert!(
        has_colored_stat_lines,
        "C_TooltipInfo.GetItemByID should include green-colored stat lines for equippable armor",
    );
}

#[test]
fn test_c_tooltip_info_get_item_by_guid_returns_bag_item_tooltip_and_guid() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local guid = C_Item.GetItemGUID({ bagID = 0, slotIndex = 1 })
            local tooltip = C_TooltipInfo.GetItemByGUID(guid)
            if not guid or guid == "" or not tooltip or tooltip.type ~= Enum.TooltipDataType.Item or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local itemLevelLine = tooltip.lines[2]
            local bindingLine = tooltip.lines[3]

            return tooltip.guid == guid
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
                and itemLevelLine
                and itemLevelLine.type == Enum.TooltipDataLineType.ItemLevel
                and itemLevelLine.leftText == "Item Level 1"
                and bindingLine
                and bindingLine.type == Enum.TooltipDataLineType.ItemBinding
                and bindingLine.leftText == ITEM_BIND_ON_PICKUP
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetItemByGUID should return bag-item tooltip data and preserve the GUID",
    );
}

#[test]
fn test_c_tooltip_info_get_owned_item_by_id_returns_owned_bag_item_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetOwnedItemByID(6948)
            local missing = C_TooltipInfo.GetOwnedItemByID(229181)
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Item or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local itemLevelLine = tooltip.lines[2]
            local bindingLine = tooltip.lines[3]

            return missing
                and missing.type == Enum.TooltipDataType.Item
                and next(missing.lines) == nil
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
                and itemLevelLine
                and itemLevelLine.leftText == "Item Level 1"
                and bindingLine
                and bindingLine.leftText == ITEM_BIND_ON_PICKUP
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetOwnedItemByID should return tooltip data only for items in the player's bags",
    );
}

#[test]
fn test_c_tooltip_info_get_recipe_result_item_returns_output_item_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetRecipeResultItem(100005, {}, nil, nil, nil)
            local orderTooltip = C_TooltipInfo.GetRecipeResultItemForOrder(100005, {}, 1, nil, nil)
            local emptyTooltip = C_TooltipInfo.GetRecipeResultItem(100006, {}, nil, nil, nil)
            if not tooltip or not orderTooltip or not emptyTooltip then
                return false
            end

            local nameLine = tooltip.lines[1]
            local orderNameLine = orderTooltip.lines[1]
            local emptyLine = emptyTooltip.lines[1]

            return tooltip.type == Enum.TooltipDataType.Item
                and orderTooltip.type == Enum.TooltipDataType.Item
                and emptyTooltip.type == Enum.TooltipDataType.Item
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Ordained Forge Maul"
                and orderNameLine
                and orderNameLine.leftText == "Ordained Forge Maul"
                and emptyLine == nil
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "recipe result tooltip getters should expose the crafted output item and stay empty for zero-output recipes",
    );
}

#[test]
fn test_c_tooltip_info_get_minimap_mouseover_returns_zone_and_subzone_lines() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetMinimapMouseover()
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.MinimapMouseover or not tooltip.lines then
                return false
            end

            local zoneLine = tooltip.lines[1]
            local subZoneLine = tooltip.lines[2]
            return zoneLine
                and zoneLine.leftText == "Stormwind City"
                and subZoneLine
                and subZoneLine.leftText == "Trade District"
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetMinimapMouseover should return minimap tooltip data with the current zone and sub-zone",
    );
}

#[test]
fn test_c_tooltip_info_get_upgrade_item_returns_selected_upgrade_item_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            C_ItemUpgrade.SetItemUpgradeFromLocation({ bagID = 0, slotIndex = 1 })
            local tooltip = C_TooltipInfo.GetUpgradeItem()
            C_ItemUpgrade.ClearItemUpgrade()
            local emptyTooltip = C_TooltipInfo.GetUpgradeItem()

            if not tooltip or not emptyTooltip then
                return false
            end

            local nameLine = tooltip.lines[1]
            local itemLevelLine = tooltip.lines[2]
            local emptyLine = emptyTooltip.lines[1]
            return tooltip.type == Enum.TooltipDataType.Item
                and emptyTooltip.type == Enum.TooltipDataType.Item
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "Hearthstone"
                and itemLevelLine
                and itemLevelLine.leftText == "Item Level 1"
                and emptyLine == nil
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetUpgradeItem should return the currently selected item-upgrade tooltip and empty data when nothing is selected",
    );
}

#[test]
fn test_c_tooltip_info_get_inventory_item_returns_equipped_item_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    assert_item_tooltip_has_name_level_and_slot(
        &env,
        r#"C_TooltipInfo.GetInventoryItem("player", 1)"#,
        "Entombed Seraph's Casque",
        "Head",
        "C_TooltipInfo.GetInventoryItem should expose equipped item tooltip lines",
    );
}

#[test]
fn test_c_tooltip_info_get_hyperlink_returns_item_and_spell_tooltips() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local itemTooltip = C_TooltipInfo.GetHyperlink("|cff0070dd|Hitem:211992:0:0:0:0:0:0:0:0:0|h[Entombed Seraph's Greaves]|h|r")
            local spellTooltip = C_TooltipInfo.GetHyperlink(GetSpellLink(19750))
            if not itemTooltip or not spellTooltip then
                return false
            end

            local itemNameLine = itemTooltip.lines[1]
            local itemLevelLine = itemTooltip.lines[2]
            local spellNameLine = spellTooltip.lines[1]
            local spellCostLine = spellTooltip.lines[2]
            local spellCastLine = spellTooltip.lines[3]

            return itemTooltip.type == Enum.TooltipDataType.Item
                and spellTooltip.type == Enum.TooltipDataType.Spell
                and itemNameLine
                and itemNameLine.type == Enum.TooltipDataLineType.ItemName
                and itemNameLine.leftText == "Entombed Seraph's Greaves"
                and itemLevelLine
                and itemLevelLine.type == Enum.TooltipDataLineType.ItemLevel
                and itemLevelLine.leftText == "Item Level 571"
                and spellNameLine
                and spellNameLine.type == Enum.TooltipDataLineType.SpellName
                and spellNameLine.leftText == "Flash of Light"
                and spellCostLine
                and spellCostLine.leftText == "10% of Base MANA"
                and spellCastLine
                and spellCastLine.leftText == "1.5 sec cast"
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetHyperlink should dispatch item and spell hyperlinks to tooltip data",
    );
}

#[test]
fn test_c_tooltip_info_get_spell_by_id_returns_spell_tooltip_lines() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetSpellByID(19750)
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Spell or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local costLine = tooltip.lines[2]
            local castLine = tooltip.lines[3]
            local descriptionLine = tooltip.lines[4]

            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and nameLine.leftText == "Flash of Light"
                and costLine
                and costLine.leftText == "10% of Base MANA"
                and castLine
                and castLine.leftText == "1.5 sec cast"
                and descriptionLine
                and descriptionLine.type == Enum.TooltipDataLineType.SpellDescription
                and type(descriptionLine.leftText) == "string"
                and descriptionLine.leftText ~= ""
                and descriptionLine.wrapText == true
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetSpellByID should expose spell name, cost, cast time, and description lines",
    );
}

#[test]
fn test_c_tooltip_info_get_spell_by_id_resolves_hammer_of_justice_duration() {
    let env = WowLuaEnv::new().unwrap();
    let description: String = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetSpellByID(853)
            return tooltip.lines[3].leftText
            "#,
        )
        .unwrap();

    assert_eq!(description, "Stuns the target for 6.");
}

#[test]
fn test_c_tooltip_info_get_spell_book_item_returns_spell_tooltip_lines() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local expectedName = C_SpellBook.GetSpellBookItemName(1, Enum.SpellBookSpellBank.Player)
            local tooltip = C_TooltipInfo.GetSpellBookItem(1, Enum.SpellBookSpellBank.Player)
            if not expectedName or not tooltip or tooltip.type ~= Enum.TooltipDataType.Spell or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local castLine = tooltip.lines[3]

            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and nameLine.leftText == expectedName
                and castLine
                and type(castLine.leftText) == "string"
                and castLine.leftText ~= ""
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetSpellBookItem should expose spellbook spell tooltip lines",
    );
}

#[test]
fn test_c_tooltip_info_get_unit_buff_returns_aura_tooltip_lines() {
    let env = WowLuaEnv::new().unwrap();
    seed_c_tooltip_info_test_state(&env);

    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetUnitBuff("player", 1, "HELPFUL")
            local auraTooltip = C_TooltipInfo.GetUnitAura("player", 1, "HELPFUL")
            if not tooltip or not auraTooltip or tooltip.type ~= Enum.TooltipDataType.UnitAura or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local durationLine = tooltip.lines[2]
            local descriptionLine = tooltip.lines[3]
            local auraNameLine = auraTooltip.lines[1]
            local auraDurationLine = auraTooltip.lines[2]
            local auraDescriptionLine = auraTooltip.lines[3]

            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and nameLine.leftText == "Flash of Light"
                and durationLine
                and durationLine.leftText == "1 hr"
                and descriptionLine
                and descriptionLine.type == Enum.TooltipDataLineType.SpellDescription
                and type(descriptionLine.leftText) == "string"
                and descriptionLine.leftText ~= ""
                and descriptionLine.wrapText == true
                and auraTooltip.type == Enum.TooltipDataType.UnitAura
                and auraNameLine
                and auraNameLine.leftText == "Flash of Light"
                and auraDurationLine
                and auraDurationLine.leftText == "1 hr"
                and auraDescriptionLine
                and auraDescriptionLine.type == Enum.TooltipDataLineType.SpellDescription
                and type(auraDescriptionLine.leftText) == "string"
                and auraDescriptionLine.leftText ~= ""
                and auraDescriptionLine.wrapText == true
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetUnitBuff and GetUnitAura should expose aura name, duration, and description lines",
    );
}

#[test]
fn test_c_tooltip_info_get_unit_debuff_returns_empty_without_simulated_debuffs() {
    let env = WowLuaEnv::new().unwrap();
    let is_empty_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetUnitDebuff("player", 1, "HARMFUL")
            return tooltip
                and tooltip.type == Enum.TooltipDataType.UnitAura
                and tooltip.lines
                and next(tooltip.lines) == nil
            "#,
        )
        .unwrap();
    assert!(
        is_empty_tooltip,
        "C_TooltipInfo.GetUnitDebuff should return an empty UnitAura tooltip when no debuffs exist",
    );
}

#[test]
fn test_c_tooltip_info_get_unit_aura_instance_getters_return_expected_tooltips() {
    let env = WowLuaEnv::new().unwrap();
    seed_c_tooltip_info_test_state(&env);

    let has_expected_tooltips: bool = env
        .eval(
            r#"
            local buffTooltip = C_TooltipInfo.GetUnitBuffByAuraInstanceID("player", 1, "HELPFUL")
            local auraTooltip = C_TooltipInfo.GetUnitAuraByAuraInstanceID("player", 1)
            local debuffTooltip = C_TooltipInfo.GetUnitDebuffByAuraInstanceID("player", 1, "HARMFUL")

            local buffNameLine = buffTooltip and buffTooltip.lines and buffTooltip.lines[1]
            local buffDurationLine = buffTooltip and buffTooltip.lines and buffTooltip.lines[2]
            local auraNameLine = auraTooltip and auraTooltip.lines and auraTooltip.lines[1]
            local auraDurationLine = auraTooltip and auraTooltip.lines and auraTooltip.lines[2]
            local debuffIsEmpty = debuffTooltip
                and debuffTooltip.type == Enum.TooltipDataType.UnitAura
                and debuffTooltip.lines
                and next(debuffTooltip.lines) == nil

            return buffTooltip
                and buffTooltip.type == Enum.TooltipDataType.UnitAura
                and buffNameLine
                and buffNameLine.type == Enum.TooltipDataLineType.SpellName
                and buffNameLine.leftText == "Flash of Light"
                and buffDurationLine
                and buffDurationLine.leftText == "1 hr"
                and auraTooltip
                and auraTooltip.type == Enum.TooltipDataType.UnitAura
                and auraNameLine
                and auraNameLine.type == Enum.TooltipDataLineType.SpellName
                and auraNameLine.leftText == "Flash of Light"
                and auraDurationLine
                and auraDurationLine.leftText == "1 hr"
                and debuffIsEmpty
            "#,
        )
        .unwrap();
    assert!(
        has_expected_tooltips,
        "C_TooltipInfo aura-instance-ID getters should resolve helpful auras and return an empty debuff tooltip when no debuffs exist",
    );
}

#[test]
fn test_c_tooltip_info_get_unit_aura_returns_helpful_and_harmful_tooltips() {
    let env = WowLuaEnv::new().unwrap();
    seed_c_tooltip_info_test_state(&env);

    let has_expected_tooltips: bool = env
        .eval(
            r#"
            local helpfulTooltip = C_TooltipInfo.GetUnitAura("player", 1, "HELPFUL")
            local harmfulTooltip = C_TooltipInfo.GetUnitAura("player", 1, "HARMFUL")

            local helpfulNameLine = helpfulTooltip and helpfulTooltip.lines and helpfulTooltip.lines[1]
            local helpfulDurationLine = helpfulTooltip and helpfulTooltip.lines and helpfulTooltip.lines[2]
            local harmfulIsEmpty = harmfulTooltip
                and harmfulTooltip.type == Enum.TooltipDataType.UnitAura
                and harmfulTooltip.lines
                and next(harmfulTooltip.lines) == nil

            return helpfulTooltip
                and helpfulTooltip.type == Enum.TooltipDataType.UnitAura
                and helpfulNameLine
                and helpfulNameLine.type == Enum.TooltipDataLineType.SpellName
                and helpfulNameLine.leftText == "Flash of Light"
                and helpfulDurationLine
                and helpfulDurationLine.leftText == "1 hr"
                and harmfulIsEmpty
            "#,
        )
        .unwrap();
    assert!(
        has_expected_tooltips,
        "C_TooltipInfo.GetUnitAura should route HELPFUL to buffs and HARMFUL to debuffs",
    );
}
