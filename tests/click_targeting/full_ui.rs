use std::path::PathBuf;
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::globals::global_frames;
use wow_ui_sim::lua_api::WowLuaEnv;

use super::common;

const CLICK_TARGETING_ADDONS: &[&str] = &[
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
    "Blizzard_ManagedFrameSystem",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIParentUtil",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_Menu",
    "Blizzard_Minimap",
    "Blizzard_StaticPopup",
    "Blizzard_TimeManager",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_SpellDiminishUI",
    "Blizzard_ActionBar",
    "Blizzard_UnitFrame",
];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap_or_else(|_| {
        wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("Blizzard UI cache should be available")
    })
}

pub(crate) fn env_with_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    for name in CLICK_TARGETING_ADDONS {
        let addon_dir = ui.join(name);
        let toc_path = find_toc_file(&addon_dir)
            .unwrap_or_else(|| panic!("active retail TOC for {name} must resolve"));
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("load {name} from {}: {err}", toc_path.display()));
        env.apply_runtime_addon_load_workarounds(name);
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    let _ = global_frames::hide_runtime_hidden_frames(&*env.rilua());
    env
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for ev in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(ev);
    }
    common::fire_player_entering_world(env, true, false);
    let _ = env.fire_edit_mode_layouts_updated();
    for ev in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(ev);
    }
}

pub(crate) fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#,
    )
    .expect("install error handler");
}

pub(crate) fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}
