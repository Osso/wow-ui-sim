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
        local provider = CreateFromMixins(EncounterTimelineDataProviderMixin)
        provider:OnLoad()
        local id = timeline.AddScriptEvent({spellID=19750, iconFileID=135907, duration=9})
        provider:AddAllEvents(timeline.GetEventList())
        assert(provider:HasEvent(id))
        local info = provider:GetEventInfo(id)
        assert(info.id == id and info.spellID == 19750 and info.duration == 9)
        "#,
    )
    .unwrap();
}
