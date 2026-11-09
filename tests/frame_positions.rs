//! Regression test: verify key frame positions in the default startup canvas.
//! Raid-warning geometry and managed-container identity are grounded in the
//! live capture and causal replay documented in frame-position-baseline-drift.md.
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
    ("paladin_power_bar",          "PaladinPowerBarFrame",           314.0,  945.0,  150.0,  43.0, 1.0),
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
    ("raid_boss_emote_anchor",     "PrivateRaidBossEmoteFrameAnchor",400.0,  182.0,  800.0,  80.0, 1.0),
    ("critical_encounter_warnings","CriticalEncounterWarnings",      500.0,   40.0,  600.0,  48.0, 1.0),
    ("medium_encounter_warnings",  "MediumEncounterWarnings",        525.0,   90.0,  550.0,  36.0, 1.0),
    ("minor_encounter_warnings",   "MinorEncounterWarnings",         550.0,  130.0,  500.0,  36.0, 1.0),
    // Managed containers
    ("right_managed_container",    "RightManagedFrameContainer",     1335.0, 260.0, 260.0, 847.0, 1.0),
    // Casting bar (hidden — no active cast; attached to PlayerFrame via PlayerFrame_AttachCastBar)
    ("casting_bar",                "PlayerCastingBarFrame",              696.0,  594.5,  208.0,  11.0, 1.0),
];

/// ActionButton1 only checks x position (y/size depend on bar layout).
fn check_action_button(env: &WowLuaEnv, expected_x: f32) {
    let (x, _y, _w, _h) = frame_rect(env, "ActionButton1");
    let alpha = frame_alpha(env, "ActionButton1");
    assert!(
        (x - expected_x).abs() <= 1.0,
        "ActionButton1 x: expected {expected_x}, got {x}"
    );
    assert!(
        (alpha - 1.0).abs() <= 0.02,
        "ActionButton1 alpha: expected 1.0, got {alpha}"
    );
}

fn assert_native_minimap_inputs(env: &WowLuaEnv) {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    let side = match ACTIVE {
        ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary => 140,
        _ => 198,
    };
    env.exec(&format!(
        r#"
        assert(UIParent:GetEffectiveScale() == 1, "default canvas scale")
        assert(Minimap:GetParent() == MinimapCluster.MinimapContainer, "native minimap parent")
        assert(Minimap:GetWidth() == {side} and Minimap:GetHeight() == {side}, "native minimap dimensions")
        assert(MinimapCluster.MinimapContainer:GetScale() == 1, "default minimap scale")
        local point, relative, relativePoint, x, y = Minimap:GetPoint(1)
        assert(point == "CENTER" and relative == MinimapCluster.MinimapContainer and relativePoint == "CENTER")
        assert(x == 0 and y == 0, "native minimap center")
        point, relative, relativePoint, x, y = MinimapCluster:GetPoint(1)
        assert(point == "TOPRIGHT" and relative == UIParent and relativePoint == "TOPRIGHT")
        assert(x == 0 and y == 0, "native minimap preset anchor")
        "#
    ))
    .expect("native minimap fixture inputs");
}

