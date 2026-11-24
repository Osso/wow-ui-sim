use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn camera_fov_defaults_drive_blizzard_slider_options(env: &WowLuaEnv) {
    env.exec(r#"
        local _, minValue, maxValue = GetCameraFOVDefaults()
        local options = Settings.CreateSliderOptions(minValue, maxValue, 5)
        A_Admin.SetCameraFOV(60)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(Settings.GetValue("PROXY_CAMERA_FOV") == 60)
        A_Admin.SetCameraFOV(80)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(Settings.GetValue("PROXY_CAMERA_FOV") == 80)
    "#).expect("Blizzard Graphics slider options should use native FOV bounds");
}
}

prefork_full_ui_case! {
fn tutorials_enabled_controls_new_player_experience_bootstrap(env: &WowLuaEnv) {
    env.exec(r#"
        local addon = "Blizzard_NewPlayerExperience"
        assert(not C_AddOns.IsAddOnLoaded(addon))
        A_Admin.SetPlayerNPERestricted(true)
        A_Admin.SetTutorialsEnabled(false)
        NPE_LoadUI()
        assert(not C_AddOns.IsAddOnLoaded(addon))
        A_Admin.SetTutorialsEnabled(true)
        NPE_LoadUI()
        assert(C_AddOns.IsAddOnLoaded(addon))
    "#).expect("NPE bootstrap should use the supplied tutorials-enabled flag");
}
}

prefork_full_ui_case! {
fn tutorial_flags_reach_blizzard_tutorial_consumer(env: &WowLuaEnv) {
    env.exec(r#"
        local tutorialID = 4
        A_Admin.SetTutorialFlag(tutorialID, true)
        local before = TutorialFrameAlertButton.id
        TutorialFrame_NewTutorial(tutorialID)
        assert(TutorialFrameAlertButton.id == before)
        A_Admin.SetTutorialFlag(tutorialID, false)
        TutorialFrame_NewTutorial(tutorialID)
        assert(TutorialFrameAlertButton.id == tutorialID)
        A_Admin.SetTutorialFlag(tutorialID, true)
        InterfaceOverrides.ShowTutorialsOnButtonClick()
        assert(not IsTutorialFlagged(tutorialID))
    "#).expect("TutorialFrame should respect the supplied tutorial flag set");
}
}
