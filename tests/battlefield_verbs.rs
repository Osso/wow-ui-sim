//! Integration tests for `src/lua_api/globals/battlefield_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BattlefieldStatus;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn fired(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .any(|e| e.name == name)
}

// ── JoinBattlefield ───────────────────────────────────────────────────────────

#[test]
fn join_battlefield_queues_player_and_fires_status() {
    let env = env();
    env.exec("C_PvP.JoinBattlefield(3)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.battlefield_queue.status, BattlefieldStatus::Queued);
    assert_eq!(st.battlefield_queue.index, 3);
    assert!(st.battlefield_queue.name.contains('3'));
    drop(st);
    assert!(fired(&env, "UPDATE_BATTLEFIELD_STATUS"));
}

#[test]
fn join_battlefield_defaults_index_to_one() {
    let env = env();
    env.exec("C_PvP.JoinBattlefield()").unwrap();
    assert_eq!(env.state().borrow().battlefield_queue.index, 1);
}

// ── AcceptBattlefieldPort ─────────────────────────────────────────────────────

#[test]
fn accept_battlefield_port_true_activates_queue() {
    let env = env();
    env.exec("C_PvP.JoinBattlefield(2)").unwrap();
    env.exec("AcceptBattlefieldPort(2, 1)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.battlefield_queue.status, BattlefieldStatus::Active);
}

#[test]
fn accept_battlefield_port_false_clears_queue() {
    let env = env();
    env.exec("C_PvP.JoinBattlefield(2)").unwrap();
    env.exec("AcceptBattlefieldPort(2, nil)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.battlefield_queue.status, BattlefieldStatus::None);
    assert_eq!(st.battlefield_queue.index, 0);
    assert!(st.battlefield_queue.name.is_empty());
}

// ── LeaveBattlefield ──────────────────────────────────────────────────────────

#[test]
fn leave_battlefield_clears_and_fires_status() {
    let env = env();
    env.exec(
        "C_PvP.JoinBattlefield(5)
               LeaveBattlefield()",
    )
    .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.battlefield_queue.status, BattlefieldStatus::None);
    assert_eq!(st.battlefield_queue.index, 0);
    drop(st);
    assert!(fired(&env, "UPDATE_BATTLEFIELD_STATUS"));
}

// ── QueueForLFG ───────────────────────────────────────────────────────────────

#[test]
fn queue_for_lfg_sets_queued_status_with_lfg_name() {
    let env = env();
    env.exec("QueueForLFG(789)").unwrap();
    let st = env.state().borrow();
    assert_eq!(st.battlefield_queue.status, BattlefieldStatus::Queued);
    assert!(st.battlefield_queue.name.contains("LFG Dungeon 789"));
}

// ── ToggleBattlefieldMinimap ──────────────────────────────────────────────────

#[test]
fn toggle_battlefield_minimap_flips_flag_twice() {
    let env = env();
    assert!(!env.state().borrow().battlefield_minimap_visible);
    env.exec("ToggleBattlefieldMinimap()").unwrap();
    assert!(env.state().borrow().battlefield_minimap_visible);
    env.exec("ToggleBattlefieldMinimap()").unwrap();
    assert!(!env.state().borrow().battlefield_minimap_visible);
}

// ── RequestBattlefieldPositions ───────────────────────────────────────────────

#[test]
fn request_battlefield_positions_fires_score_event() {
    let env = env();
    env.exec("RequestBattlefieldPositions()").unwrap();
    assert!(fired(&env, "UPDATE_BATTLEFIELD_SCORE"));
}

// ── BattlefieldStatus as_wow_str ──────────────────────────────────────────────

#[test]
fn battlefield_status_strings_match_retail_tokens() {
    assert_eq!(BattlefieldStatus::None.as_wow_str(), "none");
    assert_eq!(BattlefieldStatus::Queued.as_wow_str(), "queued");
    assert_eq!(BattlefieldStatus::Confirm.as_wow_str(), "confirm");
    assert_eq!(BattlefieldStatus::Active.as_wow_str(), "active");
}
