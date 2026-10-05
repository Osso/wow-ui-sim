//! Concrete behavioral proof for the bounded 12.0.0 mixed publication closures.
#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_mixed_string_util_integer_formatting() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local cases = {
            {12.9, '12', '13'}, {12.5, '12', '13'}, {12.49, '12', '12'},
            {-12.9, '-13', '-13'}, {-12.5, '-13', '-12'}, {-0.1, '-1', '0'},
            {0, '0', '0'}, {1000000.1, '1000000', '1000000'},
        }
        for _, c in ipairs(cases) do
            assert(C_StringUtil.FloorToNearestString(c[1]) == c[2])
            assert(C_StringUtil.RoundToNearestString(c[1]) == c[3])
        end
        for _, f in ipairs({C_StringUtil.FloorToNearestString, C_StringUtil.RoundToNearestString}) do
            assert(not pcall(f, '12'))
            assert(not pcall(f, math.huge))
            assert(not pcall(f, 0/0))
        end
    "#).unwrap();
}

#[test]
fn p1200_mixed_event_publication() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local f = CreateFrame('Frame')
        for _, event in ipairs({
            'CHAT_MSG_ENCOUNTER_EVENT', 'COMBAT_LOG_APPLY_FILTER_SETTINGS',
            'COMBAT_LOG_EVENT_INTERNAL_UNFILTERED', 'COMBAT_LOG_REFILTER_ENTRIES',
            'TOOLTIP_SHOW_ITEM_COMPARISON',
        }) do
            assert(pcall(f.RegisterEvent, f, event), event)
            assert(f:IsEventRegistered(event), event)
            f:UnregisterEvent(event)
            assert(not f:IsEventRegistered(event), event)
        end
        -- Contradictory removed rows remain registerable for current consumers.
        for _, event in ipairs({'HOUSE_LEVEL_CHANGED', 'SETTINGS_LOADED',
            'TRANSMOG_OUTFITS_CHANGED', 'UNIT_SPELLCAST_SENT'}) do
            assert(pcall(f.RegisterEvent, f, event), event)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_mixed_cvar_publication() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'minimapTrackedInfov2', 'useCompactPartyFrames'}) do
            assert(C_CVar.GetCVarDefault(name) == '0')
            C_CVar.SetCVar(name, '1')
            assert(C_CVar.GetCVar(name) == '1')
            assert(C_CVar.GetCVarDefault(name) == '0')
        end
        -- Case-insensitive identity: the Npcs row is published in this same patch.
        assert(C_CVar.GetCVarDefault('nameplateShowFriendlyNPCs') == '0')
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_mixed_action_unregister() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ButtonA = CreateFrame('CheckButton')
        ButtonB = CreateFrame('CheckButton')
        C_ActionBar.RegisterActionUIButton(ButtonA, 3)
        C_ActionBar.RegisterActionUIButton(ButtonB, 4)
        C_ActionBar.UnregisterActionUIButton(ButtonA)
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.action_ui_buttons.len(), 1);
    assert_eq!(state.action_ui_buttons[0].1, 4);
    drop(state);
    env.exec("C_ActionBar.UnregisterActionUIButton(ButtonA); C_ActionBar.UnregisterActionUIButton(ButtonB)").unwrap();
    assert!(env.state().borrow().action_ui_buttons.is_empty());
}

#[test]
fn p1200_mixed_spell_loss_of_control_duration() {
    use wow_ui_sim::lua_api::LossOfControlInfo;
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().spell_loss_of_control.insert(
        19750,
        LossOfControlInfo {
            start_time: 123.0,
            duration: 27.0,
            mod_rate: 1.25,
            is_active: true,
            should_replace_normal_cooldown: false,
        },
    );
    env.exec(r#"
        local spell = C_Spell.GetSpellLossOfControlCooldownDuration('Flash of Light')
        local book = C_SpellBook.GetSpellBookItemLossOfControlCooldownDuration(5, 0)
        for _, d in ipairs({spell, book}) do
            assert(d:GetStartTime() == 123, 'start: ' .. tostring(d:GetStartTime()))
            assert(d:GetTotalDuration() == 27 / 1.25, 'duration: ' .. tostring(d:GetTotalDuration()))
            assert(d:GetModRate() == 1.25, 'rate: ' .. tostring(d:GetModRate()))
        end
        assert(spell ~= nil and book ~= nil, 'missing spell/book duration')
        assert(select('#', C_Spell.GetSpellLossOfControlCooldownDuration(999999)) == 0)
        assert(select('#', C_SpellBook.GetSpellBookItemLossOfControlCooldownDuration(5, 1)) == 0)
    "#).unwrap();
    env.state()
        .borrow_mut()
        .spell_loss_of_control
        .get_mut(&19750)
        .unwrap()
        .is_active = false;
    env.exec("assert(select('#', C_Spell.GetSpellLossOfControlCooldownDuration(19750)) == 0)")
        .unwrap();
}
