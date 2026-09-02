//! Regression test: verify key frame positions match the origin/master baseline.
//!
//! Loads all Blizzard addons at 1600x1200, fires startup events (same sequence
//! as the dump-tree/screenshot headless path), then checks that important UI
//! elements are positioned correctly.
//!
//! Uses `harness = false` with a custom main to load the Blizzard UI once and
//! run all position checks against the shared environment.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn create_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for (name, toc_path) in &discover_blizzard_addons(&ui) {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();

    // Same sequence as run_headless_startup in main.rs
    fire_startup_events(&env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(&env);
    fire_one_on_update_tick(&env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());

    // Allow timer-driven layout callbacks to become due (real wall clock via Instant)
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Extra update ticks — drain timers and fire OnUpdate (same as main.rs)
    for _ in 0..3 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(&env);
        process_pending_timers(&env);
    }

    env
}

/// Query a frame's computed rect: (x, y, width, height) via layout's compute_frame_rect.
fn frame_rect(env: &WowLuaEnv, name: &str) -> (f32, f32, f32, f32) {
    use wow_ui_sim::iced_app::layout::compute_frame_rect;
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("Frame '{}' not found", name));
    let rect = compute_frame_rect(&state.widgets, id, 1600.0, 1200.0);
    (rect.x, rect.y, rect.width, rect.height)
}

fn frame_alpha(env: &WowLuaEnv, name: &str) -> f32 {
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("Frame '{}' not found", name));
    state.widgets.get(id).map(|f| f.alpha).unwrap_or(1.0)
}

/// Assert frame position, size, and alpha within tolerance (±1px position, ±0.02 alpha).
fn assert_frame_rect(env: &WowLuaEnv, name: &str, ex: f32, ey: f32, ew: f32, eh: f32, ea: f32) {
    let (x, y, w, h) = frame_rect(env, name);
    let alpha = frame_alpha(env, name);
    let tol = 1.0;
    let atol = 0.02;
    assert!(
        (x - ex).abs() <= tol
            && (y - ey).abs() <= tol
            && (w - ew).abs() <= tol
            && (h - eh).abs() <= tol
            && (alpha - ea).abs() <= atol,
        "{name}: expected ({ex}, {ey}, {ew}x{eh}, alpha={ea}), got ({x}, {y}, {w}x{h}, alpha={alpha})"
    );
}

/// (test_name, frame_name, x, y, width, height, alpha)
type TestCase = (&'static str, &'static str, f32, f32, f32, f32, f32);

