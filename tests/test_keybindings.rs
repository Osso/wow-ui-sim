//! Integration tests for keybinding dispatch against the real Blizzard UI.
//!
//! Loads the full Blizzard addon set, fires startup events, then presses each
//! default keybind and verifies the corresponding panel frame is shown.
//!
//! These tests exercise the real Blizzard toggle functions (ToggleAllBags,
//! ToggleCharacter, etc.) — not stubs. Failures surface real missing APIs,
//! nil widget errors, and broken on-demand addon loads.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;

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

/// Install a Lua error handler that collects errors into `__test_errors`.
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

/// Read collected errors from `__test_errors` and clear it.
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

// ── B → ToggleAllBags() ─────────────────────────────────────────────────

#[test]
fn keybind_b_opens_bags() {
    let env = setup_env();
    env.send_key_press("B", None).expect("B keybind failed");
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
    env.send_key_press("BACKSPACE", None).expect("BACKSPACE keybind failed");
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
    env.send_key_press("F8", None).expect("F8 keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame5"),
        "A bag frame should be visible after pressing F8"
    );
}

// ── F9 → ToggleBag(3) ──────────────────────────────────────────────────

#[test]
fn keybind_f9_opens_bag3() {
    let env = setup_env();
    env.send_key_press("F9", None).expect("F9 keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame4"),
        "A bag frame should be visible after pressing F9"
    );
}

// ── F10 → ToggleBag(2) ─────────────────────────────────────────────────

#[test]
fn keybind_f10_opens_bag2() {
    let env = setup_env();
    env.send_key_press("F10", None).expect("F10 keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame3"),
        "A bag frame should be visible after pressing F10"
    );
}

// ── F11 → ToggleBag(1) ─────────────────────────────────────────────────

#[test]
fn keybind_f11_opens_bag1() {
    let env = setup_env();
    env.send_key_press("F11", None).expect("F11 keybind failed");
    assert!(
        frame_is_shown(&env, "ContainerFrameCombinedBags")
            || frame_is_shown(&env, "ContainerFrame2"),
        "A bag frame should be visible after pressing F11"
    );
}

// ── C → ToggleCharacter("PaperDollFrame") ───────────────────────────────

#[test]
fn keybind_c_opens_character() {
    let env = setup_env();
    env.send_key_press("C", None).expect("C keybind failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after pressing C"
    );
}

// ── U → ToggleCharacter("ReputationFrame") ──────────────────────────────

#[test]
fn keybind_u_opens_reputation() {
    let env = setup_env();
    env.send_key_press("U", None).expect("U keybind failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after pressing U (reputation tab)"
    );
}

// ── S → PlayerSpellsUtil.ToggleSpellBookFrame() ─────────────────────────

#[test]
fn keybind_s_opens_spellbook() {
    let env = setup_env();
    env.send_key_press("S", None).expect("S keybind failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after pressing S"
    );
    // ShowUIPanel should scale-to-fit and raise strata
    let scale: f64 = env
        .eval("return PlayerSpellsFrame:GetScale()")
        .expect("GetScale failed");
    assert!(
        scale < 1.0,
        "1618px-wide frame at 1024px screen should be scaled down, got {scale}"
    );
    let strata: String = env
        .eval("return PlayerSpellsFrame:GetFrameStrata()")
        .expect("GetFrameStrata failed");
    assert_eq!(strata, "HIGH", "ShowUIPanel should raise strata to HIGH");
}

// ── N → PlayerSpellsUtil.ToggleClassTalentFrame() ───────────────────────

#[test]
fn keybind_n_opens_talents() {
    let env = setup_env();
    env.send_key_press("N", None).expect("N keybind failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after pressing N (talents tab)"
    );
}

// ── A → ToggleAchievementFrame() ────────────────────────────────────────

#[test]
fn keybind_a_opens_achievements() {
    let env = setup_env();
    env.send_key_press("A", None).expect("A keybind failed");
    assert!(
        frame_is_shown(&env, "AchievementFrame"),
        "AchievementFrame should be shown after pressing A"
    );
}

// ── L → PVEFrame_ToggleFrame() ──────────────────────────────────────────

#[test]
fn keybind_l_opens_group_finder() {
    let env = setup_env();
    env.send_key_press("L", None).expect("L keybind failed");
    assert!(
        frame_is_shown(&env, "PVEFrame"),
        "PVEFrame should be shown after pressing L"
    );
}

// ── O → ToggleFriendsFrame() ────────────────────────────────────────────

#[test]
fn keybind_o_opens_social() {
    let env = setup_env();
    env.send_key_press("O", None).expect("O keybind failed");
    assert!(
        frame_is_shown(&env, "FriendsFrame"),
        "FriendsFrame should be shown after pressing O"
    );
}

// ── J → ToggleGuildFrame() ──────────────────────────────────────────────

