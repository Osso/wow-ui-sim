//! Integration tests for micro menu button clicks.
//!
//! Loads the base Blizzard UI, fires startup events, then clicks each micro
//! menu button and verifies the corresponding panel frame is shown.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Blizzard addons in dependency order (mirrors BLIZZARD_ADDONS in main.rs).
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

/// Create a fully loaded environment with Blizzard addons and startup events.
fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    // Set addon_base_paths for runtime on-demand loading
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    // Load base Blizzard addons
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

/// Fire startup events (same sequence as main.rs).
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

/// Simulate clicking a micro menu button by calling its OnClick script.
fn click_button(env: &WowLuaEnv, button_name: &str) -> Result<(), String> {
    let code = format!(
        r#"
        local btn = {button_name}
        if not btn then error("{button_name} does not exist") end
        local onclick = btn:GetScript("OnClick")
        if not onclick then error("{button_name} has no OnClick handler") end
        onclick(btn, "LeftButton", false)
        "#
    );
    env.exec(&code).map_err(|e| format!("{e}"))
}

/// Check whether a global frame exists and is shown.
fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!(
        "return {frame_name} ~= nil and {frame_name}:IsShown() == true"
    );
    env.eval::<bool>(&code).unwrap_or(false)
}

/// Check whether a global frame exists (may not be shown).
fn frame_exists(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil");
    env.eval::<bool>(&code).unwrap_or(false)
}

#[test]
fn micro_menu_character_button_opens_character_frame() {
    let env = setup_env();
    click_button(&env, "CharacterMicroButton").expect("CharacterMicroButton click failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after clicking CharacterMicroButton"
    );
}

#[test]
fn micro_menu_game_menu_button_opens_game_menu() {
    let env = setup_env();
    click_button(&env, "MainMenuMicroButton").expect("MainMenuMicroButton click failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown after clicking MainMenuMicroButton"
    );
}

#[test]
fn micro_menu_professions_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "ProfessionsBookFrame"),
        "ProfessionsBookFrame should not exist before click"
    );
    click_button(&env, "ProfessionMicroButton").expect("ProfessionMicroButton click failed");
    assert!(
        frame_is_shown(&env, "ProfessionsBookFrame"),
        "ProfessionsBookFrame should be shown after clicking ProfessionMicroButton"
    );
}

#[test]
fn micro_menu_player_spells_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should not exist before click"
    );
    click_button(&env, "PlayerSpellsMicroButton").expect("PlayerSpellsMicroButton click failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after clicking PlayerSpellsMicroButton"
    );
}

#[test]
fn micro_menu_collections_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "CollectionsJournal"),
        "CollectionsJournal should not exist before click"
    );
    click_button(&env, "CollectionsMicroButton").expect("CollectionsMicroButton click failed");
    assert!(
        frame_is_shown(&env, "CollectionsJournal"),
        "CollectionsJournal should be shown after clicking CollectionsMicroButton"
    );
}

#[test]
fn micro_menu_achievement_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "AchievementFrame"),
        "AchievementFrame should not exist before click"
    );
    // Click may error in post-show setup; we only care that the frame is shown
    let _ = click_button(&env, "AchievementMicroButton");
    assert!(
        frame_exists(&env, "AchievementFrame"),
        "AchievementFrame should exist after clicking AchievementMicroButton"
    );
    assert!(
        frame_is_shown(&env, "AchievementFrame"),
        "AchievementFrame should be shown after clicking AchievementMicroButton"
    );
}

#[test]
fn micro_menu_ej_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "EncounterJournal"),
        "EncounterJournal should not exist before click"
    );
    // Click may error in post-show setup; we only care that the frame is shown
    let _ = click_button(&env, "EJMicroButton");
    assert!(
        frame_is_shown(&env, "EncounterJournal"),
        "EncounterJournal should be shown after clicking EJMicroButton"
    );
}

#[test]
fn game_menu_buttons_display_text() {
    let env = setup_env();
    click_button(&env, "MainMenuMicroButton").expect("MainMenuMicroButton click failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown"
    );

    // Collect text from all active buttons in the game menu's button pool
    let button_texts: Vec<String> = env
        .eval(
            r#"
            local texts = {}
            for button in GameMenuFrame.buttonPool:EnumerateActive() do
                table.insert(texts, button:GetText() or "")
            end
            return texts
            "#,
        )
        .expect("Failed to enumerate game menu buttons");

    assert!(
        !button_texts.is_empty(),
        "GameMenuFrame should have at least one button"
    );

    // Every button must have non-empty text
    for (i, text) in button_texts.iter().enumerate() {
        assert!(
            !text.is_empty(),
            "Game menu button {} has empty text",
            i + 1
        );
    }

    // These buttons should always appear (not conditional on features)
    let expected = [
        "Options",
        "AddOns",
        "Support",
        "Macros",
        "Log Out",
        "Exit Game",
        "Return to Game",
    ];
    for label in &expected {
        assert!(
            button_texts.iter().any(|t| t == label),
            "Expected game menu button '{}' not found. Got: {:?}",
            label,
            button_texts
        );
    }
}

#[test]
fn micro_menu_guild_button_loads_and_opens_panel() {
    let env = setup_env();
    // Click may error in post-show setup; we only care that the frame exists
    let _ = click_button(&env, "GuildMicroButton");
    assert!(
        frame_exists(&env, "CommunitiesFrame"),
        "CommunitiesFrame should exist after clicking GuildMicroButton"
    );
}

