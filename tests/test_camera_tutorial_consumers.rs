use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn camera_fov_defaults_drive_blizzard_slider_options(env: &WowLuaEnv) {
    env.exec(r#"
        local _, minValue, maxValue = GetCameraFOVDefaults()
        local options = Settings.CreateSliderOptions(minValue, maxValue, 5)
        local setting = Settings.GetSetting("PROXY_CAMERA_FOV")
        A_Admin.SetCameraFOV(60)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 60)
        A_Admin.SetCameraFOV(80)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 80)
    "#).expect("Blizzard Graphics slider should refresh from supplied camera FOV values");
}
}

prefork_full_ui_case! {
fn tutorials_enabled_controls_original_npe_bootstrap(env: &WowLuaEnv) {
    env.exec(r#"
        A_Admin.SetPlayerNPERestricted(true)
        A_Admin.SetTutorialsEnabled(false)
        NPE_LoadUI()
        assert(not C_AddOns.IsAddOnLoaded("Blizzard_NewPlayerExperience"))
        A_Admin.SetTutorialsEnabled(true)
        NPE_LoadUI()
        assert(C_AddOns.IsAddOnLoaded("Blizzard_NewPlayerExperience"))
    "#).expect("NPE bootstrap should use the stored tutorial enable state");
}
}

prefork_full_ui_case! {
fn tutorial_flag_controls_original_tutorial_frame_consumer(env: &WowLuaEnv) {
    env.exec(r#"
        A_Admin.SetTutorialFlag(17, true)
        TutorialFrame_NewTutorial(17)
        assert(not TutorialFrame:IsShown())
        A_Admin.SetTutorialFlag(17, false)
        TutorialFrame_NewTutorial(17)
        assert(TutorialFrame:IsShown())
    "#).expect("TutorialFrame should show only when the supplied tutorial flag is clear");
}
}

prefork_full_ui_case! {
fn reset_tutorials_clears_flags_in_settings_consumer(env: &WowLuaEnv) {
    env.exec(r#"
        A_Admin.SetTutorialFlag(17, true)
        InterfaceOverrides.ShowTutorialsOnButtonClick()
        assert(not IsTutorialFlagged(17))
        A_Admin.SetTutorialFlag(18, true)
        TutorialFrame_NewTutorial(18)
        assert(not TutorialFrame:IsShown())
    "#).expect("Settings reset should clear tutorial bits observed by TutorialFrame");
    assert_eq!(
        env.state().borrow().tutorial_requests.last(),
        Some(&1),
        "the original settings consumer should record its TriggerTutorial request"
    );
}
}
