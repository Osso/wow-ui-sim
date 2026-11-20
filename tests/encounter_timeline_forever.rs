#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

#[test]
fn encounter_timeline_original_consumer_starts_empty_and_accepts_supplied_events() {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_EncounterTimeline"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_EncounterTimeline"));
    env.exec(
        r#"
        local timeline = C_EncounterTimeline
        assert(timeline.IsFeatureAvailable())
        assert(timeline.IsFeatureEnabled())
        assert(#timeline.GetEventList() == 0)
        EncounterTimeline:Show()
        timeline.SetViewType(Enum.EncounterTimelineViewType.Timeline)
        local id = timeline.AddScriptEvent({spellID=19750, iconFileID=135907, duration=9})
        local view = EncounterTimeline.TrackView
        assert(view:HasEvent(id) and view:HasEventFrame(id))
        local event = view:GetEventFrame(id)
        assert(event:GetEventTimeRemaining() == 9)
        TimelineHostEventID = id
        "#,
    )
    .unwrap();
}
