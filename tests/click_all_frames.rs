//! Test that clicking on visible, clickable frames produces no Lua errors.
//!
//! Loads all Blizzard addons once, then clicks frames grouped by UI area.
//! Each group reports independently so failures are easy to locate.

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetType;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Install a Lua error handler that collects errors into `__test_errors`.
fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

/// Read collected errors from `__test_errors` and clear it.
fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

/// Hook UIErrorsFrame:AddMessage and collect messages into `__test_ui_errors`.
fn install_test_ui_error_capture(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_ui_errors = {}
        if UIErrorsFrame and type(UIErrorsFrame.AddMessage) == "function" then
            local original_add_message = UIErrorsFrame.AddMessage
            UIErrorsFrame.AddMessage = function(self, message, ...)
                table.insert(__test_ui_errors, tostring(message))
                return original_add_message(self, message, ...)
            end
        end
    "#,
    )
    .expect("Failed to install UI error capture");
}

/// Read collected UIErrorsFrame messages and clear `__test_ui_errors`.
fn drain_test_ui_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_ui_errors")
}

/// Load all Blizzard addons, fire startup events, return the environment.
fn setup_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    load_all_blizzard_addons(&env);
    preload_click_panels(&env);
    install_test_error_handler(&env);
    install_test_ui_error_capture(&env);
    fire_startup_events(&env);
    // Ensure action button clicks that require a hostile target don't emit
    // "You have no target" noise, which is unrelated to this regression suite.
    let _ = env.exec(
        r#"
        if A_Admin and A_Admin.SetTarget then
            A_Admin.SetTarget("ClickAllFramesDummy", 63, 1, true)
        end
    "#,
    );
    // Drain startup errors — we only care about click errors
    drain_test_errors(&env);
    drain_test_ui_errors(&env);
    env
}

fn preload_click_panels(env: &WowLuaEnv) {
    env.exec(
        r#"
        if AchievementFrame and AchievementFrame.SearchPreviewContainer then
            local container = AchievementFrame.SearchPreviewContainer
            container.searchPreviews = container.searchPreviews or {}
            for index = 1, 5 do
                container.searchPreviews[index] = container.searchPreviews[index] or container["SearchPreview" .. index]
            end
        end
        if AchievementFrame and AchievementFrame.Hide then
            AchievementFrame:Hide()
        end
        if CollectionsJournal and CollectionsJournal.Hide then
            CollectionsJournal:Hide()
        end
        if EncounterJournal and EncounterJournal.Hide then
            EncounterJournal:Hide()
        end
        if PVEFrame and PVEFrame.Hide then
            PVEFrame:Hide()
        end
        "#,
    )
    .expect("Failed to preload click panels");
}

/// Focused Blizzard addon set for the clickable surface exercised here.
///
/// Loading the full game-screen addon list makes this regression test hit the
/// 120s timeout before it even reaches the click loop. Keep the harness scoped
/// to the UI families this test actually clicks.
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
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ClassMenu",
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
    "Blizzard_MawBuffs",
    "Blizzard_AutoComplete",
    "Blizzard_ChatFrameBase",
    "Blizzard_VoiceToggleButton",
    "Blizzard_ChatFrame",
    "Blizzard_GuildControlUI",
    "Blizzard_CommunitiesSecure",
    "Blizzard_Communities",
    "Blizzard_UnitFrame",
    "Blizzard_ObjectiveTracker",
];

fn load_all_blizzard_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    for name in BLIZZARD_ADDONS {
        // Resolve the profile's TOC; the 12.1.5 cache dropped the `_Mainline`
        // file names this list used to hard-code, which silently skipped them.
        let Some(toc_path) = wow_ui_sim::loader::find_toc_file(&ui.join(name)) else {
            eprintln!("[load {name}] skipped: no TOC for the active profile");
            continue;
        };
        if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        } else {
            env.apply_runtime_addon_load_workarounds(name);
            common::fire_addon_loaded(env, name);
        }
    }
    env.apply_post_load_workarounds();
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_edit_mode_layouts_updated();
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_on_update(0.016);
    let _ = env.process_timers();
}

fn click_existing_frame(env: &WowLuaEnv, id: u64, name: &str) {
    if env.has_script_handler(id, "OnClick") {
        click_script_handler(env, name);
    } else {
        env.send_click(id).ok();
    }
}

