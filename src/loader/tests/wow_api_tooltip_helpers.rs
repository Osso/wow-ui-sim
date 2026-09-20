use crate::lua_api::WowLuaEnv;

pub(super) fn flash_of_light_aura() -> crate::lua_api::game_data::AuraInfo {
    crate::lua_api::game_data::AuraInfo {
        name: "Flash of Light".to_string(),
        spell_id: 19750,
        icon: 135987,
        duration: 3600.0,
        expiration_time: 3600.0,
        applications: 0,
        source_unit: "player".to_string(),
        is_helpful: true,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: 1,
    }
}

pub(super) fn lady_liadrin_target() -> crate::lua_api::game_data::TargetInfo {
    crate::lua_api::game_data::TargetInfo {
        unit_id: "target".to_string(),
        name: "Lady Liadrin".to_string(),
        class_index: 2,
        level: 80,
        health: 100_000,
        health_max: 100_000,
        power: 50_000,
        power_max: 100_000,
        power_type: 0,
        power_type_name: "MANA".to_string(),
        is_player: true,
        is_enemy: false,
        guid: "Player-0000-0000BEEF".to_string(),
        classification: "normal".to_string(),
        creature_type: "Blood Elf".to_string(),
        reaction: 5,
        interaction: Default::default(),
    }
}

pub(super) fn seed_c_tooltip_info_test_state(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.player.buffs = vec![flash_of_light_aura()];
    state.current_target = Some(lady_liadrin_target());
}

pub(super) fn assert_item_tooltip_has_name_level_and_slot(
    env: &WowLuaEnv,
    tooltip_expr: &str,
    expected_name: &str,
    expected_slot: &str,
    message: &str,
) {
    let has_real_tooltip: bool = env
        .eval(&format!(
            r#"
            local tooltip = {tooltip_expr}
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Item or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            local itemLevelLine = tooltip.lines[2]
            local equipSlotLine = tooltip.lines[3]

            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.ItemName
                and nameLine.leftText == "{expected_name}"
                and itemLevelLine
                and itemLevelLine.type == Enum.TooltipDataLineType.ItemLevel
                and itemLevelLine.leftText == "Item Level 571"
                and equipSlotLine
                and equipSlotLine.type == Enum.TooltipDataLineType.EquipSlot
                and equipSlotLine.leftText == "{expected_slot}"
            "#,
        ))
        .unwrap();
    assert!(has_real_tooltip, "{message}");
}

const TOOLTIP_DATA_SHAPE_LUA: &str = r#"
    local function colorShapeOk(color)
        return color == nil or (type(color) == "table" and type(color.GetRGB) == "function")
    end

    local function lineShapeOk(line)
        return type(line) == "table"
            and type(line.type) == "number"
            and (line.leftText == nil or type(line.leftText) == "string")
            and colorShapeOk(line.leftColor)
            and (line.wrapText == nil or type(line.wrapText) == "boolean")
            and (line.rightText == nil or type(line.rightText) == "string")
            and colorShapeOk(line.rightColor)
            and (line.leftOffset == nil or type(line.leftOffset) == "number")
    end

    local function tooltipShapeOk(tooltip, expectedType, allowEmpty)
        if type(tooltip) ~= "table" or tooltip.type ~= expectedType or type(tooltip.lines) ~= "table" then
            return false
        end

        local lineCount = 0
        for index, line in ipairs(tooltip.lines) do
            lineCount = index
            if not lineShapeOk(line) then
                return false
            end
        end

        return allowEmpty and lineCount == 0 or lineCount > 0
    end

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

    local checks = {
        tooltipShapeOk(C_TooltipInfo.GetTraitEntry(entryID, 1), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetAction(1), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetItemByID(211992), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetItemByGUID(C_Item.GetItemGUID({ bagID = 0, slotIndex = 1 })), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetOwnedItemByID(6948), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetRecipeResultItem(100005, {}, nil, nil, nil), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetRecipeResultItemForOrder(100005, {}, 1, nil, nil), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetMinimapMouseover(), Enum.TooltipDataType.MinimapMouseover, false),
        tooltipShapeOk((function()
            C_ItemUpgrade.SetItemUpgradeFromLocation({ bagID = 0, slotIndex = 1 })
            return C_TooltipInfo.GetUpgradeItem()
        end)(), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetInventoryItem("player", 1), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetSpellBookItem(1, Enum.SpellBookSpellBank.Player), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetSpellByID(19750), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetUnitBuff("player", 1, "HELPFUL"), Enum.TooltipDataType.UnitAura, false),
        tooltipShapeOk(C_TooltipInfo.GetUnitBuffByAuraInstanceID("player", 1, "HELPFUL"), Enum.TooltipDataType.UnitAura, false),
        tooltipShapeOk(C_TooltipInfo.GetUnitDebuff("player", 1, "HARMFUL"), Enum.TooltipDataType.UnitAura, true),
        tooltipShapeOk(C_TooltipInfo.GetUnitDebuffByAuraInstanceID("player", 1, "HARMFUL"), Enum.TooltipDataType.UnitAura, true),
        tooltipShapeOk(C_TooltipInfo.GetUnitAura("player", 1, "HELPFUL"), Enum.TooltipDataType.UnitAura, false),
        tooltipShapeOk(C_TooltipInfo.GetUnitAuraByAuraInstanceID("player", 1), Enum.TooltipDataType.UnitAura, false),
        tooltipShapeOk(C_TooltipInfo.GetHyperlink("|cff0070dd|Hitem:211992:0:0:0:0:0:0:0:0:0|h[Entombed Seraph's Greaves]|h|r"), Enum.TooltipDataType.Item, false),
        tooltipShapeOk(C_TooltipInfo.GetHyperlink(GetSpellLink(19750)), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetWorldCursor(), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetWorldLootObject("player"), Enum.TooltipDataType.Spell, false),
        tooltipShapeOk(C_TooltipInfo.GetUnit("player"), Enum.TooltipDataType.Unit, false),
    }

    for _, check in ipairs(checks) do
        if not check then
            return false
        end
    end

    return true
"#;

pub(super) fn tooltip_shape_checks_match_handler(env: &WowLuaEnv) -> bool {
    env.eval(TOOLTIP_DATA_SHAPE_LUA).unwrap()
}
