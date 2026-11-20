#[cfg(feature = "retail-12-1-0")]
#[test]
fn encounter_timeline_retail_consumer_displays_supplied_events() {
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
        assert(timeline.IsFeatureAvailable() and timeline.IsFeatureEnabled())
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

#[cfg(feature = "client-wowforever")]
#[test]
fn encounter_timeline_forever_setting_hides_unavailable_timeline() {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_SettingsDefinitions_Frame"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_SettingsDefinitions_Frame"));
    env.exec("SetCVar('combatWarningsEnabled', '1')").unwrap();
    let visibility: String = env
        .eval(
            r#"
            local layouts = {}
            local register = Settings.RegisterVerticalLayoutCategory
            Settings.RegisterVerticalLayoutCategory = function(...)
                local category, layout = register(...)
                table.insert(layouts, layout)
                return category, layout
            end
            SettingsRegistrar.allowCallRegistrants = true
            for _, callback in ipairs(SettingsRegistrar.registrants or {}) do callback() end
            for _, layout in ipairs(layouts) do
                for _, initializer in ipairs(layout:GetInitializers()) do
                    local setting = initializer.GetSetting and initializer:GetSetting()
                    if setting and setting:GetVariable() == 'encounterTimelineEnabled' then
                        return initializer:EvaluateModifyPredicates() and 'shown' or 'hidden'
                    end
                end
            end
            return 'setting not registered'
            "#,
        )
        .unwrap();
    assert_eq!(visibility, "hidden");
}
