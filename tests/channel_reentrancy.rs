//! Replacement events must never clear newer callback-created operations.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn channel_stop_callback_replacement_survives_stale_ordinary_start() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local listener = CreateFrame('Frame')
        local restarted = false
        local normalStarts = 0
        listener:RegisterEvent('UNIT_SPELLCAST_CHANNEL_STOP')
        listener:RegisterEvent('UNIT_SPELLCAST_START')
        listener:SetScript('OnEvent', function(_, event)
            if event == 'UNIT_SPELLCAST_START' then normalStarts = normalStarts + 1 end
            if event == 'UNIT_SPELLCAST_CHANNEL_STOP' and not restarted then
                restarted = true
                A_Admin.StartEmpower(357208, 'Callback replacement', '', {1, 2}, 2)
            end
        end)
        A_Admin.StartChannel(15407, 'Old channel', '', 20)
        CastSpellByID(19750)
        assert(select(8, UnitChannelInfo('player')) == 357208, 'callback empower was canceled by stale ordinary START')
        assert(UnitCastingInfo('player') == nil and normalStarts == 0)
    "#).unwrap();
}

#[test]
fn channel_stop_callback_replaces_pending_specialization_without_leak() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.active_spec_index = 1;
    env.exec(r#"
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UNIT_SPELLCAST_CHANNEL_STOP')
        listener:SetScript('OnEvent', function()
            CastSpellByID(19750)
        end)
        A_Admin.StartChannel(15407, 'Old channel', '', 20)
        assert(C_SpecializationInfo.SetSpecialization(2))
        assert(UnitChannelInfo('player') == nil)
        assert(select(9, UnitCastingInfo('player')) == 19750)
    "#).unwrap();
    assert_eq!(env.state().borrow().player.pending_spec_change, None,
        "replaced specialization must not apply on the normal cast's completion");
}

#[test]
fn channel_start_from_interrupted_callback_prevents_superseded_channel_start() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local listener = CreateFrame('Frame')
        local incomingStarted = false
        listener:RegisterEvent('UNIT_SPELLCAST_INTERRUPTED')
        listener:RegisterEvent('UNIT_SPELLCAST_CHANNEL_START')
        listener:SetScript('OnEvent', function(_, event, _, _, spell)
            if event == 'UNIT_SPELLCAST_INTERRUPTED' then
                A_Admin.StartEmpower(357208, 'Callback', '', {1}, 1)
            elseif spell == 15407 then incomingStarted = true end
        end)
        CastSpellByID(19750)
        A_Admin.StartChannel(15407, 'Superseded', '', 20)
        assert(select(8, UnitChannelInfo('player')) == 357208)
        assert(UnitCastingInfo('player') == nil and not incomingStarted)
    "#).unwrap();
}
