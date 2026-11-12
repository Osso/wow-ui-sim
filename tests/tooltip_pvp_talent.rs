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
fn retail_pvp_talent_catalog_uses_exported_rows_without_inventing_grants() {
    let env = WowLuaEnv::new().unwrap();
    let state = env.state().borrow();
    let catalog = &state.pvp_talent_tooltip.catalog;
    assert_eq!(catalog.len(), 407);
    let spellwarding = catalog.get(&5573).expect("exported talent 5573");
    assert_eq!(spellwarding.spell_id, 204018);
    assert_eq!(spellwarding.overrides_spell_id, 0);
    assert_eq!(spellwarding.level_required, 25);
    assert_eq!(spellwarding.category_position_mask, 15);
    assert!(spellwarding.eligibility_by_position.is_empty());
    assert!(!spellwarding.grant_rejected_by_item);
    assert_eq!(spellwarding.granted_item_id, None);
    assert!(!spellwarding.granted_by_essence);
    let guardian = catalog.get(&94).expect("exported talent 94");
    assert_eq!((guardian.spell_id, guardian.overrides_spell_id), (228049, 86659));
    assert_eq!(guardian.level_required, 40);
    assert!(!catalog.contains_key(&i32::MAX), "absent IDs stay catalog misses");
}

#[test]
fn pvp_slot_on_enter_uses_default_catalog_spell_and_unlock_lines() {
    let env = crate::common::panel_fixtures::setup_env();
    crate::common::supply_empty_forever_spellbook_outfits(&env);
    {
        let mut state = env.state().borrow_mut();
        state.player.level = 20;
        state.player.effective_level = 0;
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
    }
    env.exec(
        r#"
        assert(type(GameTooltip.SetPvpTalent) == "function", "original tooltip accessor is installed")
        C_SpecializationInfo.GetPvpTalentSlotInfo = function(slotIndex)
            return { enabled = true, slotIndex = slotIndex }
        end
        local originalGetPvpTalent = C_TooltipInfo.GetPvpTalent
        local getterArguments
        local getterCalls = 0
        C_TooltipInfo.GetPvpTalent = function(...)
            getterCalls = getterCalls + 1
            getterArguments = { ... }
            return originalGetPvpTalent(...)
        end
        assert(PlayerSpellsUtil and PlayerSpellsUtil.ToggleClassTalentFrame)
        PlayerSpellsUtil.ToggleClassTalentFrame()
        local talents = assert(PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame)
        local tray = assert(talents.PvPTalentSlotTray)
        local button = assert(tray.Slots[1])
        local cases = {
            { talent = 5573, spell = 204018, title = "Blessing of Spellwarding", level = 25 },
            { talent = 51, spell = 236180, title = "Den Mother", level = 30 },
        }
        for _, case in ipairs(cases) do
            -- Supply the already-selected UI value without calling LearnPvpTalent.
            button.predictedSetting.predictedValue = case.talent
            assert(button:GetSelectedTalent() == case.talent, "real PvP slot carries its selected talent")
            getterCalls = 0
            getterArguments = nil
            GameTooltip:Hide()
            GameTooltip:ClearHandlerInfo()
            GameTooltip:ClearLines()
            assert(button:GetScript("OnEnter"))(button)
            assert(GameTooltip:IsShown(), "original PvP slot hover shows the tooltip")
            assert(GameTooltip:NumLines() > 0, "selected PvP talent shows visible tooltip lines")
            assert(getterCalls == 1, "original OnEnter reaches the registered PvP getter once")
            assert(getterArguments[1] == case.talent and getterArguments[2] == false
                and getterArguments[4] == 1, "OnEnter supplies the selected talent and slot")
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.id == case.spell,
                "original hover displays the exported talent's spell line")
            assert(GameTooltip:NumLines() == #data.lines, "tooltip shows every published content line")
            for index, dataLine in ipairs(data.lines) do
                local visibleLine = _G["GameTooltipTextLeft" .. index]
                assert(visibleLine:GetText() == dataLine.leftText,
                    "visible tooltip line text matches its published content")
            end
            local info = GameTooltip:GetPrimaryTooltipInfo()
            assert(info and info.getterName == "GetPvpTalent", "original tooltip accessor metadata")
            local ordinary = C_TooltipInfo.GetSpellByID(case.spell)
            assert(#data.lines == #ordinary.lines + 1, "only the early unlock annotation applies")
            local line = data.lines[#data.lines]
            assert(line.leftText == UNLOCKED_AT_LEVEL:format(case.level),
                "only the verified early unlock annotation applies")
            assert(line.wrapText == nil, "the early unlock annotation is unwrapped")
            local r, g, b = HOSTILE_STATUS_COLOR:GetRGB()
            assert(math.abs(line.leftColor.r - r) < 0.01
                and math.abs(line.leftColor.g - g) < 0.01
                and math.abs(line.leftColor.b - b) < 0.01)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn default_pvp_talent_catalog_does_not_invent_eligibility_annotations() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.level = 80;
        state.player.effective_level = 0;
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
    }
    env.exec(
        r#"
        for _, case in ipairs({ {5573, 204018}, {51, 236180} }) do
            local data = C_TooltipInfo.GetPvpTalent(case[1])
            local ordinary = C_TooltipInfo.GetSpellByID(case[2])
            assert(data and data.id == case[2], "default catalog must supply the primary spell")
            assert(#data.lines == #ordinary.lines, "unknown eligibility and grant results add no lines")
        end
        "#,
    )
    .unwrap();
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
        .insert(853, u32::MAX);
    let mut unresolved_replacement = record();
    unresolved_replacement.overrides_spell_id = 853;
    supply_record(&env, TALENT_ID + 4, unresolved_replacement);
    env.exec(
        r#"
        local data = TestGetPvpTalent(91005)
        local direct = C_TooltipInfo.GetSpellByID(133)
        assert(#data.lines == #direct.lines,
            "a resolved replacement without a spell-name row adds no line")
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

    let mut missing_override = record();
    missing_override.overrides_spell_id = u32::MAX;
    supply_record(&env, TALENT_ID + 3, missing_override);
    env.exec(
        r#"
        local data = TestGetPvpTalent(91004)
        local direct = C_TooltipInfo.GetSpellByID(116)
        assert(#data.lines == #direct.lines,
            "an override without an original spell row adds no line")
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

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .groups
        .insert(1, [None; 4]);
    for position in 0..4 {
        let mut selected_group = [None; 4];
        selected_group[position] = Some(TALENT_ID);
        env.state()
            .borrow_mut()
            .pvp_talent_tooltip
            .groups
            .insert(0, selected_group);
        env.exec(
            r#"
            local direct = C_TooltipInfo.GetSpellByID(116)
            local selected = TestGetPvpTalent(91001)
            assert(#selected.lines == #direct.lines,
                "a talent in each active position has no eligibility line")
            "#,
        )
        .unwrap();
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
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
}

#[test]
fn game_tooltip_pvp_talent_missing_eligibility_and_level_error() {
    let env = env();
    let mut talent = record();
    talent.level_required = 70;
    talent.category_position_mask = 1;
    supply_record(&env, TALENT_ID, talent);
    {
        let mut state = env.state().borrow_mut();
        state.pvp_talent_tooltip.groups.insert(0, [None; 4]);
        state.player.level = 80;
        state.player.effective_level = 0;
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
        state.player_xp.rest_state = 0;
        state.player.is_resting = true;
    }
    env.exec(
        r#"
        local direct = C_TooltipInfo.GetSpellByID(116)
        local resting_player = TestGetPvpTalent(91001)
        assert(#resting_player.lines == #direct.lines,
            "the resting-player state also permits the talent change")
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
        talent.eligibility_level_failed = true;
        state.player.is_resting = false;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        local line = data.lines[#data.lines]
        assert(line.leftText == UNLOCKED_AT_LEVEL:format(70)
            and line.type == 0 and line.wrapText == true,
            "a supplied later level failure uses the wrapped unlock line")
        local r, g, b = HOSTILE_STATUS_COLOR:GetRGB()
        assert(math.abs(line.leftColor.r - r) < 0.01
            and math.abs(line.leftColor.g - g) < 0.01
            and math.abs(line.leftColor.b - b) < 0.01)
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
        talent.granted_item_id = Some(u32::MAX);
        state.player.in_combat = true;
    }
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001)
        assert(data.lines[#data.lines].leftText == TALENT_TOOLTIP_ADD_COMBAT_ERROR,
            "an item grant without an item row falls through to combat")
        "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .catalog
        .get_mut(&TALENT_ID)
        .unwrap()
        .granted_item_id = Some(6948);
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

    env.state()
        .borrow_mut()
        .pvp_talent_tooltip
        .inspect_grant_item_ids
        .remove(&("inspect-guid".to_string(), TALENT_ID));
    env.exec(
        r#"
        local data = TestGetPvpTalent(91001, true, 2, 1)
        local direct = C_TooltipInfo.GetSpellByID(116)
        assert(#data.lines == #direct.lines,
            "inspect mode adds no item line without a granting-item result")
        "#,
    )
    .unwrap();
}
