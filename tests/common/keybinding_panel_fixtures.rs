use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;

#[path = "token_ui_fixtures.rs"]
mod token_ui_fixtures;

use token_ui_fixtures::load_token_ui;

pub(crate) const BLIZZARD_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_ActionBar",
    "Blizzard_GameMenu",
    "Blizzard_UIWidgets",
    "Blizzard_Minimap",
    "Blizzard_AddOnList",
    "Blizzard_TimerunningUtil",
    "Blizzard_Communities",
];

pub(crate) const BLIZZARD_TOKEN_UI_ADDON: &str = "Blizzard_TokenUI";

pub(crate) fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

pub(crate) fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for addon_name in BLIZZARD_ADDONS {
        crate::common::load_required_blizzard_addon(&env, &ui, addon_name);
    }
    // PTR ships this startup addon (AllowLoad: Both); Mainline QuestMapFrame calls its globals.
    #[cfg(feature = "client-ptr")]
    crate::common::load_required_blizzard_addon(&env, &ui, "Blizzard_GamepadSmartNavigation");

    load_token_ui(&env);
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

pub(crate) fn fire_startup_events(env: &WowLuaEnv) {
    crate::common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    crate::common::call_global_if_present(env, "RequestTimePlayed");
    crate::common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "PLAYER_LEAVING_WORLD",
    ] {
        let _ = env.fire_event(event);
    }
}
