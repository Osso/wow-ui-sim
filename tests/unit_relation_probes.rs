//! Integration tests for the unit-relationship probes added to
//! `src/lua_api/globals/group_queries.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn set_party_active(env: &WowLuaEnv, active: bool) {
    env.state().borrow_mut().party_group_active = active;
}

// ── UnitInParty ───────────────────────────────────────────────────────────────

#[test]
fn unit_in_party_true_for_player_when_group_active() {
    let env = env();
    set_party_active(&env, true);
    let b: bool = env.eval(r#"return UnitInParty("player")"#).unwrap();
    assert!(b);
}

#[test]
fn unit_in_party_false_for_player_when_no_active_group() {
    let env = env();
    set_party_active(&env, false);
    let b: bool = env.eval(r#"return UnitInParty("player")"#).unwrap();
    assert!(!b);
}

#[test]
fn unit_in_party_true_for_party_token() {
    let env = env();
    set_party_active(&env, true);
    let b: bool = env.eval(r#"return UnitInParty("party1")"#).unwrap();
    assert!(b);
}

// ── UnitInRaid ────────────────────────────────────────────────────────────────

#[test]
fn unit_in_raid_false_below_six_members() {
    let env = env();
    set_party_active(&env, true);
    // Default party has <6 members.
    let b: bool = env.eval(r#"return UnitInRaid("player")"#).unwrap();
    assert!(!b);
}

#[test]
fn unit_in_raid_true_at_raid_size() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.party_group_active = true;
        let template = st.party_members[0].clone();
        while st.party_members.len() < 6 {
            st.party_members.push(template.clone());
        }
    }
    let b: bool = env.eval(r#"return UnitInRaid("player")"#).unwrap();
    assert!(b);
}

// ── UnitInOtherParty ──────────────────────────────────────────────────────────

#[test]
fn unit_in_other_party_always_false() {
    let env = env();
    let b: bool = env.eval(r#"return UnitInOtherParty("party1")"#).unwrap();
    assert!(!b);
}

// ── UnitInRange ───────────────────────────────────────────────────────────────

#[test]
fn unit_in_range_true_for_player() {
    let env = env();
    let b: bool = env.eval(r#"return UnitInRange("player")"#).unwrap();
    assert!(b);
}

#[test]
fn unit_in_range_false_for_empty_target() {
    let env = env();
    env.state().borrow_mut().current_target = None;
    let b: bool = env.eval(r#"return UnitInRange("target")"#).unwrap();
    assert!(!b);
}

#[test]
fn check_interact_distance_true_for_player() {
    let env = env();
    let b: bool = env.eval(r#"return CheckInteractDistance("player", 1)"#).unwrap();
    assert!(b);
}

#[test]
fn check_interact_distance_false_for_empty_target() {
    let env = env();
    env.state().borrow_mut().current_target = None;
    let b: bool = env.eval(r#"return CheckInteractDistance("target", 1)"#).unwrap();
    assert!(!b);
}

#[test]
fn check_interact_distance_false_for_invalid_index() {
    let env = env();
    let b: bool = env.eval(r#"return CheckInteractDistance("player", 5)"#).unwrap();
    assert!(!b);
}

// ── UnitInBattleground ────────────────────────────────────────────────────────

#[test]
fn unit_in_battleground_false_by_default() {
    let env = env();
    let b: bool = env.eval(r#"return UnitInBattleground("player")"#).unwrap();
    assert!(!b);
}

#[test]
fn unit_in_battleground_true_when_arena_flag_set() {
    let env = env();
    env.state().borrow_mut().world.battlefield_arena = true;
    let b: bool = env.eval(r#"return UnitInBattleground("player")"#).unwrap();
    assert!(b);
}

#[test]
fn unit_in_battleground_true_when_instance_type_is_pvp() {
    let env = env();
    env.state().borrow_mut().world.instance_type = "pvp".into();
    let b: bool = env.eval(r#"return UnitInBattleground("player")"#).unwrap();
    assert!(b);
}

// ── UnitCanCooperate ──────────────────────────────────────────────────────────

#[test]
fn unit_can_cooperate_true_between_player_and_party_member() {
    let env = env();
    set_party_active(&env, true);
    let b: bool = env
        .eval(r#"return UnitCanCooperate("player", "party1")"#)
        .unwrap();
    assert!(b);
}

// ── UnitIsGroupLeader ─────────────────────────────────────────────────────────

#[test]
fn unit_is_group_leader_true_for_player_when_leading_active_party() {
    let env = env();
    set_party_active(&env, true);
    let b: bool = env.eval(r#"return UnitIsGroupLeader("player")"#).unwrap();
    assert!(b);
}

#[test]
fn unit_is_group_leader_true_for_party_member_when_they_lead() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.party_group_active = true;
        st.party_leader_index = Some(0);
    }
    let b: bool = env.eval(r#"return UnitIsGroupLeader("party1")"#).unwrap();
    assert!(b);
}

#[test]
fn unit_is_group_leader_false_when_solo() {
    let env = env();
    set_party_active(&env, false);
    let b: bool = env.eval(r#"return UnitIsGroupLeader("player")"#).unwrap();
    assert!(!b);
}

// ── UnitIsGroupAssistant ──────────────────────────────────────────────────────

#[test]
fn unit_is_group_assistant_requires_everyone_assistant_flag() {
    let env = env();
    let b: bool = env
        .eval(r#"return UnitIsGroupAssistant("player")"#)
        .unwrap();
    assert!(!b);
    env.state().borrow_mut().everyone_assistant = true;
    #[cfg(feature = "retail-12-0-7")]
    {
        env.state().borrow_mut().party_group_active = true;
    }
    let b: bool = env
        .eval(r#"return UnitIsGroupAssistant("player")"#)
        .unwrap();
    assert!(b);
}
