//! Simulator inputs drive channel/empower events; no injected spellcast events.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

fn listen(env: &WowLuaEnv) {
    env.exec(r#"
        channelEvents = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'UNIT_SPELLCAST_CHANNEL_START','UNIT_SPELLCAST_CHANNEL_UPDATE',
            'UNIT_SPELLCAST_CHANNEL_STOP','UNIT_SPELLCAST_EMPOWER_START',
            'UNIT_SPELLCAST_EMPOWER_UPDATE','UNIT_SPELLCAST_EMPOWER_STOP',
            'UNIT_SPELLCAST_START','UNIT_SPELLCAST_INTERRUPTED','UNIT_SPELLCAST_STOP'}) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event, ...)
            local row = {event=event, n=select('#', ...), ...}
            row.channel = select(11, UnitChannelInfo('player'))
            row.cast = select(10, UnitCastingInfo('player'))
            channelEvents[#channelEvents+1] = row
            if restartOn == event then restartOn=nil; CastSpellByID(19750) end
        end)
    "#).unwrap();
}

fn advance(env: &WowLuaEnv, seconds: u64) {
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(seconds);
    env.fire_on_update(0.016).unwrap();
}

#[test]
fn channel_inputs_update_complete_and_preserve_identity() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(r#"
        assert(type(A_Admin.StartChannel) == 'function', 'StartChannel missing')
        assert(not A_Admin.UpdateChannel(3)); assert(not A_Admin.StopChannel())
        A_Admin.StartChannel(15407, 'Synthetic channel', 'Interface/Icons/Spell_Shadow_SiphonMana', 2)
        local before = {UnitChannelInfo('player')}
        assert(#before == 11 and before[8] == 15407 and before[9] == false and before[10] == 0)
        assert(UnitCastingInfo('player') == nil)
        assert(A_Admin.UpdateChannel(4))
        local after = {UnitChannelInfo('player')}
        assert(after[4] == before[4] and after[11] == before[11])
        assert(math.abs(after[5] - after[4] - 4000) < 0.00001)
        assert(#channelEvents == 2)
        for i, event in ipairs({'UNIT_SPELLCAST_CHANNEL_START','UNIT_SPELLCAST_CHANNEL_UPDATE'}) do
            local row = channelEvents[i]
            assert(row.event == event and row.n == 4)
            assert(row[1] == 'player' and row[2] == 'Cast-Sim-' .. before[11])
            assert(row[3] == 15407 and row[4] == before[11] and row.channel == before[11])
        end
    "#).unwrap();
    advance(&env, 3);
    env.exec("assert(UnitChannelInfo('player') ~= nil and #channelEvents == 2)").unwrap();
    advance(&env, 2);
    env.exec(r#"
        assert(UnitChannelInfo('player') == nil)
        local stop = channelEvents[3]
        assert(stop.event == 'UNIT_SPELLCAST_CHANNEL_STOP' and stop.n == 5)
        assert(stop[2] == channelEvents[1][2] and stop[4] == nil)
        assert(stop[5] == channelEvents[1][4] and stop.channel == nil)
        assert(not A_Admin.StopChannel())
    "#).unwrap();
    advance(&env, 10);
    env.exec("assert(#channelEvents == 3)").unwrap();
}

#[test]
fn empower_inputs_expose_milliseconds_and_finish_after_hold() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(r#"
        assert(type(A_Admin.StartEmpower) == 'function', 'StartEmpower missing')
        A_Admin.StartEmpower(357208, 'Synthetic empower', 'Interface/Icons/Spell_Fire_Fire', {0.8, 1.2, 1.5}, 2)
        originalChannel = {UnitChannelInfo('player')}
        assert(originalChannel[9] and originalChannel[10] == 3)
        assert(math.abs(originalChannel[5]-originalChannel[4]-3500) < 0.00001)
        assert(GetUnitEmpowerStageDuration('player', 0) == 800)
        assert(GetUnitEmpowerStageDuration('player', 1) == 1200)
        assert(GetUnitEmpowerStageDuration('player', 2) == 1500)
        assert(GetUnitEmpowerHoldAtMaxTime('player') == 2000)
        assert(GetUnitEmpowerStageDuration('player', 3) == nil)
        assert(GetUnitEmpowerStageDuration('target', 0) == nil)
        assert(GetUnitEmpowerHoldAtMaxTime('target') == nil)
        assert(A_Admin.UpdateEmpower({1, 1, 2}, 3))
        assert(select(11, UnitChannelInfo('player')) == originalChannel[11])
        assert(GetUnitEmpowerStageDuration('player', 2) == 2000)
        assert(GetUnitEmpowerHoldAtMaxTime('player') == 3000)
        assert(channelEvents[1].event == 'UNIT_SPELLCAST_EMPOWER_START')
        assert(channelEvents[2].event == 'UNIT_SPELLCAST_EMPOWER_UPDATE')
        assert(channelEvents[1][4] == channelEvents[2][4])
    "#).unwrap();
    advance(&env, 5);
    env.exec("assert(UnitChannelInfo('player') ~= nil and #channelEvents == 2)").unwrap();
    advance(&env, 3);
    env.exec(r#"
        local stop = channelEvents[3]
        assert(stop.event == 'UNIT_SPELLCAST_EMPOWER_STOP' and stop.n == 6)
        assert(stop[4] == true and stop[5] == nil and stop[6] == originalChannel[11])
        assert(stop.channel == nil and UnitChannelInfo('player') == nil)
        assert(GetUnitEmpowerHoldAtMaxTime('player') == nil)
    "#).unwrap();
}

#[test]
fn channel_inputs_validate_atomically_and_isolate_modes() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(r#"
        A_Admin.StartChannel(15407, 'Channel', '', 5)
        local id = select(11, UnitChannelInfo('player'))
        for _, duration in ipairs({-1, 0, math.huge, 0/0, '5', 1e308}) do
            assert(not pcall(A_Admin.StartChannel, 15407, 'Channel', '', duration))
            assert(not pcall(A_Admin.UpdateChannel, duration))
        end
        for _, stages in ipairs({{}, {1, -1}, {1, math.huge}, {[2]=1}, {1, 1e308}, {1, '2'}}) do
            assert(not pcall(A_Admin.StartEmpower, 1, 'Empower', '', stages, 1))
        end
        assert(select(11, UnitChannelInfo('player')) == id and #channelEvents == 1)
        assert(not A_Admin.UpdateEmpower({1}, 1))
        A_Admin.StartEmpower(357208, 'Empower', '', {1, 1}, 1)
        assert(UnitCastingInfo('player') == nil)
        assert(channelEvents[2].event == 'UNIT_SPELLCAST_CHANNEL_STOP')
        assert(channelEvents[2][4] == UnitGUID('player'))
        assert(channelEvents[3].event == 'UNIT_SPELLCAST_EMPOWER_START')
        assert(not A_Admin.UpdateChannel(10))
        assert(not pcall(A_Admin.StopChannel, 'bad'))
        assert(A_Admin.StopChannel(true))
        local stop = channelEvents[#channelEvents]
        assert(stop.event == 'UNIT_SPELLCAST_EMPOWER_STOP' and stop[4] == true and stop[5] == nil)
        assert(not A_Admin.StopChannel())
        A_Admin.StartEmpower(357208, 'Empower', '', {1}, 2)
        assert(SpellStopCasting())
        stop = channelEvents[#channelEvents]
        assert(stop.event == 'UNIT_SPELLCAST_EMPOWER_STOP' and stop[4] == false)
        assert(stop[5] == UnitGUID('player') and UnitChannelInfo('player') == nil)
    "#).unwrap();
}

#[test]
fn channel_replacement_callbacks_preserve_new_modes_and_cancel_spec() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(r#"
        CastSpellByID(19750)
        A_Admin.StartChannel(15407, 'Channel', '', 3)
        assert(UnitCastingInfo('player') == nil)
        assert(channelEvents[2].event == 'UNIT_SPELLCAST_INTERRUPTED')
        assert(channelEvents[3].event == 'UNIT_SPELLCAST_STOP')
        assert(channelEvents[4].event == 'UNIT_SPELLCAST_CHANNEL_START')
        restartOn = 'UNIT_SPELLCAST_CHANNEL_STOP'
        assert(A_Admin.StopChannel())
        assert(UnitChannelInfo('player') == nil and UnitCastingInfo('player') ~= nil)
        A_Admin.StartEmpower(357208, 'Empower', '', {1}, 1)
        restartOn = 'UNIT_SPELLCAST_EMPOWER_STOP'
    "#).unwrap();
    advance(&env, 3);
    env.exec(r#"
        assert(UnitChannelInfo('player') == nil and UnitCastingInfo('player') ~= nil)
        A_Admin.StartChannel(15407, 'Channel', '', 10)
        CastSpellByID(82326)
        assert(UnitChannelInfo('player') == nil and select(9, UnitCastingInfo('player')) == 82326)
        A_Admin.StartChannel(15407, 'Channel', '', 10)
        A_Admin.SetCasting(19750, 'Fixture', '', 1)
        assert(UnitChannelInfo('player') == nil and UnitCastingInfo('player') ~= nil)
        assert(C_SpecializationInfo.SetSpecialization(2))
        A_Admin.StartChannel(15407, 'Channel', '', 10)
        assert(UnitCastingInfo('player') == nil)
    "#).unwrap();
    assert_eq!(env.state().borrow().player.pending_spec_change, None);
}
