//! Modeled duration producers exercised through simulator cast/channel inputs.
#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        function absent(query, unit) assert(select('#', query(unit)) == 0) end
        function total(query, expected, ...)
            local d = query('player', ...)
            assert(d and math.abs(d:GetTotalDuration() - expected) < 0.00001)
            assert(math.abs(d:GetEndTime() - d:GetStartTime() - expected) < 0.00001)
            assert(d:GetModRate() == 1)
            return d
        end
    "#,
    )
    .unwrap();
    env
}

fn advance(env: &WowLuaEnv, seconds: u64) {
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(seconds);
    env.fire_on_update(0.016).unwrap();
}

#[test]
fn unit_cast_durations_resolve_self_interrupt_name() {
    let env = setup();
    env.state().borrow_mut().player.name = "Interrupt Actor".to_owned();
    env.exec(
        r#"
        local name, realm = UnitNameFromGUID(UnitGUID('player'))
        assert(name == 'Interrupt Actor' and name == UnitName('player'))
        assert(realm == 'SimRealm')
        assert(select('#', UnitNameFromGUID('unknown')) == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn unit_cast_durations_idle_and_nonplayer_return_no_results() {
    let env = setup();
    env.exec(r#"
        for _, query in ipairs({UnitCastingDuration, UnitChannelDuration, UnitEmpoweredChannelDuration}) do
            absent(query, 'player')
        end
        assert(type(UnitCastingDuration) == 'function')
        assert(type(UnitChannelDuration) == 'function')
        assert(type(UnitEmpoweredChannelDuration) == 'function')
        A_Admin.SetCasting(19750, 'Flash of Light', 'cast-icon', 20)
        for _, unit in ipairs({'target', 'party1', 'focus', 'unknown'}) do
            absent(UnitCastingDuration, unit)
        end
        A_Admin.StartEmpower(357208, 'Empower', 'cast-icon', {1, 2}, 4)
        for _, unit in ipairs({'target', 'party1', 'focus', 'unknown'}) do
            absent(UnitChannelDuration, unit)
            absent(UnitEmpoweredChannelDuration, unit)
        end
    "#).unwrap();
}

#[test]
fn unit_cast_durations_cast_tracks_time_replacement_cancel_and_expiry() {
    let env = setup();
    env.exec(
        r#"
        A_Admin.SetCasting(19750, 'Flash of Light', 'cast-icon', 20)
        castDuration = total(UnitCastingDuration, 20)
        local _, _, _, startMs, endMs = UnitCastingInfo('player')
        assert(math.abs(castDuration:GetStartTime() * 1000 - startMs) < 0.00001)
        assert(math.abs(castDuration:GetEndTime() * 1000 - endMs) < 0.00001)
        absent(UnitChannelDuration, 'player')
        absent(UnitEmpoweredChannelDuration, 'player')
    "#,
    )
    .unwrap();
    advance(&env, 3);
    env.exec(
        r#"
        assert(castDuration:GetElapsedDuration() >= 3)
        assert(castDuration:GetRemainingDuration() <= 17)
        A_Admin.SetCasting(19750, 'Replacement', 'cast-icon', 10)
        total(UnitCastingDuration, 10)
        assert(A_Admin.DelayCasting(2))
        total(UnitCastingDuration, 12)
        A_Admin.StopCasting()
        absent(UnitCastingDuration, 'player')
        A_Admin.SetCasting(19750, 'Completion', 'cast-icon', 1)
    "#,
    )
    .unwrap();
    advance(&env, 2);
    // OnUpdate advances duration clocks; the GUI cast-completion stage removes
    // ordinary casts. That boundary is covered by casting::duration_tests.
    env.exec("assert(UnitCastingDuration('player'):HasExpired())")
        .unwrap();
}

#[test]
fn unit_cast_durations_channel_updates_and_lifecycle() {
    let env = setup();
    env.exec(
        r#"
        A_Admin.SetCasting(19750, 'Old cast', 'icon', 20)
        A_Admin.StartChannel(15407, 'Channel', 'icon', 20)
        absent(UnitCastingDuration, 'player')
        absent(UnitEmpoweredChannelDuration, 'player')
        local original = total(UnitChannelDuration, 20)
        assert(A_Admin.UpdateChannel(30))
        assert(total(UnitChannelDuration, 30):GetStartTime() == original:GetStartTime())
        assert(A_Admin.StopChannel())
        absent(UnitChannelDuration, 'player')
        A_Admin.StartChannel(15407, 'Channel', 'icon', 20)
        A_Admin.SetCasting(19750, 'Replacement cast', 'icon', 20)
        absent(UnitChannelDuration, 'player')
        A_Admin.StartChannel(15407, 'Completing channel', 'icon', 1)
    "#,
    )
    .unwrap();
    advance(&env, 2);
    env.exec("absent(UnitChannelDuration, 'player')").unwrap();
}

#[test]
fn unit_cast_durations_consumer_matches_numeric_bar_ids_on_updates_and_stops() {
    let env = setup();
    env.exec(r#"
        local frame = CreateFrame('Frame')
        local activeID
        updates, stops = 0, 0
        for _, event in ipairs({'UNIT_SPELLCAST_START', 'UNIT_SPELLCAST_DELAYED',
            'UNIT_SPELLCAST_STOP', 'UNIT_SPELLCAST_CHANNEL_START',
            'UNIT_SPELLCAST_CHANNEL_UPDATE', 'UNIT_SPELLCAST_CHANNEL_STOP'}) do
            frame:RegisterEvent(event)
        end
        frame:SetScript('OnEvent', function(_, event, unit, guid, spell, a, b)
            if event == 'UNIT_SPELLCAST_START' then
                activeID = select(10, UnitCastingInfo(unit))
                assert(type(activeID) == 'number' and activeID == a)
            elseif event == 'UNIT_SPELLCAST_CHANNEL_START' then
                activeID = select(11, UnitChannelInfo(unit))
                assert(type(activeID) == 'number' and activeID == a)
            elseif event == 'UNIT_SPELLCAST_DELAYED' or event == 'UNIT_SPELLCAST_CHANNEL_UPDATE' then
                if a and activeID == a then updates = updates + 1 end
            else
                local id = event == 'UNIT_SPELLCAST_CHANNEL_STOP' and b or a
                if id and activeID == id then stops = stops + 1; activeID = nil end
            end
        end)
        CastSpellByID(19750)
        assert(type(activeID) == 'number', 'cast START must initialize consumer identity')
        assert(A_Admin.DelayCasting(2))
        assert(SpellStopCasting())
        A_Admin.StartChannel(15407, 'Channel', 'icon', 20)
        assert(A_Admin.UpdateChannel(30))
        assert(A_Admin.StopChannel())
        assert(updates == 2 and stops == 2)
    "#).unwrap();
}

#[test]
fn unit_cast_durations_empower_hold_update_and_completion() {
    let env = setup();
    env.exec(
        r#"
        A_Admin.StartEmpower(357208, 'Empower', 'icon', {1, 2}, 4)
        total(UnitChannelDuration, 3)
        total(UnitEmpoweredChannelDuration, 7)
        total(UnitEmpoweredChannelDuration, 7, true)
        total(UnitEmpoweredChannelDuration, 3, false)
        assert(A_Admin.UpdateEmpower({2, 3}, 5))
        total(UnitChannelDuration, 5)
        total(UnitEmpoweredChannelDuration, 10)
        total(UnitEmpoweredChannelDuration, 5, false)
    "#,
    )
    .unwrap();
    advance(&env, 6);
    env.exec(
        r#"
        assert(UnitChannelDuration('player'):HasExpired())
        assert(not UnitEmpoweredChannelDuration('player'):HasExpired())
        total(UnitEmpoweredChannelDuration, 10)
    "#,
    )
    .unwrap();
    advance(&env, 5);
    env.exec(
        r#"
        absent(UnitChannelDuration, 'player')
        absent(UnitEmpoweredChannelDuration, 'player')
        A_Admin.StartEmpower(357208, 'Canceled', 'icon', {2, 3}, 5)
        assert(SpellStopCasting())
        absent(UnitEmpoweredChannelDuration, 'player')
    "#,
    )
    .unwrap();
}
