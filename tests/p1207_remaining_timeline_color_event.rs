#![cfg(feature = "retail-12-1-5")]

//! Existing later timeline producer proof only; not strict-12.0.7 availability proof.
use wow_ui_sim::lua_api::WowLuaEnv;

fn color_event_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("timeline color observer");
    env.exec(
        r#"
        TimelineColorEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED')
        listener:SetScript('OnEvent', function(_, name, ...)
            assert(name == 'ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED')
            assert(select('#', ...) == 1)
            local id = ...
            assert(type(id) == 'number' and not issecretvalue(id))
            assert(C_EncounterTimeline.GetEventInfo(id) ~= nil)
            assert(C_EncounterTimeline.GetEventTimeRemaining(id) <= 5)
            local before = debug.getstacktaint()
            local current = C_EncounterTimeline.GetEventState(id)
            assert(current == Enum.EncounterTimelineEventState.Active)
            assert(debug.getstacktaint() == before)
            TimelineColorEvents[#TimelineColorEvents + 1] = id
        end)
        debug.setobjecttaint(listener:GetScript('OnEvent'), 'TimelineObserver')
        "#,
    )
    .expect("observe native notifications through tainted frame callback");
    env
}

#[test]
fn p1207_existing_timeline_color_event_empty_default_is_silent() {
    let env = color_event_environment();
    env.fire_on_update(60.0).unwrap();
    assert_eq!(env.eval::<f64>("return #TimelineColorEvents").unwrap(), 0.0);
    env.exec("assert(#C_EncounterTimeline.GetEventList() == 0)").unwrap();
}

#[test]
fn p1207_existing_timeline_color_event_crosses_five_seconds_once() {
    let env = color_event_environment();
    env.exec(
        r#"
        ColorEventID = C_EncounterTimeline.AddScriptEvent({
            spellID = 19750, iconFileID = 135907, duration = 8,
        })
        assert(#TimelineColorEvents == 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(2.5).unwrap();
    assert_eq!(env.eval::<f64>("return #TimelineColorEvents").unwrap(), 0.0);
    env.fire_on_update(0.5).unwrap();
    env.exec(
        "assert(#TimelineColorEvents == 1 and TimelineColorEvents[1] == ColorEventID)",
    )
    .expect("exact payload and live timer state at callback");
    env.fire_on_update(0.0).unwrap();
    env.fire_on_update(0.25).unwrap();
    assert_eq!(env.eval::<f64>("return #TimelineColorEvents").unwrap(), 1.0);
}

#[test]
fn p1207_existing_timeline_color_event_is_environment_local() {
    let first = color_event_environment();
    let second = color_event_environment();
    first.exec(
        r#"
        LocalColorID = C_EncounterTimeline.AddScriptEvent({
            spellID = 19750, iconFileID = 135907, duration = 6,
        })
        "#,
    )
    .unwrap();
    first.fire_on_update(1.0).unwrap();
    second.fire_on_update(1.0).unwrap();
    first.exec(
        "assert(#TimelineColorEvents == 1 and TimelineColorEvents[1] == LocalColorID)",
    )
    .unwrap();
    second.exec("assert(#TimelineColorEvents == 0 and #C_EncounterTimeline.GetEventList() == 0)")
        .unwrap();
}

#[test]
fn p1207_existing_timeline_color_event_cancellation_prevents_notification() {
    let env = color_event_environment();
    env.exec(
        r#"
        local id = C_EncounterTimeline.AddScriptEvent({
            spellID = 19750, iconFileID = 135907, duration = 9,
        })
        C_EncounterTimeline.CancelScriptEvent(id)
        "#,
    )
    .unwrap();
    env.fire_on_update(4.0).unwrap();
    assert_eq!(env.eval::<f64>("return #TimelineColorEvents").unwrap(), 0.0);
}
