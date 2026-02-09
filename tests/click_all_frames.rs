//! Test that clicking on visible, clickable frames produces no Lua errors.
//!
//! Loads all Blizzard addons once, then clicks frames grouped by UI area.
//! Each group reports independently so failures are easy to locate.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetType;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
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

/// Load all Blizzard addons, fire startup events, return the environment.
fn setup_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }
    env.apply_post_load_workarounds();
    install_test_error_handler(&env);

    let lua = env.lua();
    let _ = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    );
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_edit_mode_layouts_updated();
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    );
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_on_update(0.016);
    let _ = env.process_timers();

    // Drain startup errors — we only care about click errors
    drain_test_errors(&env);

    env
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
    env.send_click(id).ok();
    drain_test_errors(env)
        .into_iter()
        .map(|e| format!("[{name}] {e}"))
        .collect()
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
            .all_ids()
            .into_iter()
            .filter_map(|id| {
                let frame = state.widgets.get(id)?;
                let name = frame.name.as_ref()?;
                if !name.starts_with(prefix) || !frame.visible {
                    return None;
                }
                match frame.widget_type {
                    WidgetType::Button
                    | WidgetType::CheckButton
                    | WidgetType::Frame => {}
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
    }

    (frames.len(), all_errors)
}

/// Run a named test group, collecting errors into the report.
fn run_group(
    env: &WowLuaEnv,
    label: &str,
    names: &[&str],
    report: &mut Vec<String>,
) {
    let (clicked, errors) = click_group(env, names);
    eprintln!("[{label}] Clicked {clicked}/{} frames", names.len());
    report.extend(errors);
}

/// Run a prefix-based test group, collecting errors into the report.
fn run_prefix(
    env: &WowLuaEnv,
    label: &str,
    prefix: &str,
    report: &mut Vec<String>,
) {
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

const UNIT_FRAMES: &[&str] = &[
    "PlayerFrame",
    "TargetFrame",
    "FocusFrame",
    "PetFrame",
];

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

const OBJECTIVE_TRACKER: &[&str] = &[
    "ObjectiveTrackerFrame",
    "QuestObjectiveTracker",
];

const CLOSE_BUTTONS: &[&str] = &[
    "AddonListCloseButton",
    "SettingsCloseButton",
];

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
const KNOWN_ERROR_COUNT: usize = 43;

#[test]
fn test_click_all_frames() {
    let env = setup_full_ui();
    let report = click_all_groups(&env);
    let count = report.len();

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

    if count < KNOWN_ERROR_COUNT {
        panic!(
            "Click error count improved from {KNOWN_ERROR_COUNT} to {count}! \
             Update KNOWN_ERROR_COUNT to {count} to lock in the improvement."
        );
    }
}