fn expected_position(test: TestCase) -> TestCase {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    let (name, frame, ..) = test;
    match (ACTIVE, name) {
        (ClientProfile::Ptr, "minimap") => (name, frame, 1389.5, 44.0, 198.0, 198.0, 1.0),
        (ClientProfile::Ptr, "minimap_cluster") => (name, frame, 1357.0, 0.0, 243.0, 252.5, 1.0),
        (ClientProfile::Ptr, "micro_menu_container") => {
            (name, frame, 1265.0, 1154.0, 329.0, 40.0, 1.0)
        }
        (ClientProfile::WowForever, "minimap") => (name, frame, 1393.5, 44.0, 198.0, 198.0, 1.0),
        (ClientProfile::WowForever, "minimap_cluster") => {
            (name, frame, 1365.0, 0.0, 235.0, 256.0, 1.0)
        }
        (ClientProfile::WowForever, "micro_menu")
        | (ClientProfile::WowForever, "micro_menu_container") => {
            (name, frame, 752.0, 1154.0, 329.0, 40.0, 1.0)
        }
        (ClientProfile::WowForever, "main_action_bar") => {
            (name, frame, 185.5, 1153.0, 562.0, 45.0, 1.0)
        }
        (ClientProfile::Mists, "paladin_power_bar") => {
            // Mists/PaladinPowerBar.xml:49-51: 136x39, TOP to PlayerFrame
            // BOTTOM (25,35). Classic/PlayerFrame.xml:14: height100.
            // x = 268 + 232/2 + 25 - 136/2; y = 850 + 100 - 35.
            (name, "PaladinPowerBar", 341.0, 915.0, 136.0, 39.0, 1.0)
        }
        (ClientProfile::Mists, "minimap")
        | (ClientProfile::Era, "minimap")
        | (ClientProfile::Anniversary, "minimap") => (name, frame, 1441.5, 22.0, 140.0, 140.0, 1.0),
        (ClientProfile::Mists, "minimap_cluster")
        | (ClientProfile::Era, "minimap_cluster")
        | (ClientProfile::Anniversary, "minimap_cluster") => {
            (name, frame, 1423.0, 0.0, 177.0, 165.0, 1.0)
        }
        (ClientProfile::Mists, "party_frame")
        | (ClientProfile::Era, "party_frame")
        | (ClientProfile::Anniversary, "party_frame") => {
            // Classic/Blizzard_CompactRaidFrameManager.lua:153-155 + XML:62:
            // right = -182 + 200 = 18. Classic/EditModePresetLayouts.lua:292-297:
            // TOPLEFT to manager TOPRIGHT (0,12): y = 140 - 12 = 128.
            // Classic/PartyFrameTemplates.xml:70-71: members128x53;
            // Shared/PartyFrame.xml:9,35: height = 4*53 + 3*10 + 2 = 244.
            (name, frame, 18.0, 128.0, 128.0, 244.0, 1.0)
        }
        (ClientProfile::Mists, "compact_party_frame")
        | (ClientProfile::Era, "compact_party_frame")
        | (ClientProfile::Anniversary, "compact_party_frame") => {
            // Shared/CompactPartyFrame.lua:5: parent PartyFrame; same origin.
            // Keep its separate 98x234 compact-group size, not party width128.
            (name, frame, 18.0, 128.0, 98.0, 234.0, 1.0)
        }
        (ClientProfile::Mists, "chat_edit_box")
        | (ClientProfile::Era, "chat_edit_box")
        | (ClientProfile::Anniversary, "chat_edit_box") => {
            (name, frame, 30.0, 1152.0, 440.0, 32.0, 0.35)
        }
        (ClientProfile::Era, "casting_bar") | (ClientProfile::Anniversary, "casting_bar") => {
            (name, frame, 702.5, 593.5, 195.0, 13.0, 1.0)
        }
        (ClientProfile::Mists, "raid_boss_emote_anchor")
        | (ClientProfile::Era, "raid_boss_emote_anchor")
        | (ClientProfile::Anniversary, "raid_boss_emote_anchor") => {
            (name, "RaidBossEmoteFrame", 544.0, 252.0, 512.0, 80.0, 1.0)
        }
        (ClientProfile::Mists, "right_managed_container")
        | (ClientProfile::Era, "right_managed_container")
        | (ClientProfile::Anniversary, "right_managed_container") => (
            name,
            "UIParentRightManagedFrameContainer",
            test.2,
            test.3,
            test.4,
            test.5,
            test.6,
        ),
        _ => test,
    }
}

fn expects_missing_frame(test: &str) -> bool {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    match test {
        "paladin_power_bar" => matches!(
            ACTIVE,
            ClientProfile::WowForever | ClientProfile::Era | ClientProfile::Anniversary
        ),
        "critical_encounter_warnings"
        | "medium_encounter_warnings"
        | "minor_encounter_warnings" => matches!(
            ACTIVE,
            ClientProfile::WowForever
                | ClientProfile::Mists
                | ClientProfile::Era
                | ClientProfile::Anniversary
        ),
        _ => false,
    }
}

fn assert_frame_absent(env: &WowLuaEnv, name: &str) {
    let global_exists: bool = env
        .eval(&format!("return _G[{name:?}] ~= nil"))
        .expect("read native frame global");
    let widget_exists = env.state().borrow().widgets.get_id_by_name(name).is_some();
    assert!(
        !global_exists && !widget_exists,
        "{name} is excluded from this profile's selected Blizzard UI source"
    );
}

fn assert_chat_edit_box_layout(env: &WowLuaEnv) {
    let (chat_x, chat_y, chat_width, chat_height) = frame_rect(env, "ChatFrame1");
    env.exec(
        r#"
        local point, relative, relativePoint, x, y = ChatFrame1EditBox:GetPoint(1)
        assert(point == "TOPLEFT" and relative == ChatFrame1 and relativePoint == "BOTTOMLEFT")
        assert(x == -5 and y == -2)
        point, relative, relativePoint, x, y = ChatFrame1EditBox:GetPoint(2)
        assert(point == "TOPRIGHT" and relative == ChatFrame1 and relativePoint == "BOTTOMRIGHT")
        assert(x == 5 and y == -2)
        "#,
    )
    .expect("check native chat edit-box anchors");
    assert_frame_rect(
        env,
        "ChatFrame1EditBox",
        chat_x - 5.0,
        chat_y + chat_height + 2.0,
        chat_width + 10.0,
        32.0,
        0.35,
    );
}

