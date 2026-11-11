#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::c_tooltip_info_pvp_talent::PvpTalentTooltipRecord;
use wow_ui_sim::lua_api::WowLuaEnv;

const TALENT_ID: i32 = 91_001;

fn record() -> PvpTalentTooltipRecord {
    PvpTalentTooltipRecord {
        spell_id: 116,
        ..PvpTalentTooltipRecord::default()
    }
}

fn env() -> WowLuaEnv {
    let env = crate::tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        function TestGetPvpTalent(...)
            GameTooltip:Hide()
            GameTooltip:ClearHandlerInfo()
            GameTooltip:ClearLines()
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetPvpTalent(...)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            assert(info and info.getterName == "GetPvpTalent",
                "the original accessor must dispatch C_TooltipInfo.GetPvpTalent")
            return GameTooltip:GetPrimaryTooltipData()
        end
        "#,
    )
    .expect("test wrapper should call the original TooltipDataHandler accessor");
    env
}

fn supply_record(env: &WowLuaEnv, talent_id: i32, record: PvpTalentTooltipRecord) {
    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .catalog
        .insert(talent_id, record);
}

#[test]
fn get_pvp_talent_is_registered_on_retail() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let getter_type: String = env
        .eval(
            r#"
            local namespace = rawget(_G, "C_TooltipInfo")
            return type(namespace and rawget(namespace, "GetPvpTalent"))
            "#,
        )
        .expect("GetPvpTalent registration probe should run");

    assert_eq!(getter_type, "function");
}