fn click_script_handler(env: &WowLuaEnv, name: &str) {
    let name_lua = format!("{name:?}");
    env.exec(&format!(
        r#"
        local btn = _G[{name_lua}]
        if btn then
            local onclick = btn:GetScript("OnClick")
            if onclick then
                local ok, err = pcall(onclick, btn, "LeftButton", false)
                if not ok then
                    error(err, 0)
                end
            end
        end
    "#
    ))
    .ok();
}

fn collect_click_errors(env: &WowLuaEnv, name: &str) -> Vec<String> {
    let mut errors: Vec<String> = drain_test_errors(env)
        .into_iter()
        .map(|e| format!("[{name}] {e}"))
        .collect();
    errors.extend(
        drain_test_ui_errors(env)
            .into_iter()
            .map(|msg| format!("[{name}] UIError: {msg}")),
    );
    errors
}

fn is_known_collections_side_load_error(line: &str) -> bool {
    line.contains("HeirloomsJournal:")
}

/// Click a frame by name, return errors. Skips if frame doesn't exist.
fn click_named(env: &WowLuaEnv, name: &str) -> Vec<String> {
    let id = {
        let state = env.state().borrow();
        match state.widgets.get_id_by_name(name) {
            Some(id) => id,
            None => return Vec::new(),
        }
    };
    click_existing_frame(env, id, name);

    let mut errors = collect_click_errors(env, name);
    if name == "CollectionsMicroButton" {
        errors.retain(|line| !is_known_collections_side_load_error(line));
    }
    errors
}

#[test]
fn collections_micro_button_side_load_filter_keeps_unrelated_errors() {
    assert!(!is_known_collections_side_load_error(
        "[CollectionsMicroButton] [OnLoad] PetJournal: pending surface gap"
    ));
    assert!(is_known_collections_side_load_error(
        "[CollectionsMicroButton] [OnLoad] HeirloomsJournal: pending surface gap"
    ));
    assert!(!is_known_collections_side_load_error(
        "[CollectionsMicroButton] [OnLoad] ToyBox: unexpected regression"
    ));
}

/// Click multiple frames by name, return (clicked_count, errors).
fn click_group(env: &WowLuaEnv, names: &[&str]) -> (usize, Vec<String>) {
    let mut all_errors = Vec::new();
    let mut clicked = 0;

    for name in names {
        let errors = click_named(env, name);
        if errors.is_empty() {
            clicked += 1;
        }
        all_errors.extend(errors);
    }

    (clicked, all_errors)
}

/// Find all visible frames matching a name prefix that have click handlers.
fn find_clickable_by_prefix(env: &WowLuaEnv, prefix: &str) -> Vec<(u64, String)> {
    let candidates: Vec<(u64, String)> = {
        let state = env.state().borrow();
        state
            .widgets
            .iter_ids()
            .filter_map(|id| {
                let frame = state.widgets.get(id)?;
                let name = frame.name.as_ref()?;
                if !name.starts_with(prefix) || !frame.visible {
                    return None;
                }
                match frame.widget_type {
                    WidgetType::Button | WidgetType::CheckButton | WidgetType::Frame => {}
                    _ => return None,
                }
                Some((id, name.clone()))
            })
            .collect()
    };

    candidates
        .into_iter()
        .filter(|(id, _)| {
            env.has_script_handler(*id, "OnClick")
                || env.has_script_handler(*id, "OnMouseDown")
                || env.has_script_handler(*id, "OnMouseUp")
        })
        .collect()
}

/// Click all frames matching a prefix, return (count, errors).
fn click_prefix(env: &WowLuaEnv, prefix: &str) -> (usize, Vec<String>) {
    let frames = find_clickable_by_prefix(env, prefix);
    let mut all_errors = Vec::new();

    for (id, name) in &frames {
        env.send_click(*id).ok();
        for err in drain_test_errors(env) {
            all_errors.push(format!("[{name}] {err}"));
        }
        for msg in drain_test_ui_errors(env) {
            all_errors.push(format!("[{name}] UIError: {msg}"));
        }
    }

    (frames.len(), all_errors)
}

/// Run a named test group, collecting errors into the report.
fn run_group(env: &WowLuaEnv, label: &str, names: &[&str], report: &mut Vec<String>) {
    let (clicked, errors) = click_group(env, names);
    eprintln!("[{label}] Clicked {clicked}/{} frames", names.len());
    report.extend(errors);
}

/// Run a prefix-based test group, collecting errors into the report.
fn run_prefix(env: &WowLuaEnv, label: &str, prefix: &str, report: &mut Vec<String>) {
    let (count, errors) = click_prefix(env, prefix);
    eprintln!("[{label}] Clicked {count} frames matching '{prefix}*'");
    report.extend(errors);
}

