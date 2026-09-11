//! Self-cancel producer tests; no injected spellcast events or enemy interrupt actor.
use super::{apply_spell_effect, extract_completed_cast, fire_cast_complete_events};
use crate::lua_api::WowLuaEnv;

fn listen_for_cancellation(env: &WowLuaEnv) {
    env.exec(
        r#"
        cancellationEvents = {}
        cancellationListener = CreateFrame("Frame")
        for _, event in ipairs({"UNIT_SPELLCAST_START", "UNIT_SPELLCAST_INTERRUPTED",
                                "UNIT_SPELLCAST_STOP", "UNIT_SPELLCAST_SUCCEEDED"}) do
            cancellationListener:RegisterEvent(event)
        end
        cancellationListener:SetScript("OnEvent", function(_, event, ...)
            local row = {event = event, count = select('#', ...), ...}
            row.queryBarID = select(10, UnitCastingInfo("player"))
            table.insert(cancellationEvents, row)
            if restartOnEvent == event then
                restartOnEvent = nil
                CastSpellByID(82326)
            end
        end)
        "#,
    )
    .unwrap();
}

fn tick_completed_effects(env: &WowLuaEnv) {
    if let Some((cast_id, spell_id)) = extract_completed_cast(env.state()) {
        fire_cast_complete_events(env, cast_id, spell_id);
        apply_spell_effect(env.state(), env, spell_id);
    }
}

#[test]
fn spellcast_interrupted_self_cancel_reports_actor_and_never_completes() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.party_members[0].health = 1_000;
        sim.party_members[0].health_max = 100_000;
    }
    listen_for_cancellation(&env);
    env.exec(
        r#"
        TargetUnit("party1")
        assert(UnitGUID("target") ~= UnitGUID("player"))
        CastSpellByID(19750)
        assert(UnitCastingInfo("player") ~= nil)
        local function pack(...) return {count = select('#', ...), ...} end
        local result = pack(SpellStopCasting())
        assert(result.count == 1 and result[1] == true)
        assert(#cancellationEvents == 3, "expected START, INTERRUPTED, STOP")
        local start, interrupted, stop = unpack(cancellationEvents)
        assert(interrupted.event == "UNIT_SPELLCAST_INTERRUPTED")
        assert(interrupted.count == 5, "interruption requires five arguments")
        assert(interrupted[1] == "player" and interrupted[2] == start[2])
        assert(interrupted[3] == 19750 and interrupted[4] == UnitGUID("player"))
        assert(interrupted[5] == start[4] and interrupted[5] == start.queryBarID)
        assert(interrupted.queryBarID == nil)
        assert(stop.event == "UNIT_SPELLCAST_STOP" and stop.count == 4)
        for i = 1, 4 do assert(stop[i] == start[i]) end
        assert(stop.queryBarID == nil and UnitCastingInfo("player") == nil)
        assert(SpellStopCasting() == false and #cancellationEvents == 3)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(30);
    tick_completed_effects(&env);
    assert!(extract_completed_cast(env.state()).is_none());
    assert_eq!(env.state().borrow().party_members[0].health, 1_000);
    env.exec("assert(#cancellationEvents == 3); assert(UnitCastingInfo('player') == nil)")
        .unwrap();
}

fn assert_replacement_started(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(#cancellationEvents == 4, "replacement START must survive cancellation")
        local oldStart = cancellationEvents[1]
        local interrupted = cancellationEvents[2]
        assert(interrupted.event == "UNIT_SPELLCAST_INTERRUPTED")
        assert(interrupted.count == 5 and interrupted[2] == oldStart[2])
        assert(interrupted[4] == UnitGUID("player") and interrupted[5] == oldStart[4])
        assert(interrupted.queryBarID == nil)
        local replacement, oldStop
        for _, row in ipairs(cancellationEvents) do
            if row.event == "UNIT_SPELLCAST_START" and row[3] == 82326 then replacement = row end
            if row.event == "UNIT_SPELLCAST_STOP" then oldStop = row end
        end
        assert(replacement and oldStop and oldStop[2] == oldStart[2])
        assert(oldStop[4] == oldStart[4] and oldStop.count == 4)
        if cancellationEvents[3].event == "UNIT_SPELLCAST_START" then
            assert(oldStop.queryBarID == replacement[4])
        else
            assert(oldStop.queryBarID == nil)
        end
        assert(replacement[2] ~= oldStart[2] and replacement[4] ~= oldStart[4])
        assert(select(9, UnitCastingInfo("player")) == 82326)
        assert(select(10, UnitCastingInfo("player")) == replacement[4])
        replacementGUID, replacementBarID = replacement[2], replacement[4]
        "#,
    )
    .unwrap();
}

fn complete_replacement(env: &WowLuaEnv) {
    assert_eq!(env.state().borrow().player.health, 1_000);
    env.state().borrow_mut().casting.as_mut().unwrap().end_time = 0.0;
    tick_completed_effects(env);
    env.exec(
        r#"
        assert(#cancellationEvents == 6)
        local stop, success = cancellationEvents[5], cancellationEvents[6]
        assert(stop.event == "UNIT_SPELLCAST_STOP" and success.event == "UNIT_SPELLCAST_SUCCEEDED")
        for _, row in ipairs({stop, success}) do
            assert(row.count == 4 and row[2] == replacementGUID)
            assert(row[3] == 82326 and row[4] == replacementBarID)
            assert(row.queryBarID == nil)
        end
        assert(SpellStopCasting() == false and #cancellationEvents == 6)
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().player.health, 36_000);
    tick_completed_effects(env);
    assert_eq!(env.state().borrow().player.health, 36_000);
}

fn assert_reentrant_cancel_preserves_replacement(restart_event: &str) {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.health = 1_000;
        sim.player.health_max = 100_000;
    }
    listen_for_cancellation(&env);
    env.exec(&format!(
        "ClearTarget(); restartOnEvent = '{restart_event}'; \
         CastSpellByID(19750); assert(SpellStopCasting() == true)"
    ))
    .unwrap();
    assert_replacement_started(&env);
    complete_replacement(&env);
}

#[test]
fn spellcast_interrupted_handler_can_start_a_replacement_cast() {
    assert_reentrant_cancel_preserves_replacement("UNIT_SPELLCAST_INTERRUPTED");
}

#[test]
fn spellcast_interrupted_stop_handler_can_start_a_replacement_cast() {
    assert_reentrant_cancel_preserves_replacement("UNIT_SPELLCAST_STOP");
}
