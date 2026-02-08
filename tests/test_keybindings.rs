//! Integration tests for keybinding dispatch against the real Blizzard UI.
//!
//! Loads the full Blizzard addon set, fires startup events, then presses each
//! default keybind and verifies the corresponding panel frame is shown.
//!
//! These tests exercise the real Blizzard toggle functions (ToggleAllBags,
//! ToggleCharacter, etc.) — not stubs. Failures surface real missing APIs,
//! nil widget errors, and broken on-demand addon loads.

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Blizzard addons in dependency order (same as micro_menu.rs).
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

/// Check whether a global frame exists and is shown.
fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!(
        "return {frame_name} ~= nil and {frame_name}:IsShown() == true"
    );
    env.eval::<bool>(&code).unwrap_or(false)
}

/// Check whether a global frame exists.
#[allow(dead_code)]
fn frame_exists(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil");
    env.eval::<bool>(&code).unwrap_or(false)
}

// ── B → ToggleAllBags() ─────────────────────────────────────────────────

#[test]
fn keybind_b_opens_bags() {
    let env = setup_env();
    env.send_key_press("B").expect("B keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame1"),
        "A bag frame should be visible after pressing B"
    );
}

// ── BACKSPACE → ToggleBackpack() ────────────────────────────────────────

#[test]
fn keybind_backspace_opens_backpack() {
    let env = setup_env();
    env.send_key_press("BACKSPACE").expect("BACKSPACE keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame1"),
        "Backpack should be visible after pressing BACKSPACE"
    );
}

// ── F8 → ToggleBag(4) ──────────────────────────────────────────────────

#[test]
fn keybind_f8_opens_bag4() {
    let env = setup_env();
    env.send_key_press("F8").expect("F8 keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags"),
        "ContainerFrameCombinedBags should be shown after pressing F8"
    );
}

// ── C → ToggleCharacter("PaperDollFrame") ───────────────────────────────

#[test]
fn keybind_c_opens_character() {
    let env = setup_env();
    env.send_key_press("C").expect("C keybind failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after pressing C"
    );
}

// ── U → ToggleCharacter("ReputationFrame") ──────────────────────────────

#[test]
fn keybind_u_opens_reputation() {
    let env = setup_env();
    env.send_key_press("U").expect("U keybind failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after pressing U (reputation tab)"
    );
}

// ── S → PlayerSpellsUtil.ToggleSpellBookFrame() ─────────────────────────

#[test]
fn keybind_s_opens_spellbook() {
    let env = setup_env();
    env.send_key_press("S").expect("S keybind failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after pressing S"
    );
}

// ── N → PlayerSpellsUtil.ToggleClassTalentFrame() ───────────────────────

#[test]
fn keybind_n_opens_talents() {
    let env = setup_env();
    env.send_key_press("N").expect("N keybind failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after pressing N (talents tab)"
    );
}

// ── A → ToggleAchievementFrame() ────────────────────────────────────────

#[test]
fn keybind_a_opens_achievements() {
    let env = setup_env();
    env.send_key_press("A").expect("A keybind failed");
    assert!(
        frame_is_shown(&env, "AchievementFrame"),
        "AchievementFrame should be shown after pressing A"
    );
}

// ── L → PVEFrame_ToggleFrame() ──────────────────────────────────────────

#[test]
fn keybind_l_opens_group_finder() {
    let env = setup_env();
    env.send_key_press("L").expect("L keybind failed");
    assert!(
        frame_is_shown(&env, "PVEFrame"),
        "PVEFrame should be shown after pressing L"
    );
}

// ── O → ToggleFriendsFrame() ────────────────────────────────────────────

#[test]
fn keybind_o_opens_social() {
    let env = setup_env();
    env.send_key_press("O").expect("O keybind failed");
    assert!(
        frame_is_shown(&env, "FriendsFrame"),
        "FriendsFrame should be shown after pressing O"
    );
}

// ── J → ToggleGuildFrame() ──────────────────────────────────────────────

#[test]
fn keybind_j_opens_guild() {
    let env = setup_env();
    env.send_key_press("J").expect("J keybind failed");
    assert!(
        frame_is_shown(&env, "CommunitiesFrame"),
        "CommunitiesFrame should be shown after pressing J"
    );
}

// ── M → ToggleWorldMap() ────────────────────────────────────────────────

#[test]
fn keybind_m_opens_world_map() {
    let env = setup_env();
    env.send_key_press("M").expect("M keybind failed");
    assert!(
        frame_is_shown(&env, "WorldMapFrame"),
        "WorldMapFrame should be shown after pressing M"
    );
}

// ── ESCAPE → toggle GameMenuFrame ───────────────────────────────────────

#[test]
fn keybind_escape_opens_game_menu() {
    let env = setup_env();
    env.send_key_press("ESCAPE").expect("ESCAPE keybind failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown after pressing ESCAPE"
    );
}

#[test]
fn keybind_escape_closes_game_menu() {
    let env = setup_env();
    env.send_key_press("ESCAPE").expect("first ESCAPE failed");
    assert!(frame_is_shown(&env, "GameMenuFrame"));
    env.send_key_press("ESCAPE").expect("second ESCAPE failed");
    assert!(
        !frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be hidden after second ESCAPE"
    );
}
