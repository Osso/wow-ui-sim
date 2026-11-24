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
    "#).expect("Blizzard Graphics slider options should use native FOV bounds");
}
}