fn assert_objective_tracker_layout(env: &WowLuaEnv) {
    let (container_x, container_y, container_width, container_height) =
        frame_rect(env, "RightManagedFrameContainer");
    assert_frame_rect(
        env,
        "ObjectiveTrackerFrame",
        container_x,
        container_y + 15.0,
        container_width,
        container_height - 15.0,
        1.0,
    );
}

fn assert_chat_dock_layout(env: &WowLuaEnv) {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    let (chat_x, chat_y, chat_width, _) = frame_rect(env, "ChatFrame1");
    let (expected_x, expected_y, expected_width, anchor_check) = match ACTIVE {
        ClientProfile::Ptr | ClientProfile::WowForever => {
            let (has_scrollbar, quick_button_height): (bool, f32) = env
                .eval(
                    r#"
                    return ChatFrame1.ScrollBar ~= nil,
                        (CombatLogQuickButtonFrame and CombatLogQuickButtonFrame:GetHeight()) or 0
                    "#,
                )
                .expect("read native chat background extents");
            let scrollbar_width = if has_scrollbar { 8.0 } else { 0.0 };
            (
                chat_x - 2.0,
                chat_y - 29.0 - quick_button_height,
                chat_width + 9.0 + scrollbar_width,
                r#"
                local point, relative, relativePoint, x, y = GeneralDockManager:GetPoint(1)
                assert(point == "BOTTOMLEFT" and relative == ChatFrame1.Background and relativePoint == "TOPLEFT")
                assert(x == 0 and y == 0)
                "#,
            )
        }
        ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary => (
            chat_x,
            chat_y - 32.0,
            chat_width,
            r#"
            local point, relative, relativePoint, x, y = GeneralDockManager:GetPoint(1)
            assert(point == "BOTTOMLEFT" and relative == ChatFrame1 and relativePoint == "TOPLEFT")
            assert(x == 0 and y == 6)
            "#,
        ),
        ClientProfile::Retail | ClientProfile::Wrath => {
            unreachable!("Retail keeps its committed baseline; Wrath is outside this lane")
        }
    };
    env.exec(anchor_check)
        .expect("check native chat dock anchor");
    assert_frame_rect(
        env,
        "GeneralDockManager",
        expected_x,
        expected_y,
        expected_width,
        26.0,
        1.0,
    );
}

/// Mists Classic/EditModePresetLayouts.lua:237-279 gives the modern preset
/// anchors; Classic/PlayerFrame.xml:14 and TargetFrame.xml:147 give 232x100.
/// At 1600x1200: Player=(800-300-232,1200-250-100),
/// Target=(800+300,1200-250-100), Focus=(800+520,1200-265-100).
fn assert_mists_unit_frame_anchor(env: &WowLuaEnv, test: &str) {
    let (frame, point, relative_point, x, y) = match test {
        "player_frame" => ("PlayerFrame", "BOTTOMRIGHT", "BOTTOM", -300, 250),
        "target_frame" => ("TargetFrame", "BOTTOMLEFT", "BOTTOM", 300, 250),
        "focus_frame" => ("FocusFrame", "BOTTOMLEFT", "BOTTOM", 520, 265),
        _ => return,
    };
    env.exec(&format!(
        r#"
        local frame = {frame}
        assert(frame:GetParent() == UIParent, "native unit-frame parent")
        assert(frame:GetScale() == 1, "native default frame scale")
        local point, relative, relativePoint, x, y = frame:GetPoint(1)
        assert(point == "{point}" and relative == UIParent and relativePoint == "{relative_point}", "native unit-frame anchor")
        assert(x == {x} and y == {y}, "native unit-frame anchor offsets")
        "#
    ))
    .unwrap_or_else(|error| panic!("{test} native anchor: {error}"));
}

