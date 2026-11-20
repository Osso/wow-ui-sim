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
    let result: String = env
        .eval(
            r#"
            local timeline = C_EncounterTimeline
            if not timeline.IsFeatureAvailable() then return "unavailable" end
            if not timeline.IsFeatureEnabled() then return "disabled" end
            if #timeline.GetEventList() ~= 0 then return "default event list is not empty" end
            if type(EncounterTimelineDataProviderMixin) ~= "table" then return "provider not loaded" end
            local provider = CreateFromMixins(EncounterTimelineDataProviderMixin)
            provider:OnLoad()
            local id = timeline.AddScriptEvent({spellID=19750, iconFileID=135907, duration=9})
            provider:AddAllEvents(timeline.GetEventList())
            if not provider:HasEvent(id) then return "consumer did not add event" end
            local info = provider:GetEventInfo(id)
            if info.id ~= id or info.spellID ~= 19750 or info.duration ~= 9 then return "consumer event differs" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}
