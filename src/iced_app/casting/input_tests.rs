//! Simulator inputs drive real timed casts; events are never injected by tests.
use super::{
    apply_spec_change, apply_spell_effect, extract_completed_cast, fire_cast_complete_events,
};
use crate::lua_api::WowLuaEnv;

fn listen(env: &WowLuaEnv) {
    env.exec(
        r#"
        inputEvents = {}
        local f = CreateFrame('Frame')
        for _, e in ipairs({'UNIT_SPELLCAST_START','UNIT_SPELLCAST_DELAYED',
            'UNIT_SPELLCAST_FAILED','UNIT_SPELLCAST_FAILED_QUIET','UNIT_SPELLCAST_STOP',
            'UNIT_SPELLCAST_SUCCEEDED'}) do f:RegisterEvent(e) end
        f:SetScript('OnEvent', function(_, event, ...)
            local row = {event=event, n=select('#', ...), ...}
            row.bar = select(10, UnitCastingInfo('player'))
            row.delay = select(11, UnitCastingInfo('player'))
            inputEvents[#inputEvents+1] = row
            if restartOn == event then restartOn=nil; CastSpellByID(82326) end
        end)
    "#,
    )
    .unwrap();
}

fn complete(env: &WowLuaEnv) {
    if let Some((id, spell)) = extract_completed_cast(env.state()) {
        fire_cast_complete_events(env, id, spell);
        apply_spell_effect(env.state(), env, spell);
        apply_spec_change(env.state(), env);
    }
}

#[test]
fn spellcast_input_delay_updates_query_and_deadline_atomically() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(
        r#"
        assert(A_Admin.DelayCasting(1) == false)
        CastSpellByID(19750)
        original = {UnitCastingInfo('player')}
        assert(A_Admin.DelayCasting(0.5) == true)
        assert(A_Admin.DelayCasting(0.25) == true)
        local updated = {UnitCastingInfo('player')}
        assert(updated[4] == original[4] and updated[7] == original[7])
        assert(updated[10] == original[10] and updated[11] == 750)
        assert(math.abs(updated[5] - original[5] - 750) < 0.00001)
        assert(#inputEvents == 3)
        for i=2,3 do
            local row = inputEvents[i]
            assert(row.event == 'UNIT_SPELLCAST_DELAYED' and row.n == 4)
            for j=1,4 do assert(row[j] == inputEvents[1][j]) end
            assert(row.bar == original[10] and row.delay == (i == 2 and 500 or 750))
        end
        for _, invalid in ipairs({-1, 1e308, math.huge, -math.huge, 0/0, false, '1'}) do
            assert(not pcall(A_Admin.DelayCasting, invalid))
        end
        assert(not pcall(A_Admin.DelayCasting))
        assert(select(11, UnitCastingInfo('player')) == 750 and #inputEvents == 3)
        assert(A_Admin.DelayCasting(0) == true)
    "#,
    )
    .unwrap();
    let old_deadline = env.state().borrow().casting.as_ref().unwrap().end_time - 0.75;
    let now = env.state().borrow().start_time.elapsed().as_secs_f64();
    env.state().borrow_mut().start_time -=
        std::time::Duration::from_secs_f64(old_deadline + 0.1 - now);
    assert!(extract_completed_cast(env.state()).is_none());
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(30);
    complete(&env);
    env.exec("assert(UnitCastingInfo('player') == nil); assert(A_Admin.DelayCasting(1) == false)")
        .unwrap();
}

#[test]
fn spellcast_input_failure_preserves_reentrant_cast_and_cancels_old_effects() {
    for (quiet, event) in [
        (false, "UNIT_SPELLCAST_FAILED"),
        (true, "UNIT_SPELLCAST_FAILED_QUIET"),
    ] {
        for restart in [event, "UNIT_SPELLCAST_STOP"] {
            let env = WowLuaEnv::new().unwrap();
            env.state().borrow_mut().player.health = 1000;
            listen(&env);
            env.exec(&format!(
                r#"
                assert(A_Admin.FailCasting() == false)
                CastSpellByID(19750)
                assert(not pcall(A_Admin.FailCasting, 'bad'))
                assert(UnitCastingInfo('player') ~= nil and #inputEvents == 1)
                restartOn = '{restart}'
                assert(A_Admin.FailCasting({quiet}) == true)
                assert(#inputEvents == 4)
                local failure = inputEvents[2]
                assert(failure.event == '{event}' and failure.n == 4 and failure.bar == nil)
                for i=1,4 do assert(failure[i] == inputEvents[1][i]) end
                local stop
                for _, row in ipairs(inputEvents) do
                    if row.event == 'UNIT_SPELLCAST_STOP' then stop=row end
                end
                assert(stop and stop.n == 4)
                for i=1,4 do assert(stop[i] == inputEvents[1][i]) end
                assert(select(9, UnitCastingInfo('player')) == 82326)
                assert(select(10, UnitCastingInfo('player')) ~= inputEvents[1][4])
            "#
            ))
            .unwrap();
            assert_eq!(env.state().borrow().player.health, 1000);
            env.state().borrow_mut().casting.as_mut().unwrap().end_time = 0.0;
            complete(&env);
            env.exec("assert(#inputEvents == 6); assert(A_Admin.FailCasting() == false)")
                .unwrap();
            assert!(extract_completed_cast(env.state()).is_none());
        }
    }
}

#[test]
fn spellcast_input_failed_specialization_does_not_leak_into_next_completion() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.active_spec_index = 1;
    listen(&env);
    env.exec("assert(C_SpecializationInfo.SetSpecialization(2)); assert(A_Admin.FailCasting())")
        .unwrap();
    env.exec("CastSpellByID(19750)").unwrap();
    env.state().borrow_mut().casting.as_mut().unwrap().end_time = 0.0;
    complete(&env);
    assert_eq!(env.state().borrow().player.active_spec_index, 1);
    assert_eq!(env.state().borrow().player.pending_spec_change, None);
}

#[test]
fn spellcast_input_failure_callback_can_replace_specialization_action() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.active_spec_index = 1;
    env.exec(
        r#"
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('UNIT_SPELLCAST_FAILED')
        listener:SetScript('OnEvent', function()
            assert(C_SpecializationInfo.SetSpecialization(3))
        end)
        assert(C_SpecializationInfo.SetSpecialization(2))
        assert(A_Admin.FailCasting())
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().player.pending_spec_change, Some(3));
    env.state().borrow_mut().casting.as_mut().unwrap().end_time = 0.0;
    complete(&env);
    assert_eq!(env.state().borrow().player.active_spec_index, 3);
}

#[test]
fn spellcast_input_failed_cast_never_completes_or_applies_effects() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.health = 1000;
    listen(&env);
    env.exec(
        "CastSpellByID(19750); assert(A_Admin.FailCasting()); assert(not A_Admin.FailCasting())",
    )
    .unwrap();
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(30);
    complete(&env);
    assert_eq!(env.state().borrow().player.health, 1000);
    env.exec("assert(#inputEvents == 3); assert(inputEvents[2].event == 'UNIT_SPELLCAST_FAILED')")
        .unwrap();
}
