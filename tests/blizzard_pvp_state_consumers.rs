#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_game_ui() -> WowLuaEnv {
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let addons = discover_blizzard_addons_for_screen(&root, ScreenKind::Game);
    assert_selected(&addons, "Blizzard_QueueStatusFrame");
    assert_selected(&addons, "Blizzard_UnitFrame");

    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![root];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in addons {
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn assert_selected(addons: &[(String, PathBuf)], addon: &str) {
    assert!(
        addons.iter().any(|(name, _)| name == addon),
        "{addon} must be selected for the Game screen"
    );
}

#[test]
fn battlefield_queue_consumer_reads_each_supplied_queue_index() {
    let env = load_game_ui();
    env.exec(
        r#"
        A_Admin.SetBattlefieldQueue(1, "queued", "Queue One")
        QueueStatusFrame:Update()
        assert(QueueStatusButton:IsShown(), "queue index 1 must appear")

        A_Admin.SetBattlefieldQueue(2, "queued", "Queue Two")
        QueueStatusFrame:Update()
        assert(QueueStatusButton:IsShown(), "queue index 2 must appear")
        "#,
    )
    .expect("original QueueStatusFrame consumer must read the supplied queue state");
}

#[test]
fn arena_party_visibility_consumer_reads_arena_state() {
    let env = load_game_ui();
    env.exec(
        r#"
        A_Admin.SetBattlefieldArena(false)
        assert(not ShouldShowArenaParty(), "party frames must not use arena layout outside arena")

        A_Admin.SetBattlefieldArena(true)
        assert(ShouldShowArenaParty(), "original arena visibility helper must detect arena")
        "#,
    )
    .expect("original GroupFrameVisibility consumer must read supplied arena state");
}

#[test]
fn player_frame_timer_consumer_reads_timer_state() {
    let env = load_game_ui();
    env.exec(
        r#"
        A_Admin.SetPVPTimerRunning(false)
        PlayerFrame_OnEvent(PlayerFrame, "PVP_TIMER_UPDATE")
        assert(not PlayerPVPTimerText:IsShown(), "stopped PvP timer must be hidden")

        A_Admin.SetPVPTimerRunning(true)
        PlayerFrame_OnEvent(PlayerFrame, "PVP_TIMER_UPDATE")
        assert(PlayerPVPTimerText:IsShown(), "running PvP timer must be shown")
        "#,
    )
    .expect("original PlayerFrame consumer must read supplied timer state");
}
