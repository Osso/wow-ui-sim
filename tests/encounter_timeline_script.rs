//! Public script-event producer proof; track/filter/EditMode behavior is separate.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        timeline = C_EncounterTimeline
        function add(duration, extra)
            local request = {spellID=19750, iconFileID=135907, duration=duration}
            for k,v in pairs(extra or {}) do request[k] = v end
            return timeline.AddScriptEvent(request)
        end
        assert(#timeline.GetEventList() == 0, "PTR demo must be absent")
    "#).unwrap();
    env
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_defaults_info_and_atomic_validation() {
    let env = environment();
    env.exec(r#"
        local id = add(5)
        local info = timeline.GetEventInfo(id)
        assert(id > 0 and info.id == id and info.source == Enum.EncounterTimelineEventSource.Script)
        assert(info.spellID == 19750 and info.spellName == "Flash of Light")
        assert(info.iconFileID == 135907 and info.duration == 5 and info.maxQueueDuration == 0)
        assert(info.icons == 0 and info.severity == Enum.EncounterEventSeverity.Medium)
        assert(info.isApproximate == false and timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Active)
        info.duration = 999
        assert(timeline.GetEventInfo(id).duration == 5)
        local other = add(7, {maxQueueDuration=3, overrideName="Custom\0Name", icons=9, severity=2, paused=true})
        local explicit = timeline.GetEventInfo(other)
        assert(other ~= id and explicit.spellName == "Custom\0Name" and explicit.maxQueueDuration == 3)
        assert(explicit.icons == 9 and explicit.severity == 2 and explicit.duration == 7)
        assert(timeline.GetEventState(other) == Enum.EncounterTimelineEventState.Paused)
        assert(timeline.HasAnyEvents() and timeline.HasActiveEvents() and timeline.HasPausedEvents())
        assert(timeline.GetEventCountBySource(Enum.EncounterTimelineEventSource.Script) == 2)
        assert(timeline.GetEventCountBySource(Enum.EncounterTimelineEventSource.Encounter) == 0)
        assert(timeline.IsEventBlocked(id) == false)
        local count = #timeline.GetEventList()
        for _,bad in ipairs({-1, math.huge, 0/0, "bad"}) do assert(not pcall(add, bad)) end
        assert(not pcall(add, 1, {paused=1}))
        assert(not pcall(add, 1, {severity=8}))
        assert(not pcall(timeline.AddScriptEvent, {duration=1}))
        assert(#timeline.GetEventList() == count)
        assert(timeline.GetEventInfo(0) == nil and timeline.GetEventTimer(9999) == nil)
        assert(select('#', timeline.GetEventState(9999)) == 0)
        assert(select('#', timeline.CancelScriptEvent(9999)) == 0)
        assert(EncounterTimelineEventInfo == nil)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_clock_pause_resume_and_retained_timer() {
    let env = environment();
    env.exec(r#"
        id = add(5)
        timer = timeline.GetEventTimer(id)
        another = timeline.GetEventTimer(id)
        assert(timer ~= another and timer:GetTotalDuration() == 5)
        assert(timer:GetElapsedDuration() == 0 and timer:GetRemainingDuration() == 5)
        before = timeline.GetCurrentTime()
    "#).unwrap();
    env.fire_on_update(1.25).unwrap();
    env.exec(r#"
        assert(timeline.GetCurrentTime() == before + 1.25)
        assert(timeline.GetEventTimeElapsed(id) == 1.25 and timer:GetRemainingDuration() == 3.75)
        timeline.PauseScriptEvent(id)
        assert(timeline.HasPausedEvents() and not timeline.HasActiveEvents())
        another:SetTimeFromStart(0, 100)
        assert(timer:GetTotalDuration() == 5 and timeline.GetEventInfo(id).duration == 5)
    "#).unwrap();
    env.fire_on_update(10.0).unwrap();
    env.exec(r#"
        assert(timer:GetElapsedDuration() == 1.25 and timer:GetRemainingDuration() == 3.75)
        timeline.ResumeScriptEvent(id)
    "#).unwrap();
    env.fire_on_update(0.75).unwrap();
    env.exec(r#"
        assert(timer:GetElapsedDuration() == 2 and timeline.GetEventTimeRemaining(id) == 3)
        timeline.CancelScriptEvent(id)
        assert(timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Canceled)
    "#).unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec(r#"
        assert(timeline.GetEventInfo(id) == nil and timeline.GetEventTimer(id) == nil)
        collectgarbage('collect')
        assert(timer:GetElapsedDuration() == 2 and timer:GetRemainingDuration() == 3)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_terminal_callbacks_observe_consistent_tick_state() {
    let env = environment();
    env.exec(r#"
        log = {}; observations = 0
        listener = CreateFrame("Frame")
        for _,event in ipairs({"ENCOUNTER_TIMELINE_EVENT_ADDED", "ENCOUNTER_TIMELINE_EVENT_STATE_CHANGED", "ENCOUNTER_TIMELINE_EVENT_REMOVED"}) do listener:RegisterEvent(event) end
        listener:SetScript("OnEvent", function(_, event, payload)
            if event == "ENCOUNTER_TIMELINE_EVENT_ADDED" then
                assert(timeline.GetEventInfo(payload.id).duration == payload.duration)
                payload.spellName = "caller mutation"
                table.insert(log, "added")
            elseif event == "ENCOUNTER_TIMELINE_EVENT_STATE_CHANGED" then
                assert(timeline.GetEventState(payload) == Enum.EncounterTimelineEventState.Finished)
                assert(timeline.GetEventTimeRemaining(payload) == 0)
                table.insert(log, "finished")
            else
                assert(timeline.GetEventInfo(payload) == nil and timeline.GetEventState(payload) == nil)
                table.insert(log, "removed")
            end
        end)
        listener:SetScript("OnUpdate", function()
            if id and timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Finished then observations = observations + 1 end
        end)
        id = add(1)
        assert(log[1] == "added" and timeline.GetEventInfo(id).spellName == "Flash of Light")
        saved = timeline.GetEventTimer(id)
    "#).unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(#log == 2 and log[2] == 'finished' and observations == 1); assert(timeline.HasAnyEvents() and not timeline.HasActiveEvents())").unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("assert(log[3] == 'removed' and #log == 3 and observations == 2); assert(not timeline.HasAnyEvents() and saved:GetElapsedDuration() == 1)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_cancel_all_reentrancy_and_initial_pause() {
    let env = environment();
    env.exec(r#"
        a = add(10, {paused=true}); b = add(10)
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("ENCOUNTER_TIMELINE_EVENT_STATE_CHANGED")
        listener:SetScript("OnEvent", function(_,_, id)
            if id == a then
                assert(timeline.GetEventState(a) == Enum.EncounterTimelineEventState.Canceled)
                survivor = add(20)
                timeline.FinishScriptEvent(b)
            end
        end)
        timeline.CancelAllScriptEvents()
        assert(timeline.GetEventState(a) == Enum.EncounterTimelineEventState.Canceled)
        assert(timeline.GetEventState(b) == Enum.EncounterTimelineEventState.Finished)
        assert(timeline.GetEventState(survivor) == Enum.EncounterTimelineEventState.Active)
        assert(timeline.GetEventTimeElapsed(a) == 0 and timeline.GetEventTimeElapsed(b) == 10)
    "#).unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(#timeline.GetEventList() == 1 and timeline.GetEventList()[1] == survivor); assert(timeline.GetEventTimeElapsed(survivor) == 0.5)").unwrap();
    let other = environment();
    other.exec("assert(not timeline.HasAnyEvents() and #timeline.GetEventList() == 0)").unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn encounter_script_preserves_retail_demo_baseline() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(r#"
            assert(C_EncounterTimeline.GetEventList()[1] == 1)
            assert(C_EncounterTimeline.GetEventInfo(1).spellName == "Flash of Light")
            assert(C_EncounterTimeline.GetEventTimer(1):GetRemainingDuration() == 12.5)
            assert(C_EncounterTimeline.HasActiveEvents())
        "#).unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_zero_duration_and_bootstrap_do_not_reseed_demo() {
    let env = environment();
    env.exec(r#"
        id = add(0, {paused=true, maxQueueDuration=4})
        assert(timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Paused)
        assert(timeline.GetEventTimeElapsed(id) == 0)
        assert(timeline.GetEventTrack(id) == Enum.EncounterTimelineTrack.Indeterminate)
        assert(#timeline.GetSortedEventList() == 0)
    "#).unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(timeline.HasPausedEvents()); timeline.ResumeScriptEvent(id)") .unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec("assert(timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Active); assert(timeline.GetEventTrack(id) == Enum.EncounterTimelineTrack.Queued)").unwrap();
    env.fire_on_update(4.0).unwrap();
    env.exec("assert(timeline.GetEventState(id) == Enum.EncounterTimelineEventState.Finished)").unwrap();
    env.fire_on_update(0.0).unwrap();
    wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    env.exec(r#"
        assert(timeline.GetEventInfo(id) == nil and #timeline.GetEventList() == 0)
        local nextID = add(3)
        assert(nextID > id)
        assert(timeline.GetEventInfo(nextID).duration == 3)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_script_timer_supports_blizzard_event_frame_consumer() {
    let env = environment();
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap()
        .join("Blizzard_EncounterTimeline");
    for file in ["EncounterTimelineSettings.lua", "EncounterTimelineEventFrame.lua"] {
        env.exec(&std::fs::read_to_string(root.join(file)).unwrap()).unwrap();
    }
    env.exec(r#"
        id = add(4)
        consumer = CreateFromMixins(EncounterTimelineEventFrameMixin)
        consumer:Init(timeline.GetEventInfo(id), timeline.GetEventTimer(id),
                      timeline.GetEventState(id), nil, nil, false, nil)
        assert(consumer:GetEventTimeRemaining() == 4)
    "#).unwrap();
    env.fire_on_update(1.5).unwrap();
    env.exec(r#"
        assert(consumer:GetEventTimeElapsed() == 1.5 and consumer:GetEventTimeRemaining() == 2.5)
        timeline.PauseScriptEvent(id)
    "#).unwrap();
    env.fire_on_update(5.0).unwrap();
    env.exec(r#"
        assert(consumer:GetEventTimeRemaining() == 2.5)
        timeline.FinishScriptEvent(id)
        assert(consumer:GetEventTimeRemaining() == 0)
    "#).unwrap();
    env.fire_on_update(0.1).unwrap();
    env.exec("assert(timeline.GetEventInfo(id) == nil and consumer:GetEventTimeElapsed() == 4)").unwrap();
}