fn assert_layout_state(env: &WowLuaEnv, test: &str) {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    if matches!(test, "minimap" | "minimap_cluster") {
        assert_native_minimap_inputs(env);
    }
    if ACTIVE == ClientProfile::Mists {
        assert_mists_unit_frame_anchor(env, test);
    }
    let assertion = match (ACTIVE, test) {
        (
            ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary,
            "raid_boss_emote_anchor",
        ) => {
            r#"
            local point, relative, relativePoint = RaidBossEmoteFrame:GetPoint(1)
            assert(point == "TOP" and relative == RaidWarningFrame and relativePoint == "BOTTOM")
            "#
        }
        (
            ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever,
            "raid_boss_emote_anchor",
        ) => {
            r#"
            assert(UIParent:GetEffectiveScale() == 1)
            assert(RaidWarningFrame:GetLowestMessage() == nil)
            assert(not DeadlyDebuffFrame:IsShown())
            local point, relative, relativePoint, x, y = PrivateRaidBossEmoteFrameAnchor:GetPoint(1)
            assert(point == "TOP" and relative == RaidWarningFrame and relativePoint == "TOP")
            assert(x == 0 and y == 0)
            "#
        }
        (
            ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary,
            "party_frame" | "compact_party_frame",
        ) => {
            r#"
            assert(CompactRaidFrameManager.collapsed == true)
            local point, relative, relativePoint, x, y = CompactRaidFrameManager:GetPoint(1)
            assert(point == "TOPLEFT" and relative == UIParent and relativePoint == "TOPLEFT")
            assert(x == -182 and y == -140)
            point, relative, relativePoint, x, y = PartyFrame:GetPoint(1)
            assert(point == "TOPLEFT" and relative == CompactRaidFrameManager and relativePoint == "TOPRIGHT")
            assert(x == 0 and y == 12)
            "#
        }
        (ClientProfile::Mists, "paladin_power_bar") => {
            r#"
            assert(PaladinPowerBar:GetParent() == PlayerFrame)
            assert(PaladinPowerBar:GetWidth() == 136 and PaladinPowerBar:GetHeight() == 39)
            local point, relative, relativePoint, x, y = PaladinPowerBar:GetPoint(1)
            assert(point == "TOP" and relative == PlayerFrame and relativePoint == "BOTTOM")
            assert(x == 25 and y == 35)
            "#
        }
        (ClientProfile::Retail, "objective_tracker") => {
            r#"
            assert(ObjectiveTrackerFrame:IsInDefaultPosition(), "tracker default-position state")
            assert(ObjectiveTrackerFrame:GetParent() == RightManagedFrameContainer, "tracker parent")
            local _, relative, _, _, offsetY = ObjectiveTrackerFrame:GetPoint(1)
            assert(relative == RightManagedFrameContainer and offsetY == 0, "tracker anchor")
            assert(math.abs(ObjectiveTrackerFrame:GetHeight() - RightManagedFrameContainer:GetHeight()) < 0.001, "tracker height")
            "#
        }
        (ClientProfile::Ptr | ClientProfile::WowForever, "objective_tracker") => {
            r#"
            assert(ObjectiveTrackerFrame:IsInDefaultPosition(), "tracker default-position state")
            assert(ObjectiveTrackerFrame:GetParent() == RightManagedFrameContainer, "tracker parent")
            local _, relative, _, _, offsetY = ObjectiveTrackerFrame:GetPoint(1)
            assert(relative == RightManagedFrameContainer and offsetY == -15, "tracker native top padding")
            assert(math.abs(ObjectiveTrackerFrame:GetHeight() - (RightManagedFrameContainer:GetHeight() - 15)) < 0.001, "tracker native top-padded height")
            "#
        }
        _ => return,
    };
    env.exec(assertion)
        .unwrap_or_else(|error| panic!("{test} fixture state: {error}"));
}

fn run_tests(env: &WowLuaEnv) -> (usize, usize) {
    let mut passed = 0;
    let mut failed = 0;

    for test in POSITION_TESTS {
        let (name, frame, ..) = *test;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if expects_missing_frame(name) {
                assert_frame_absent(env, frame);
            } else if name == "general_dock_manager" {
                if wow_ui_sim::client_profile::ACTIVE
                    == wow_ui_sim::client_profile::ClientProfile::Retail
                {
                    let (_, frame, ex, ey, ew, eh, ea) = expected_position(*test);
                    assert_frame_rect(env, frame, ex, ey, ew, eh, ea);
                } else {
                    assert_chat_dock_layout(env);
                }
            } else if name == "objective_tracker"
                && matches!(
                    wow_ui_sim::client_profile::ACTIVE,
                    wow_ui_sim::client_profile::ClientProfile::Ptr
                        | wow_ui_sim::client_profile::ClientProfile::WowForever
                )
            {
                assert_layout_state(env, name);
                assert_objective_tracker_layout(env);
            } else if name == "chat_edit_box"
                && matches!(
                    wow_ui_sim::client_profile::ACTIVE,
                    wow_ui_sim::client_profile::ClientProfile::Mists
                        | wow_ui_sim::client_profile::ClientProfile::Era
                        | wow_ui_sim::client_profile::ClientProfile::Anniversary
                )
            {
                assert_chat_edit_box_layout(env);
            } else {
                assert_layout_state(env, name);
                let (_, frame, ex, ey, ew, eh, ea) = expected_position(*test);
                assert_frame_rect(env, frame, ex, ey, ew, eh, ea);
            }
        }));
        report_result(&result, name, &mut passed, &mut failed);
    }

    let expected_action_button_x = match wow_ui_sim::client_profile::ACTIVE {
        wow_ui_sim::client_profile::ClientProfile::WowForever => 185.5,
        _ => 519.0,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_action_button(env, expected_action_button_x);
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