#[test]
fn game_tooltip_pvp_talent_uses_resolved_spell_and_replacement_line() {
    let env = env();
    let mut talent = record();
    talent.overrides_spell_id = 853;
    talent.category_position_mask = 1;
    talent.eligibility_by_position.insert(0, true);
    supply_record(&env, TALENT_ID, talent);
    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .groups
        .insert(0, [None; 4]);
    {
        let mut state = env.state().borrow_mut();
        state.pvp_talent_tooltip.resolved_spell_ids.insert(116, 133);
        // The replacement resolver excludes the primary candidate.
        state.pvp_talent_tooltip.resolved_spell_ids.insert(853, 116);
    }

    env.exec(
        r#"
        assert(type(GameTooltip.SetPvpTalent) == "function",
            "Retail TooltipDataHandler accessor must be selected")
        local data = TestGetPvpTalent(91001)
        local displayed = C_TooltipInfo.GetSpellByID(133)
        local replacement = C_TooltipInfo.GetSpellByID(853)
        local lines = data.lines
        assert(data.type == 1 and data.id == 133)
        assert(data.dataInstanceID ~= nil)
        assert(lines[1].leftText == displayed.lines[1].leftText)
        assert(#lines == #displayed.lines + 2,
            "replacement and talent annotation follow ordinary spell content")
        local line = lines[#lines - 1]
        assert(line.type == 0)
        assert(line.leftText == REPLACES_SPELL:format(replacement.lines[1].leftText))
        assert(lines[#lines].leftText == TALENT_TOOLTIP_ADDPREVIEWPOINT,
            "talent annotation follows REPLACES_SPELL")
        local first = TestGetPvpTalent(91001)
        local second = TestGetPvpTalent(91001)
        assert(second.dataInstanceID == first.dataInstanceID + 1,
            "populated tooltips use the shared incrementing instance counter")
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .resolved_spell_ids
        .clear();
    let mut no_override = record();
    no_override.spell_id = 116;
    supply_record(&env, TALENT_ID + 1, no_override);
    env.exec(
        r#"
        local data = TestGetPvpTalent(91002)
        local direct = C_TooltipInfo.GetSpellByID(116)
        assert(data.id == 116 and #data.lines == #direct.lines,
            "an empty resolver map means identity; zero OverridesSpellID adds no line")
        "#,
    )
    .unwrap();

    let mut suppressed = record();
    suppressed.overrides_spell_id = 115869;
    supply_record(&env, TALENT_ID + 2, suppressed);
    env.exec(
        r#"
        local data = TestGetPvpTalent(91003)
        local direct = C_TooltipInfo.GetSpellByID(133)
        assert(#data.lines == #direct.lines,
            "attribute mask 32768 suppresses the replacement line")
        "#,
    )
    .unwrap();
}

#[test]
fn game_tooltip_pvp_talent_local_annotation_predicates() {
    let env = env();
    let mut talent = record();
    talent.level_required = 75;
    talent.category_position_mask = 1;
    talent.eligibility_by_position.insert(0, true);
    supply_record(&env, TALENT_ID, talent);
    {
        let mut state = env.state().borrow_mut();
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
        state.player.level = 70;
        state.player.effective_level = 0;
    }

    env.exec(
        r#"
        assert(C_SpecializationInfo.GetActiveSpecGroup() == 1)
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == UNLOCKED_AT_LEVEL:format(75))
        assert(line.type == 0 and line.wrapText == nil,
            "early unlock line is plain and does not use the later wrap helper")
        local r, g, b = HOSTILE_STATUS_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();

    env.state().borrow_mut().player.level = 80;
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == TALENT_TOOLTIP_ADDPREVIEWPOINT)
        assert(line.type == 0 and line.wrapText == true)
        local r, g, b = FRIENDLY_STATUS_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)

        local explicitGroup = TestGetPvpTalent(91001, false, 1, 0)
        assert(explicitGroup.lines[#explicitGroup.lines].leftText
            == TALENT_TOOLTIP_ADDPREVIEWPOINT,
            "one-based group 1 selects the active group")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        state
            .pvp_talent_tooltip
            .groups
            .insert(0, [None, None, None, Some(TALENT_ID)]);
        state.pvp_talent_tooltip.groups.insert(1, [None; 4]);
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local selected = TestGetPvpTalent(91001)
        assert(#selected.lines == #direct.lines,
            "a talent in any of the four active positions has no eligibility line")
        local otherGroup = TestGetPvpTalent(91001, false, 2, 0)
        assert(#otherGroup.lines == #direct.lines,
            "a non-current group suppresses local annotations")
        local unknownGroup = TestGetPvpTalent(91001, false, 99, 0)
        assert(#unknownGroup.lines == #direct.lines,
            "an out-of-range group has no local context")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
        state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap()
            .category_position_mask = 0;
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local masked = TestGetPvpTalent(91001)
        assert(#masked.lines == #direct.lines,
            "a category mask that excludes the position cannot preview")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.category_position_mask = 1;
        talent.eligibility_by_position.insert(1, true);
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local secondPosition = TestGetPvpTalent(91001, false, 1, 1)
        assert(#secondPosition.lines == #direct.lines,
            "talentIndex 1 stays position 1; it is not decremented")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.category_position_mask = 8;
        talent.eligibility_by_position.insert(3, true);
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001, false, 1, 3)
        assert(data.lines[#data.lines].leftText == TALENT_TOOLTIP_ADDPREVIEWPOINT,
            "the fourth local position is included")
        local direct = C_TooltipInfo.GetSpellByID(116)
        local pastEnd = TestGetPvpTalent(91001, false, 1, 4)
        assert(#pastEnd.lines == #direct.lines,
            "positions beyond zero through three have no local annotation")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.category_position_mask = 1;
        talent.level_required = 70;
        talent.eligibility_by_position.remove(&0);
        talent.eligibility_level_failed = false;
        state.player_xp.rest_state = 1;
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local unmodeled = TestGetPvpTalent(91001)
        assert(#unmodeled.lines == #direct.lines,
            "a missing eligibility-helper result emits no guessed message")
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .catalog
        .get_mut(&TALENT_ID)
        .unwrap()
        .eligibility_by_position
        .insert(0, false);
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local resting = TestGetPvpTalent(91001)
        assert(#resting.lines == #direct.lines,
            "the allowed rest state does not add a rest error")
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.level_required = 70;
        talent.eligibility_level_failed = true;
        state.player_xp.rest_state = 0;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == UNLOCKED_AT_LEVEL:format(70)
            and line.type == 0 and line.wrapText == true,
            "a supplied later level failure uses the wrapped unlock line")
        "#,
    )
    .unwrap();
}

