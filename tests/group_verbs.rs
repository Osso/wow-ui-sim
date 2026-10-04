//! Integration tests for `src/lua_api/globals/group_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

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

// ── InviteToGroup ─────────────────────────────────────────────────────────────

#[test]
fn invite_to_group_appends_member_and_flags_active() {
    let env = env();
    let before = env.state().borrow().party_members.len();
    env.exec(r#"InviteToGroup("Arthas")"#).unwrap();
    let st = env.state().borrow();
    assert_eq!(st.party_members.len(), before + 1);
    assert_eq!(
        st.party_members.last().map(|m| m.name.clone()),
        Some("Arthas".to_string())
    );
    assert!(st.party_group_active);
    drop(st);
    assert!(fired(&env, "GROUP_ROSTER_UPDATE"));
}

#[test]
fn invite_to_group_empty_name_is_noop() {
    let env = env();
    let before = env.state().borrow().party_members.len();
    env.exec(r#"InviteToGroup("")"#).unwrap();
    assert_eq!(env.state().borrow().party_members.len(), before);
}

// ── AcceptGroup / DeclineGroup ────────────────────────────────────────────────

#[test]
fn accept_group_flags_active_and_fires_roster() {
    let env = env();
    env.state().borrow_mut().party_group_active = false;
    env.exec("AcceptGroup()").unwrap();
    assert!(env.state().borrow().party_group_active);
    assert!(fired(&env, "GROUP_ROSTER_UPDATE"));
}

#[test]
fn decline_group_is_silent_noop() {
    let env = env();
    let before_count = env.state().borrow().events.pending().len();
    env.exec("DeclineGroup()").unwrap();
    let after_count = env.state().borrow().events.pending().len();
    assert_eq!(
        after_count, before_count,
        "DeclineGroup must not queue events"
    );
}

// ── LeaveParty ────────────────────────────────────────────────────────────────

#[test]
fn leave_party_clears_roster_and_fires_event() {
    let env = env();
    // Default party has members — confirm they disappear.
    let had_members = !env.state().borrow().party_members.is_empty();
    {
        let mut st = env.state().borrow_mut();
        st.party_leader_index = Some(0);
        st.is_party_lfg = true;
        st.everyone_assistant = true;
    }
    env.exec("LeaveParty()").unwrap();
    let st = env.state().borrow();
    assert!(st.party_members.is_empty());
    assert!(!st.party_group_active);
    assert_eq!(st.party_leader_index, None);
    assert!(!st.is_party_lfg);
    assert!(!st.everyone_assistant);
    drop(st);
    if had_members {
        assert!(fired(&env, "GROUP_ROSTER_UPDATE"));
    }
}

// ── RemoveFromParty ───────────────────────────────────────────────────────────

#[test]
fn remove_from_party_by_name_succeeds() {
    let env = env();
    let first_name = env
        .state()
        .borrow()
        .party_members
        .first()
        .map(|m| m.name.clone())
        .expect("default party should exist");
    let before = env.state().borrow().party_members.len();
    env.exec(&format!(r#"RemoveFromParty("{first_name}")"#))
        .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.party_members.len(), before - 1);
    assert!(st.party_members.iter().all(|m| m.name != first_name));
    drop(st);
    assert!(fired(&env, "GROUP_ROSTER_UPDATE"));
}

#[test]
fn remove_from_party_unknown_name_is_noop() {
    let env = env();
    let before = env.state().borrow().party_members.len();
    env.exec(r#"RemoveFromParty("NobodyHere")"#).unwrap();
    assert_eq!(env.state().borrow().party_members.len(), before);
}

// ── UninviteUnit / KickUnit ───────────────────────────────────────────────────

#[test]
fn uninvite_unit_by_party_token_removes_indexed_member() {
    let env = env();
    let first_name = env.state().borrow().party_members[0].name.clone();
    #[cfg(not(feature = "retail-12-0-7"))]
    env.exec(r#"UninviteUnit("party1")"#).unwrap();
    #[cfg(feature = "retail-12-0-7")]
    env.exec(r#"assert(rawget(_G, "UninviteUnit") == nil); C_PartyInfo.UninviteUnit("party1")"#)
        .unwrap();
    assert!(
        env.state()
            .borrow()
            .party_members
            .iter()
            .all(|m| m.name != first_name)
    );
}

#[test]
fn uninvite_unit_by_name_removes_matching_member() {
    let env = env();
    let second_name = env.state().borrow().party_members[1].name.clone();
    #[cfg(not(feature = "retail-12-0-7"))]
    env.exec(&format!(r#"UninviteUnit("{second_name}")"#))
        .unwrap();
    #[cfg(feature = "retail-12-0-7")]
    env.exec(&format!(r#"assert(rawget(_G, "UninviteUnit") == nil); C_PartyInfo.UninviteUnit("{second_name}")"#))
        .unwrap();
    assert!(
        env.state()
            .borrow()
            .party_members
            .iter()
            .all(|m| m.name != second_name)
    );
}

#[test]
fn kick_unit_aliases_uninvite_unit() {
    let env = env();
    let name = env.state().borrow().party_members[0].name.clone();
    env.exec(r#"KickUnit("party1")"#).unwrap();
    assert!(
        env.state()
            .borrow()
            .party_members
            .iter()
            .all(|m| m.name != name)
    );
}

// ── ReadyCheck ────────────────────────────────────────────────────────────────

#[test]
fn ready_check_fires_event() {
    let env = env();
    let delivered: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local event
            frame:RegisterEvent("READY_CHECK")
            frame:SetScript("OnEvent", function(_, firedEvent)
                event = firedEvent
            end)
            ReadyCheck()
            return event == "READY_CHECK"
            "#,
        )
        .unwrap();

    assert!(delivered, "ReadyCheck should synchronously deliver READY_CHECK");
}
