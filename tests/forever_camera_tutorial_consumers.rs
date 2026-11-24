//! Forever camera consumer proof.
#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_camera_fov_defaults_drive_graphics_slider() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_Settings_Shared"], &[]);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SettingsDefinitions_Shared"], &[]);
    env.exec(
        r#"
        local _, minValue, maxValue = GetCameraFOVDefaults()
        local options = Settings.CreateSliderOptions(minValue, maxValue, 5)
        local setting = Settings.GetSetting("PROXY_CAMERA_FOV")
        A_Admin.SetCameraFOV(60)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 60)
        A_Admin.SetCameraFOV(80)
        assert(options.minValue == 50 and options.maxValue == 90)
        assert(setting:GetValue() == 80)
    "#,
    )
    .expect(
        "Forever Graphics setting should read supplied camera FOV through its original consumer",
    );
}
