//! Shared setup for keybinding panel detail integration tests.

use crate::common;
use crate::token_ui_fixtures::load_token_ui;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;

/// Blizzard addons in dependency order (same as micro_menu.rs).
const BLIZZARD_ADDONS: &[&str] = &[
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
    "Blizzard_GameTooltip",
    "Blizzard_StaticPopup",
    "Blizzard_GameMenuEsc",
    "Blizzard_StaticPopup_Game",
    "Blizzard_TransmogShared",
    "Blizzard_Communities",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_Menu",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_ActionBar",
    "Blizzard_ActionBarController",
    "Blizzard_GameMenu",
    "Blizzard_UIWidgets",
    "Blizzard_Minimap",
    "Blizzard_AddOnList",
    "Blizzard_TimerunningUtil",
];

pub(crate) fn setup_env() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);
        let ui = default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced");

        {
            let mut state = env.state().borrow_mut();
            state.addon_base_paths = vec![ui.clone()];
        }

        for addon_name in BLIZZARD_ADDONS {
            common::load_required_blizzard_addon(&env, &ui, addon_name);
            env.apply_runtime_addon_load_workarounds(addon_name);
        }
        common::load_click_binding_bootstrap(&env, &ui);

        load_token_ui(&env);
        env.apply_post_load_workarounds();
        fire_startup_events(&env);
        env
    })
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::call_global_if_present(env, "RequestTimePlayed");
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "PLAYER_LEAVING_WORLD",
    ] {
        let _ = env.fire_event(event);
    }
}

pub(crate) fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsShown() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

pub(crate) fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

pub(crate) fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}