#[test]
fn game_tooltip_pvp_talent_failure_and_inspect_annotations() {
    let env = env();
    let mut talent = record();
    talent.category_position_mask = 1;
    talent.eligibility_by_position.insert(0, false);
    supply_record(&env, TALENT_ID, talent);
    {
        let mut state = env.state().borrow_mut();
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
        state.player.level = 80;
        state.player.in_combat = true;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == TALENT_TOOLTIP_ADD_COMBAT_ERROR)
        assert(line.type == 0 and line.wrapText == true)
        local r, g, b = HOSTILE_STATUS_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        state.player.in_combat = false;
        state.player.is_resting = false;
        state.player_xp.rest_state = 0;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == TALENT_TOOLTIP_ADD_REST_ERROR_PVP)
        assert(line.type == 0 and line.wrapText == true)
        local r, g, b = HOSTILE_STATUS_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.grant_rejected_by_item = true;
        talent.granted_item_id = Some(6948);
        state.player.in_combat = true;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local lines = data.lines
        assert(lines[#lines].leftText == TALENT_GRANTED_BY_ITEM:format("Hearthstone"))
        assert(lines[#lines - 1].type == 1 and lines[#lines - 2].type == 1,
            "item grants have two blank rows before the item message")
        assert(lines[#lines].wrapText == true)
        local r, g, b = REFUND_TOOLTIP_COLOR:GetRGB()
        assert(math.abs(lines[#lines].leftColor.r - r) < 0.01
            and math.abs(lines[#lines].leftColor.g - g) < 0.01
            and math.abs(lines[#lines].leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .catalog
        .get_mut(&TALENT_ID)
        .unwrap()
        .granted_by_essence = true;
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local lines = data.lines
        assert(lines[#lines].leftText == PVP_TALENT_GRANTED_BY_AZERITE_ESSENCE)
        assert(lines[#lines].type == 0 and lines[#lines].wrapText == true)
        assert(lines[#lines - 1].type == 1,
            "the essence message replaces the item message")
        local r, g, b = REFUND_TOOLTIP_COLOR:GetRGB()
        assert(math.abs(lines[#lines].leftColor.r - r) < 0.01
            and math.abs(lines[#lines].leftColor.g - g) < 0.01
            and math.abs(lines[#lines].leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let talent = state
            .pvp_talent_tooltip
            .catalog
            .get_mut(&TALENT_ID)
            .unwrap();
        talent.level_required = 75;
        talent.eligibility_by_position.insert(0, true);
        talent.eligibility_level_failed = false;
        state
            .pvp_talent_tooltip
            .groups
            .insert(0, [Some(TALENT_ID), None, None, None]);
        state.inspect_specialization_cache = Some(("inspect-guid".to_string(), 65));
        state
            .pvp_talent_tooltip
            .inspect_grant_item_ids
            .insert(("inspect-guid".to_string(), TALENT_ID), 6948);
        state.player.level = 1;
        state.player.in_combat = false;
        state.player.is_resting = false;
        state.player_xp.rest_state = 0;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001, true, 2, 1)
        local line = data.lines[#data.lines]
        assert(line.leftText == TALENT_GRANTED_BY_ITEM:format("Hearthstone"))
        assert(line.type == 0)
        for _, candidate in ipairs(data.lines) do
            assert(candidate.leftText ~= UNLOCKED_AT_LEVEL:format(75)
                and candidate.leftText ~= TALENT_TOOLTIP_ADDPREVIEWPOINT,
                "inspect bypasses local level and selection annotations")
        end
        assert(line.wrapText == true)
        local r, g, b = ITEM_LEGENDARY_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)
        "#,
    )
    .unwrap();
}
