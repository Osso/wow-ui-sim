#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_strings_initialize_actual_gameplay_categories_and_format_tokens() {
    let env = WowLuaEnv::new().unwrap();
    let source = std::fs::read_to_string(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_SettingsDefinitions_Frame/Mainline/GameplaySettingsGroup.lua"),
    )
    .unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        assert(CUSTOM_GAMEPLAY_SETTINGS_ORDER["Controls"] == 7)
        assert(CUSTOM_GAMEPLAY_SETTINGS_ORDER["Gamepad (Alpha)"] == 12)
        assert(CUSTOM_GAMEPLAY_SETTINGS_ORDER["Nameplates"] == 16)
        assert(SLASH_CAA_HELP_SAY_COMBAT_START_SOUND ==
            "sound:[%d - %d] Choose a sound to indicate that combat has started")
        assert(string.format(SLASH_CAA_HELP_SAY_COMBAT_START_SOUND, 1, 94) ==
            "sound:[1 - 94] Choose a sound to indicate that combat has started")
    "#,
    )
    .unwrap();
}

#[test]
fn forever_string_registration_preserves_functions_and_restoration_values() {
    let mut lua = rilua::Lua::new().unwrap();
    lua.exec("GAMEPAD_LABEL = function() return 'dynamic' end")
        .unwrap();
    wow_ui_sim::lua_api::globals::strings::register_all_ui_strings(&mut lua).unwrap();
    lua.exec(
        "assert(GAMEPAD_LABEL() == 'dynamic'); CONTROLS_LABEL = 'custom'; GAMEPAD_LABEL = nil",
    )
    .unwrap();
    wow_ui_sim::lua_api::globals::strings::restore_missing_ui_strings(&mut lua).unwrap();
    lua.exec("assert(CONTROLS_LABEL == 'custom'); assert(GAMEPAD_LABEL == 'Gamepad (Alpha)')")
        .unwrap();
}
