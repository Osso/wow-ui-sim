#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
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
    crate::common::load_game_startup_inventory(&env);
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
        assert(GetMaxBattlefieldID() == 0, "unsupplied battlefield maximum defaults to zero")
        A_Admin.SetMaxBattlefieldID(2)
        A_Admin.SetBattlefieldQueue(1, true)
        QueueStatusFrame:Update()
        assert(QueueStatusButton:IsShown(), "queue index 1 must appear")

        A_Admin.SetBattlefieldQueue(2, true)
        QueueStatusFrame:Update()
        assert(GetMaxBattlefieldID() == 2, "maximum battlefield ID must use supplied state")
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
        A_Admin.SetBattlefieldArena(false, false)
        local arenaActive, hasArenaRecord = IsActiveBattlefieldArena()
        assert(not arenaActive and not hasArenaRecord,
            "inactive arena must return both false values")
        assert(not ShouldShowArenaParty(), "party frames must not use arena layout outside arena")

        A_Admin.SetBattlefieldArena(true, true)
        arenaActive, hasArenaRecord = IsActiveBattlefieldArena()
        assert(arenaActive and hasArenaRecord,
            "active arena must return active state and current-record state")
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
        A_Admin.SetManualClock(10)
        A_Admin.SetPlayerPresent(false)
        assert(GetPVPTimer() == 0, "missing player must produce zero")

        A_Admin.SetPlayerPresent(true)
        A_Admin.SetPVPTimer(false, 0)
        assert(GetPVPTimer() == 301000, "cleared PvP status must use the native fallback")
        assert(not IsPVPTimerRunning(), "cleared PvP status must not be running")
        PlayerFrame_OnEvent(PlayerFrame, "PVP_TIMER_UPDATE")
        assert(not PlayerPVPTimerText:IsShown(), "stopped PvP timer must be hidden")

        A_Admin.SetPVPTimer(true, 15000)
        assert(IsPVPTimerRunning(), "supplied PvP status must be running")
        assert(GetPVPTimer() == 6000, "unexpired deadline must use the simulator clock")
        PlayerFrame_OnEvent(PlayerFrame, "PVP_TIMER_UPDATE")
        assert(PlayerPVPTimerText:IsShown(), "running PvP timer must be shown")
        assert(PlayerPVPTimerText.timeLeft == 6000, "consumer must display computed remaining time")

        A_Admin.SetPVPTimer(true, 5000)
        assert(GetPVPTimer() == 999, "expired active timer must use the native active fallback")
        "#,
    )
    .expect("original PlayerFrame consumer must read supplied timer state");
}