#[test]
fn keybind_j_opens_guild() {
    let env = setup_env();
    env.send_key_press("J", None).expect("J keybind failed");
    assert!(
        frame_is_shown(&env, "CommunitiesFrame"),
        "CommunitiesFrame should be shown after pressing J"
    );
}

// ── M → ToggleWorldMap() ────────────────────────────────────────────────

#[test]
fn keybind_m_opens_world_map() {
    let env = setup_env();
    env.send_key_press("M", None).expect("M keybind failed");
    assert!(
        frame_is_shown(&env, "WorldMapFrame"),
        "WorldMapFrame should be shown after pressing M"
    );
}

// ── ESCAPE → toggle GameMenuFrame ───────────────────────────────────────

#[test]
fn keybind_escape_opens_game_menu() {
    let env = setup_env();
    env.send_key_press("ESCAPE", None).expect("ESCAPE keybind failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown after pressing ESCAPE"
    );
}

#[test]
fn keybind_escape_closes_game_menu() {
    let env = setup_env();
    env.send_key_press("ESCAPE", None).expect("first ESCAPE failed");
    assert!(frame_is_shown(&env, "GameMenuFrame"));
    env.send_key_press("ESCAPE", None).expect("second ESCAPE failed");
    assert!(
        !frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be hidden after second ESCAPE"
    );
}

// ── S → Spellbook panel opens without errors ─────────────────────────────

#[test]
fn keybind_s_opens_spellbook_no_errors() {
    let env = setup_env();
    install_test_error_handler(&env);

    env.send_key_press("S", None).expect("S keybind dispatch failed");

    let errors = drain_test_errors(&env);
    assert!(
        errors.is_empty(),
        "Opening spellbook produced {} Lua error(s):\n{}",
        errors.len(),
        errors.join("\n"),
    );
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after pressing S"
    );
}

// ── Target frame visibility tests (full addon load including Blizzard_UnitFrame) ──

/// Create environment with ALL Blizzard addons (including Blizzard_UnitFrame).
fn setup_full_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    let _ = global_frames::hide_runtime_hidden_frames(env.lua());
    env
}

#[test]
fn target_frame_shown_after_targeting() {
    let env = setup_full_env();
    install_test_error_handler(&env);

    assert!(
        frame_exists(&env, "TargetFrame"),
        "TargetFrame should exist after full addon load"
    );

    // TargetFrame starts hidden (hide_runtime_hidden_frames) or via startup;
    // ensure it's hidden before testing
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }

    // F1 = target self → TargetFrame should show
    env.send_key_press("F1", None).expect("F1 keybind failed");
    let _ = drain_test_errors(&env); // non-fatal errors from TargetFrame:Update()
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting self with F1"
    );

    // ESCAPE = clear target → TargetFrame should hide
    env.send_key_press("ESCAPE", None).expect("ESCAPE keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        !frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be hidden after clearing target with ESCAPE"
    );
}

#[test]
fn target_frame_shown_for_enemy() {
    let env = setup_full_env();
    install_test_error_handler(&env);

    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }

    // TAB = target nearest enemy → TargetFrame should show
    env.send_key_press("TAB", None).expect("TAB keybind failed");
    let _ = drain_test_errors(&env); // non-fatal errors from TargetFrame:Update()
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting enemy with TAB"
    );
}

// ── F2–F5 → TargetUnit('party1')–('party4') ─────────────────────────────

#[test]
fn keybind_f2_targets_party1() {
    let env = setup_full_env();
    install_test_error_handler(&env);
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }
    env.send_key_press("F2", None).expect("F2 keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting party1 with F2"
    );
}

#[test]
fn keybind_f3_targets_party2() {
    let env = setup_full_env();
    install_test_error_handler(&env);
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }
    env.send_key_press("F3", None).expect("F3 keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting party2 with F3"
    );
}

#[test]
fn keybind_f4_targets_party3() {
    let env = setup_full_env();
    install_test_error_handler(&env);
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }
    env.send_key_press("F4", None).expect("F4 keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting party3 with F4"
    );
}

#[test]
fn keybind_f5_targets_party4() {
    let env = setup_full_env();
    install_test_error_handler(&env);
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }
    env.send_key_press("F5", None).expect("F5 keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting party4 with F5"
    );
}

// ── F6 → TargetUnit('enemy1') ────────────────────────────────────────────

#[test]
fn keybind_f6_targets_enemy() {
    let env = setup_full_env();
    install_test_error_handler(&env);
    if frame_is_shown(&env, "TargetFrame") {
        env.exec("TargetFrame:Hide()").unwrap();
    }
    env.send_key_press("F6", None).expect("F6 keybind failed");
    let _ = drain_test_errors(&env);
    assert!(
        frame_is_shown(&env, "TargetFrame"),
        "TargetFrame should be shown after targeting enemy with F6"
    );
}
