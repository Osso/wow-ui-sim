use super::*;

#[test]
fn test_c_tooltip_info_get_unit_returns_player_and_target_tooltips() {
    let env = WowLuaEnv::new().unwrap();
    seed_c_tooltip_info_test_state(&env);
    let (player_name, player_level, player_race, player_class, player_color) = {
        let state = env.state().borrow();
        let player = &state.player;
        let player_race = crate::lua_api::state::RACE_DATA
            .get(player.race_index)
            .map(|(name, _, _)| (*name).to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let player_class = crate::lua_api::game_data::CLASS_LABELS
            .get((player.class_index - 1).max(0) as usize)
            .copied()
            .unwrap_or("Unknown")
            .to_string();
        let player_color = match player.class_index {
            1 => (0.78, 0.61, 0.43),
            2 => (0.96, 0.55, 0.73),
            3 => (0.67, 0.83, 0.45),
            4 => (1.0, 0.96, 0.41),
            5 => (1.0, 1.0, 1.0),
            6 => (0.77, 0.12, 0.23),
            7 => (0.0, 0.44, 0.87),
            8 => (0.25, 0.78, 0.92),
            9 => (0.53, 0.53, 0.93),
            10 => (0.0, 1.0, 0.6),
            11 => (1.0, 0.49, 0.04),
            12 => (0.64, 0.19, 0.79),
            13 => (0.2, 0.58, 0.5),
            _ => (1.0, 1.0, 1.0),
        };
        (
            player.name.clone(),
            player.level,
            player_race,
            player_class,
            player_color,
        )
    };
    let has_expected_tooltips: bool = env
        .eval(&format!(
            r#"
            local playerTooltip = C_TooltipInfo.GetUnit("player")
            local targetTooltip = C_TooltipInfo.GetUnit("target")
            local missingTooltip = C_TooltipInfo.GetUnit("mouseover")

            local playerNameLine = playerTooltip and playerTooltip.lines and playerTooltip.lines[1]
            local playerLevelLine = playerTooltip and playerTooltip.lines and playerTooltip.lines[2]
            local playerRaceLine = playerTooltip and playerTooltip.lines and playerTooltip.lines[3]
            local playerClassLine = playerTooltip and playerTooltip.lines and playerTooltip.lines[4]
            local targetNameLine = targetTooltip and targetTooltip.lines and targetTooltip.lines[1]
            local targetLevelLine = targetTooltip and targetTooltip.lines and targetTooltip.lines[2]
            local targetRaceLine = targetTooltip and targetTooltip.lines and targetTooltip.lines[3]
            local targetClassLine = targetTooltip and targetTooltip.lines and targetTooltip.lines[4]

            local function approx(a, b)
                return math.abs(a - b) <= 0.01
            end

            local pr, pg, pb = playerNameLine.leftColor:GetRGB()
            local tr, tg, tb = targetNameLine.leftColor:GetRGB()

            return playerTooltip
                and playerTooltip.type == Enum.TooltipDataType.Unit
                and playerNameLine
                and playerNameLine.type == Enum.TooltipDataLineType.UnitName
                and playerNameLine.leftText == "{player_name}"
                and playerLevelLine
                and playerLevelLine.leftText == "Level {player_level}"
                and playerRaceLine
                and playerRaceLine.leftText == "{player_race}"
                and playerClassLine
                and playerClassLine.leftText == "{player_class}"
                and approx(pr, {player_r:.2})
                and approx(pg, {player_g:.2})
                and approx(pb, {player_b:.2})
                and targetTooltip
                and targetTooltip.type == Enum.TooltipDataType.Unit
                and targetNameLine
                and targetNameLine.type == Enum.TooltipDataLineType.UnitName
                and targetNameLine.leftText == "Lady Liadrin"
                and targetLevelLine
                and targetLevelLine.leftText == "Level 80"
                and targetRaceLine
                and targetRaceLine.leftText == "Blood Elf"
                and targetClassLine
                and targetClassLine.leftText == "Paladin"
                and approx(tr, 0.96)
                and approx(tg, 0.55)
                and approx(tb, 0.73)
                and missingTooltip
                and missingTooltip.type == Enum.TooltipDataType.Unit
                and missingTooltip.lines
                and next(missingTooltip.lines) == nil
            "#,
            player_r = player_color.0,
            player_g = player_color.1,
            player_b = player_color.2,
        ))
        .unwrap();
    assert!(
        has_expected_tooltips,
        "C_TooltipInfo.GetUnit should expose unit name, level, race, and class lines",
    );
}

#[test]
fn test_c_tooltip_info_get_world_cursor_returns_spell_tooltip_with_world_loot_fields() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetWorldCursor()
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Spell or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and type(nameLine.leftText) == "string"
                and nameLine.leftText ~= ""
                and type(tooltip.worldLootObjectInventoryType) == "number"
                and type(tooltip.id) == "number"
                and type(tooltip.worldLootObjectGUID) == "string"
                and tooltip.worldLootObjectGUID ~= ""
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetWorldCursor should expose a spell tooltip plus world-loot cursor metadata",
    );
}

#[test]
fn test_c_tooltip_info_get_world_loot_object_returns_spell_tooltip_with_world_loot_fields() {
    let env = WowLuaEnv::new().unwrap();
    let has_real_tooltip: bool = env
        .eval(
            r#"
            local tooltip = C_TooltipInfo.GetWorldLootObject("player")
            if not tooltip or tooltip.type ~= Enum.TooltipDataType.Spell or not tooltip.lines then
                return false
            end

            local nameLine = tooltip.lines[1]
            return nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and type(nameLine.leftText) == "string"
                and nameLine.leftText ~= ""
                and type(tooltip.worldLootObjectInventoryType) == "number"
                and type(tooltip.id) == "number"
                and type(tooltip.worldLootObjectGUID) == "string"
                and tooltip.worldLootObjectGUID ~= ""
            "#,
        )
        .unwrap();
    assert!(
        has_real_tooltip,
        "C_TooltipInfo.GetWorldLootObject should expose a spell tooltip plus world-loot object metadata",
    );
}

#[test]
fn test_c_tooltip_info_get_action_returns_spell_tooltip_for_spell_slots() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q2: keep the original cost/cast positions using final host helper outputs.
    {
        let mut sim = env.state().borrow_mut();
        let input = sim.spell_tooltip_inputs.entry(19750).or_default();
        input.cost = Some("host MANA cost".into());
        input.cast_ms = Some(1500);
    }
    let has_expected_tooltips: bool = env
        .eval(
            r#"
            local slotTooltip = C_TooltipInfo.GetAction(1)
            local emptyTooltip = C_TooltipInfo.GetAction(999)

            local nameLine = slotTooltip and slotTooltip.lines and slotTooltip.lines[1]
            local castTimeLine = slotTooltip and slotTooltip.lines and slotTooltip.lines[3]

            return slotTooltip
                and slotTooltip.type == Enum.TooltipDataType.Spell
                and nameLine
                and nameLine.type == Enum.TooltipDataLineType.SpellName
                and nameLine.leftText == "Flash of Light"
                and castTimeLine
                and castTimeLine.leftText == "1.5 sec cast"
                and emptyTooltip == nil
            "#,
        )
        .unwrap();
    assert!(
        has_expected_tooltips,
        "C_TooltipInfo.GetAction should delegate spell slots to spell tooltip data and return nil for empty slots",
    );
}
