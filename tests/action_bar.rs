//! Integration tests for main action bar visibility after startup.
//!
//! Verifies that loading Blizzard addons and firing startup events results
//! in the MainActionBar and its 12 ActionButton children being visible.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

fn blizzard_toc(addon: &str, toc_name: &str) -> PathBuf {
    blizzard_ui_dir().join(addon).join(toc_name)
}

/// Blizzard addons needed for the action bar, in dependency order.
const ACTION_BAR_ADDONS: &[(&str, &str)] = &[
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
    ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame_Mainline.toc"),
    ("Blizzard_UIPanelTemplates", "Blizzard_UIPanelTemplates_Mainline.toc"),
    ("Blizzard_FrameXMLBase", "Blizzard_FrameXMLBase_Mainline.toc"),
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    ("Blizzard_AccessibilityTemplates", "Blizzard_AccessibilityTemplates.toc"),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent_Mainline.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI_Mainline.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    ("Blizzard_UIParentPanelManager", "Blizzard_UIParentPanelManager_Mainline.toc"),
    ("Blizzard_Settings_Shared", "Blizzard_Settings_Shared_Mainline.toc"),
    ("Blizzard_SettingsDefinitions_Shared", "Blizzard_SettingsDefinitions_Shared.toc"),
    ("Blizzard_SettingsDefinitions_Frame", "Blizzard_SettingsDefinitions_Frame_Mainline.toc"),
    ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil_Mainline.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML_Mainline.toc"),
    ("Blizzard_UIPanels_Game", "Blizzard_UIPanels_Game_Mainline.toc"),
    ("Blizzard_MapCanvasSecureUtil", "Blizzard_MapCanvasSecureUtil.toc"),
    ("Blizzard_MapCanvas", "Blizzard_MapCanvas.toc"),
    ("Blizzard_SharedMapDataProviders", "Blizzard_SharedMapDataProviders_Mainline.toc"),
    ("Blizzard_WorldMap", "Blizzard_WorldMap_Mainline.toc"),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
];

/// Load addons, fire startup events, and apply post-startup fixups.
fn env_with_action_bar() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    for (name, toc) in ACTION_BAR_ADDONS {
        let toc_path = blizzard_toc(name, toc);
        if !toc_path.exists() {
            continue;
        }
        if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
            eprintln!("{name} failed: {e}");
        }
    }

    fire_startup_events(&env);
    env
}

/// Replicate the startup event sequence from main.rs / app.rs.
fn fire_startup_events(env: &WowLuaEnv) {
    let lua = env.lua();
    let _ = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    );
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    if let Ok(f) = lua.globals().get::<mlua::Function>("RequestTimePlayed") {
        let _ = f.call::<()>(());
    }
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    );
    let _ = env.fire_edit_mode_layouts_updated();

    // WoW's C++ engine fires ACTIONBAR_SHOWGRID on startup to show empty slots.
    let _ = env.fire_event("ACTIONBAR_SHOWGRID");

    for event in ["UPDATE_BINDINGS", "DISPLAY_SIZE_CHANGED", "UI_SCALE_CHANGED"] {
        let _ = env.fire_event(event);
    }
}

#[test]
fn test_main_action_bar_visible_after_startup() {
    let env = env_with_action_bar();

    let visible: bool = env
        .eval("return MainActionBar ~= nil and MainActionBar:IsVisible()")
        .unwrap();
    assert!(visible, "MainActionBar should be visible after startup");
}

#[test]
fn test_action_buttons_visible_after_startup() {
    let env = env_with_action_bar();

    let count: i32 = env
        .eval(r#"
            local n = 0
            for i = 1, 12 do
                local btn = _G["ActionButton" .. i]
                if btn and btn:IsVisible() then
                    n = n + 1
                end
            end
            return n
        "#)
        .unwrap();
    assert_eq!(count, 12, "All 12 ActionButtons should be visible");
}

#[test]
fn test_action_buttons_have_showgrid_attribute() {
    let env = env_with_action_bar();

    let all_have_grid: bool = env
        .eval(r#"
            for i = 1, 12 do
                local btn = _G["ActionButton" .. i]
                if not btn then return false end
                local grid = btn:GetAttribute("showgrid")
                if not grid or grid <= 0 then return false end
            end
            return true
        "#)
        .unwrap();
    assert!(all_have_grid, "All ActionButtons should have showgrid > 0");
}
