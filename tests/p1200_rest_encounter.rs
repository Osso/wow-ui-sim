#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_encounter_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_InstanceEncounter.IsEncounterLimitingResurrections() == false)
        assert(C_InstanceEncounter.IsEncounterSuppressingRelease() == false)
        assert(C_InstanceEncounter.ShouldShowTimelineForEncounter() == false)
        assert(C_EncounterWarnings.IsFeatureAvailable() == false)
        assert(C_EncounterWarnings.IsFeatureEnabled() == false)
        assert(C_EncounterWarnings.GetSoundKitForSeverity(0) == 0)
        assert(not pcall(C_EncounterWarnings.GetSoundKitForSeverity, 9))
    "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.encounter_policy.limiting_resurrections = true;
        state.encounter_policy.suppressing_release = true;
        state.encounter_policy.show_timeline = true;
        state.encounter_warning_settings.available = true;
        state.encounter_warning_settings.sound_kits = [100, 200, 300];
    }
    env.exec(
        r#"
        assert(C_InstanceEncounter.IsEncounterLimitingResurrections())
        assert(C_InstanceEncounter.IsEncounterSuppressingRelease())
        assert(C_InstanceEncounter.ShouldShowTimelineForEncounter())
        assert(C_EncounterWarnings.IsFeatureAvailable())
        assert(C_EncounterWarnings.IsFeatureEnabled())
        for severity = 0, 2 do
            assert(C_EncounterWarnings.GetSoundKitForSeverity(severity) == (severity + 1) * 100)
        end
        C_CVar.SetCVar('encounterWarningsEnabled', '0')
        assert(not C_EncounterWarnings.IsFeatureEnabled())
        assert(C_EncounterWarnings.IsFeatureAvailable())
    "#,
    )
    .unwrap();
    WowLuaEnv::new()
        .unwrap()
        .exec("assert(not C_InstanceEncounter.IsEncounterSuppressingRelease())")
        .unwrap();
}
