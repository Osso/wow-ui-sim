//! Regression test: verify key frame positions match the origin/master baseline.
//!
//! Loads all Blizzard addons at 1024x768, fires startup events (same sequence
//! as the dump-tree/screenshot headless path), then checks that important UI
//! elements are positioned correctly. Expected values from origin/master dump-tree.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Replicate the headless startup path from main.rs `run_headless_startup`.
fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

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
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());

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
    let id = state.widgets.get_id_by_name(name)
        .unwrap_or_else(|| panic!("Frame '{}' not found", name));
    let rect = compute_frame_rect(&state.widgets, id, 1024.0, 768.0);
    (rect.x, rect.y, rect.width, rect.height)
}

/// Assert frame position and size within tolerance (±1px for rounding).
fn assert_frame_rect(env: &WowLuaEnv, name: &str, ex: f32, ey: f32, ew: f32, eh: f32) {
    let (x, y, w, h) = frame_rect(env, name);
    let tol = 1.0;
    assert!(
        (x - ex).abs() <= tol && (y - ey).abs() <= tol
            && (w - ew).abs() <= tol && (h - eh).abs() <= tol,
        "{name}: expected ({ex}, {ey}, {ew}x{eh}), got ({x}, {y}, {w}x{h})"
    );
}

// ── Player / Target / Group frames ────────────────────────────

#[test]
fn player_frame_position() {
    let env = setup_env();
    // PlayerFrame [Button] (232x100) visible LOW:2 x=0, y=418
    assert_frame_rect(&env, "PlayerFrame", 0.0, 418.0, 232.0, 100.0);
}

#[test]
fn target_frame_position() {
    let env = setup_env();
    // TargetFrame [Button] (232x100) hidden LOW:500 x=792, y=418
    assert_frame_rect(&env, "TargetFrame", 792.0, 418.0, 232.0, 100.0);
}

#[test]
fn focus_frame_position() {
    let env = setup_env();
    // FocusFrame [Button] (174x75) [stored=232x100] hidden LOW:500 x=850, y=494 scale=0.75
    assert_frame_rect(&env, "FocusFrame", 850.0, 494.0, 174.0, 75.0);
}

#[test]
fn pet_frame_position() {
    let env = setup_env();
    // PetFrame [Button] (120x49) hidden LOW:4 x=93, y=535
    assert_frame_rect(&env, "PetFrame", 93.0, 535.0, 120.0, 49.0);
}

#[test]
fn paladin_power_bar_position() {
    let env = setup_env();
    // PaladinPowerBarFrame [Frame] (150x43) visible LOW:5 x=73, y=490
    assert_frame_rect(&env, "PaladinPowerBarFrame", 73.0, 490.0, 150.0, 43.0);
}

#[test]
fn party_frame_position() {
    let env = setup_env();
    // PartyFrame [Frame] (120x244) visible LOW:2 x=22, y=147
    assert_frame_rect(&env, "PartyFrame", 22.0, 147.0, 120.0, 244.0);
}

#[test]
fn compact_party_frame_position() {
    let env = setup_env();
    // CompactPartyFrame [Frame] (90x224) hidden LOW:3 x=22, y=147
    assert_frame_rect(&env, "CompactPartyFrame", 22.0, 147.0, 90.0, 224.0);
}

// ── HUD elements ──────────────────────────────────────────────

#[test]
fn minimap_position() {
    let env = setup_env();
    // Minimap [Minimap] (198x198) visible LOW:4 x=807, y=44
    assert_frame_rect(&env, "Minimap", 807.0, 44.0, 198.0, 198.0);
}

#[test]
fn objective_tracker_position() {
    let env = setup_env();
    // ObjectiveTrackerFrame [Frame] (260x400) visible LOW:3 x=759, y=271
    assert_frame_rect(&env, "ObjectiveTrackerFrame", 759.0, 271.0, 260.0, 400.0);
}

#[test]
fn bags_bar_position() {
    let env = setup_env();
    // BagsBar [Frame] (208x47) visible MEDIUM:2 x=810, y=672
    assert_frame_rect(&env, "BagsBar", 810.0, 672.0, 208.0, 47.0);
}

#[test]
fn micro_menu_position() {
    let env = setup_env();
    // MicroMenu [Frame] (329x40) visible MEDIUM:3 x=629, y=717
    assert_frame_rect(&env, "MicroMenu", 629.0, 717.0, 329.0, 40.0);
}

#[test]
fn buff_frame_position() {
    let env = setup_env();
    // BuffFrame [Frame] (400x135) visible LOW:2 x=369, y=10
    assert_frame_rect(&env, "BuffFrame", 369.0, 10.0, 400.0, 135.0);
}

#[test]
fn debuff_frame_position() {
    let env = setup_env();
    // DebuffFrame [Frame] (280x90) visible LOW:2 x=474, y=155
    assert_frame_rect(&env, "DebuffFrame", 474.0, 155.0, 280.0, 90.0);
}

#[test]
fn chat_frame_position() {
    let env = setup_env();
    // ChatFrame1 [MessageFrame] (430x170) visible LOW:5 x=35, y=548
    assert_frame_rect(&env, "ChatFrame1", 35.0, 548.0, 430.0, 170.0);
}

#[test]
fn action_button_1_position() {
    let env = setup_env();
    // ActionButton1 [CheckButton] (0x0) [stored=45x45] visible MEDIUM:52 x=512, y=768
    let (x, _y, _w, _h) = frame_rect(&env, "ActionButton1");
    assert!((x - 512.0).abs() <= 1.0, "ActionButton1 x: expected 512, got {x}");
}

#[test]
fn micro_menu_container_position() {
    let env = setup_env();
    // MicroMenuContainer [Frame] (389x45) visible MEDIUM:2 x=629, y=717
    assert_frame_rect(&env, "MicroMenuContainer", 629.0, 717.0, 389.0, 45.0);
}
