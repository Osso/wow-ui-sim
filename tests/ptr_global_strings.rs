#![cfg(feature = "client-ptr")]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn ptr_gamepad_label_allows_original_gameplay_settings_group() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let mut loaded_settings = false;
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).expect("load Blizzard addon");
        if name == "Blizzard_SettingsDefinitions_Frame" {
            loaded_settings = true;
        }
    }
    assert!(loaded_settings, "original settings addon was not selected");
    let errors: Vec<_> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|error| {
            error.contains("GameplaySettingsGroup.lua") && error.contains("table index is nil")
        })
        .cloned()
        .collect();
    assert!(
        errors.is_empty(),
        "original GameplaySettingsGroup.lua nil key: {errors:?}"
    );
    env.exec("assert(type(GAMEPAD_LABEL) == 'string' and GAMEPAD_LABEL == 'Gamepad', 'PTR GlobalStrings row 58600')")
        .expect("PTR GAMEPAD_LABEL");
}
