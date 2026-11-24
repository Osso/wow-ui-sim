//! Forever consumers for modeled camera and tutorial state.
#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_addon(env: &WowLuaEnv, addon: &str) {
    load_blizzard_addon_closure_into_env(env, &blizzard_ui_dir(), &[addon], &[]);
}

#[test]
fn forever_tutorial_flags_clear_through_settings_consumer() {
    let env = new_blizzard_addon_env(&blizzard_ui_dir());
    load_addon(&env, "Blizzard_Settings_Shared");
    load_addon(&env, "Blizzard_SettingsDefinitions_Shared");
    load_addon(&env, "Blizzard_SettingsDefinitions_Frame");
    load_addon(&env, "Blizzard_FrameXML");
    env.exec(
        r#"
        A_Admin.SetTutorialFlag(1, true)
        InterfaceOverrides.ShowTutorialsOnButtonClick()
        assert(not IsTutorialFlagged(1))
    "#,
    )
    .expect("Forever settings consumer should clear supplied tutorial bits");
}