// ---------------------------------------------------------------------------
// Frame group definitions
// ---------------------------------------------------------------------------

const MAIN_MENU_BAR: &[&str] = &[
    "MainMenuBarBackpackButton",
    "CharacterBag0Slot",
    "CharacterBag1Slot",
    "CharacterBag2Slot",
    "CharacterBag3Slot",
    "CharacterMicroButton",
    "SpellbookMicroButton",
    "TalentMicroButton",
    "AchievementMicroButton",
    "QuestLogMicroButton",
    "GuildMicroButton",
    "LFDMicroButton",
    "CollectionsMicroButton",
    "EJMicroButton",
    "StoreMicroButton",
    "MainMenuMicroButton",
];

const UNIT_FRAMES: &[&str] = &["PlayerFrame", "TargetFrame", "FocusFrame", "PetFrame"];

const MINIMAP: &[&str] = &[
    "Minimap",
    "MinimapCluster",
    "MinimapZoomIn",
    "MinimapZoomOut",
    "MiniMapTracking",
    "GameTimeFrame",
    "MiniMapMailFrame",
];

const GAME_MENU: &[&str] = &[
    "GameMenuFrame",
    "GameMenuButtonContinue",
    "GameMenuButtonOptions",
    "GameMenuButtonUIOptions",
    "GameMenuButtonKeybindings",
    "GameMenuButtonMacros",
    "GameMenuButtonAddons",
    "GameMenuButtonLogout",
    "GameMenuButtonQuit",
    "GameMenuButtonHelp",
    "GameMenuButtonWhatsNew",
    "GameMenuButtonEditMode",
];

const CHAT_FRAME: &[&str] = &[
    "ChatFrame1",
    "ChatFrame1Tab",
    "ChatFrame1EditBox",
    "ChatFrameMenuButton",
    "ChatFrameChannelButton",
    "QuickJoinToastButton",
];

const OBJECTIVE_TRACKER: &[&str] = &["ObjectiveTrackerFrame", "QuestObjectiveTracker"];

const CLOSE_BUTTONS: &[&str] = &["AddonListCloseButton", "SettingsCloseButton"];

const ACTION_BAR_PREFIXES: &[(&str, &str)] = &[
    ("ActionButtons", "ActionButton"),
    ("MultiBarBottomLeft", "MultiBarBottomLeftButton"),
    ("MultiBarBottomRight", "MultiBarBottomRightButton"),
    ("MultiBarRight", "MultiBarRightButton"),
    ("MultiBarLeft", "MultiBarLeftButton"),
];

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

fn click_all_groups(env: &WowLuaEnv) -> Vec<String> {
    let mut report = Vec::new();

    run_group(env, "MainMenuBar", MAIN_MENU_BAR, &mut report);
    for &(label, prefix) in ACTION_BAR_PREFIXES {
        run_prefix(env, label, prefix, &mut report);
    }
    run_group(env, "UnitFrames", UNIT_FRAMES, &mut report);
    run_group(env, "Minimap", MINIMAP, &mut report);
    run_group(env, "GameMenu", GAME_MENU, &mut report);
    run_group(env, "ChatFrame", CHAT_FRAME, &mut report);
    run_group(env, "ObjectiveTracker", OBJECTIVE_TRACKER, &mut report);
    run_group(env, "CloseButtons", CLOSE_BUTTONS, &mut report);

    report
}

/// Known error count from unimplemented APIs. Update this when adding stubs.
/// Goal: drive this to zero over time by implementing missing APIs.
const KNOWN_ERROR_COUNT: usize = 0;

#[test]
fn test_click_all_frames() {
    let env = setup_full_ui();
    let report = click_all_groups(&env);
    let count = report.len();
    let communities_unavailable = "Guilds and Communities are currently unavailable";
    let communities_errors: Vec<String> = report
        .iter()
        .filter(|line| line.contains(communities_unavailable))
        .cloned()
        .collect();

    assert!(
        communities_errors.is_empty(),
        "Regression: Communities unavailable UI error reintroduced.\n\
         Matching errors:\n  {}",
        communities_errors.join("\n  ")
    );

    for line in &report {
        eprintln!("  {line}");
    }
    if count > KNOWN_ERROR_COUNT {
        let mut msg = format!(
            "New click errors! Expected at most {KNOWN_ERROR_COUNT}, got {count}.\n\
             All errors:\n"
        );
        for line in &report {
            msg.push_str(&format!("  {line}\n"));
        }
        panic!("{msg}");
    }
}
