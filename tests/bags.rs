//! Tests for bag frames opening and displaying items.
//!
//! Loads the full Blizzard addon set, opens bags via keybind, and verifies
//! that item slots are populated with real item data from the mock inventory.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

const BLIZZARD_ADDONS: &[(&str, &str)] = &[
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
    ("Blizzard_GameMenu", "Blizzard_GameMenu_Mainline.toc"),
    ("Blizzard_UIWidgets", "Blizzard_UIWidgets_Mainline.toc"),
    ("Blizzard_Minimap", "Blizzard_Minimap_Mainline.toc"),
    ("Blizzard_AddOnList", "Blizzard_AddOnList.toc"),
    ("Blizzard_TimerunningUtil", "Blizzard_TimerunningUtil.toc"),
    ("Blizzard_Communities", "Blizzard_Communities_Mainline.toc"),
];

fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for (name, toc) in BLIZZARD_ADDONS {
        let toc_path = ui.join(name).join(toc);
        if !toc_path.exists() {
            continue;
        }
        if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

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
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "PLAYER_LEAVING_WORLD",
    ] {
        let _ = env.fire_event(event);
    }
}

fn install_test_error_handler(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_errors = {}
        seterrorhandler(function(msg)
            table.insert(__test_errors, tostring(msg))
        end)
    "#,
    )
    .expect("Failed to install test error handler");
}

fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    let lua = env.lua();
    let table: mlua::Table = match lua.globals().get("__test_errors") {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut errors = Vec::new();
    for entry in table.sequence_values::<String>() {
        if let Ok(msg) = entry {
            errors.push(msg);
        }
    }
    let _ = lua.load("__test_errors = {}").exec();
    errors
}

#[test]
fn test_container_frames_registered() {
    let env = setup_env();

    // Check ContainerFrameContainer.ContainerFrames population
    let count: i32 = env
        .eval(
            r#"
            local t = ContainerFrameContainer.ContainerFrames
            if type(t) ~= "table" then return -1 end
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
        "#,
        )
        .unwrap();
    assert_eq!(count, 6, "ContainerFrameContainer.ContainerFrames should have 6 entries");

    // Check individual frames exist
    for i in 1..=6 {
        let exists: bool = env
            .eval(&format!("return ContainerFrame{i} ~= nil"))
            .unwrap();
        assert!(exists, "ContainerFrame{i} should exist");
    }
}

/// Open all bags via pcall-protected ToggleAllBags, logging any Lua errors.
fn open_all_bags(env: &WowLuaEnv) {
    env.exec(
        r#"
        local ok, err = pcall(ToggleAllBags)
        if not ok then
            table.insert(__test_errors, "ToggleAllBags: " .. tostring(err))
        end
    "#,
    )
    .unwrap();

    let errors = drain_test_errors(env);
    for e in &errors {
        eprintln!("Lua error: {e}");
    }
}

/// Assert that at least one bag frame is visible.
fn assert_bag_frame_visible(env: &WowLuaEnv) {
    let bag_shown: bool = env
        .eval(
            "return (ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown()) \
             or (ContainerFrame1 and ContainerFrame1:IsShown())",
        )
        .unwrap();
    assert!(bag_shown, "A bag frame should be visible after ToggleAllBags");
}

/// Assert the backpack has the expected number of populated item slots.
fn assert_backpack_item_count(env: &WowLuaEnv, expected: i32) {
    let populated_slots: i32 = env
        .eval(
            r#"
            local count = 0
            for slot = 1, 16 do
                local info = C_Container.GetContainerItemInfo(0, slot)
                if info and info.itemID then
                    count = count + 1
                end
            end
            return count
        "#,
        )
        .unwrap();
    assert_eq!(populated_slots, expected, "Backpack populated slot count mismatch");
}

#[test]
fn test_bags_open_with_items() {
    let env = setup_env();
    install_test_error_handler(&env);

    open_all_bags(&env);
    assert_bag_frame_visible(&env);
    assert_backpack_item_count(&env, 6);

    // Verify item data fields are correct for a known slot
    let item_link: String = env
        .eval(r#"return C_Container.GetContainerItemInfo(0, 1).hyperlink"#)
        .unwrap();
    assert!(
        item_link.contains("Hearthstone"),
        "Slot 1 should contain Hearthstone, got: {item_link}",
    );

    // Verify empty slots return nil
    let empty: bool = env
        .eval("return C_Container.GetContainerItemInfo(0, 2) == nil")
        .unwrap();
    assert!(empty, "Slot 2 should be empty");
}