/// Expected frame positions at 1600x1200 after full startup.
///
/// Each entry is checked as an individual named test by the custom main().
/// PetFrame is omitted (hidden — no pet in simulator).
#[rustfmt::skip]
const POSITION_TESTS: &[TestCase] = &[
    // Player / Target / Group frames
    ("player_frame",               "PlayerFrame",                    268.0,  850.0,  232.0, 100.0, 1.0),
    ("target_frame",               "TargetFrame",                   1100.0,  850.0,  232.0, 100.0, 1.0),
    ("focus_frame",                "FocusFrame",                    1320.0,  835.0, 232.0, 100.0, 1.0),
    // Managed by PlayerBottomManagedFrameContainer: UIParent.lua:167 skips a
    // frame that answers false to IsInDefaultPosition, and plain frames (no
    // systemInfo) answered false before; the client has no method on them.
    ("paladin_power_bar",          "PaladinPowerBarFrame",           341.5,  922.0,  150.0,  43.0, 1.0),
    ("party_frame",                "PartyFrame",                      22.0,  147.0,  120.0, 244.0, 1.0),
    ("compact_party_frame",        "CompactPartyFrame",               22.0,  147.0,   98.0, 234.0, 1.0),
    // HUD elements
    ("minimap",                    "Minimap",                       1391.0,   44.0,  198.0, 198.0, 1.0),
    ("minimap_cluster",            "MinimapCluster",                1360.0,    0.0,  240.0, 252.0, 1.0),
    ("objective_tracker",          "ObjectiveTrackerFrame",         1335.0,  260.0,  260.0, 847.5, 1.0),
    ("bags_bar",                   "BagsBar",                       1386.0, 1104.0,  208.0,  47.0, 1.0),
    ("micro_button_bags_bar",      "MicroButtonAndBagsBar",         1362.0, 1114.0,  232.0,  80.0, 1.0),
    ("micro_menu",                 "MicroMenu",                     1265.0, 1154.0,  329.0,  40.0, 1.0),
    ("micro_menu_container",       "MicroMenuContainer",            1205.0, 1149.0,  389.0,  45.0, 1.0),
    ("buff_frame",                 "BuffFrame",                      945.0,   10.0,  400.0, 135.0, 1.0),
    ("debuff_frame",               "DebuffFrame",                   1050.0,  155.0,  280.0,  90.0, 1.0),
    // Chat
    ("chat_frame",                 "ChatFrame1",                      35.0,  980.0,  430.0, 170.0, 1.0),
    ("chat_edit_box",              "ChatFrame1EditBox",                30.0, 1152.0,  466.0,  32.0, 0.35),
    ("general_dock_manager",       "GeneralDockManager",              35.0,  951.0,  430.0,  26.0, 1.0),
    // Action bars
    ("main_action_bar",            "MainActionBar",                  519.0, 1110.0,  562.0,  45.0, 1.0),
    ("status_tracking_bar",        "StatusTrackingBarManager",       514.0, 1166.0,  571.0,  34.0, 1.0),
    // Overlay / warning frames
    ("ui_errors_frame",            "UIErrorsFrame",                  544.0,  122.0,  512.0,  60.0, 1.0),
    // 12.1.0 moved the anchor into Blizzard_RaidWarning (RaidWarning.xml:27,
    // Size 800x80); the 512-wide definition is the legacy FrameXML copy.
    ("raid_boss_emote_anchor",     "PrivateRaidBossEmoteFrameAnchor",400.0,  182.0,  800.0,  80.0, 1.0),
    ("critical_encounter_warnings","CriticalEncounterWarnings",      500.0,   40.0,  600.0,  48.0, 1.0),
    ("medium_encounter_warnings",  "MediumEncounterWarnings",        525.0,   90.0,  550.0,  36.0, 1.0),
    ("minor_encounter_warnings",   "MinorEncounterWarnings",         550.0,  130.0,  500.0,  36.0, 1.0),
    // Managed containers
    // 12.1.0's UIParent.xml:83 leaves UIParentRightManagedFrameContainer an
    // empty shell (no size, no anchors); the live container the tracker
    // anchors to is Blizzard_ManagedFrameSystem's RightManagedFrameContainer.
    ("right_managed_container",    "RightManagedFrameContainer",         1335.0, 260.0, 260.0, 847.0, 1.0),
    // Casting bar (hidden — no active cast; attached to PlayerFrame via PlayerFrame_AttachCastBar)
    ("casting_bar",                "PlayerCastingBarFrame",              696.0,  594.5,  208.0,  11.0, 1.0),
];

/// ActionButton1 only checks x position (y/size depend on bar layout).
fn check_action_button(env: &WowLuaEnv) {
    let (x, _y, _w, _h) = frame_rect(env, "ActionButton1");
    let alpha = frame_alpha(env, "ActionButton1");
    assert!(
        (x - 519.0).abs() <= 1.0,
        "ActionButton1 x: expected 519, got {x}"
    );
    assert!(
        (alpha - 1.0).abs() <= 0.02,
        "ActionButton1 alpha: expected 1.0, got {alpha}"
    );
}

fn run_tests(env: &WowLuaEnv) -> (usize, usize) {
    let mut passed = 0;
    let mut failed = 0;

    for (name, frame, ex, ey, ew, eh, ea) in POSITION_TESTS {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_frame_rect(env, frame, *ex, *ey, *ew, *eh, *ea);
        }));
        report_result(&result, name, &mut passed, &mut failed);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_action_button(env);
    }));
    report_result(
        &result,
        "action_button_1_position",
        &mut passed,
        &mut failed,
    );

    (passed, failed)
}

fn report_result(
    result: &Result<(), Box<dyn std::any::Any + Send>>,
    name: &str,
    passed: &mut usize,
    failed: &mut usize,
) {
    match result {
        Ok(()) => {
            *passed += 1;
            eprintln!("test {name} ... ok");
        }
        Err(e) => {
            *failed += 1;
            let msg = e
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("(unknown panic)");
            eprintln!("test {name} ... FAILED\n  {msg}");
        }
    }
}

fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let env = create_env();
        let (passed, failed) = run_tests(&env);
        let _ = tx.send((passed, failed));
    });

    match rx.recv_timeout(std::time::Duration::from_secs(120)) {
        Ok((passed, failed)) => {
            handle.join().expect("test thread panicked");
            let total = passed + failed;
            eprintln!(
                "\ntest result: {}. {passed} passed; {failed} failed; 0 ignored; \
                      0 measured; 0 filtered out",
                if failed == 0 { "ok" } else { "FAILED" }
            );
            if failed > 0 {
                std::process::exit(1);
            }
            assert_eq!(total, 28, "Expected 28 tests, ran {total}");
        }
        Err(_) => {
            eprintln!("\ntest timed out after 120s");
            std::process::exit(1);
        }
    }
}
