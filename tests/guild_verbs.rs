//! Integration tests for `src/lua_api/globals/guild_verbs.rs`.

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

// ── GuildInvite ───────────────────────────────────────────────────────────────

#[test]
fn guild_invite_appends_member_at_lowest_rank_and_fires_roster() {
    let env = env();
    env.exec(r#"GuildInvite("Recruit1")"#).unwrap();
    let st = env.state().borrow();
    let member = st
        .world
        .guild_members
        .last()
        .expect("new member must exist");
    assert_eq!(member.name, "Recruit1");
    assert!(
        member.rank_index >= 1,
        "rank index must be at least 1 (Guild Master)"
    );
    drop(st);
    assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
}

#[test]
fn guild_invite_empty_name_is_noop() {
    let env = env();
    let before = env.state().borrow().world.guild_members.len();
    env.exec(r#"GuildInvite("")"#).unwrap();
    assert_eq!(env.state().borrow().world.guild_members.len(), before);
}

// ── GuildUninvite / GuildKick ─────────────────────────────────────────────────

#[test]
fn guild_uninvite_removes_by_name_and_fires_roster() {
    let env = env();
    env.exec(
        r#"GuildInvite("Bob")
               GuildInvite("Alice")"#,
    )
    .unwrap();
    env.exec(r#"GuildUninvite("Bob")"#).unwrap();
    let st = env.state().borrow();
    assert!(st.world.guild_members.iter().all(|m| m.name != "Bob"));
    assert!(st.world.guild_members.iter().any(|m| m.name == "Alice"));
    drop(st);
    assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
}

#[test]
fn guild_kick_aliases_guild_uninvite() {
    let env = env();
    env.exec(r#"GuildInvite("Carol")"#).unwrap();
    env.exec(r#"GuildKick("Carol")"#).unwrap();
    let st = env.state().borrow();
    assert!(st.world.guild_members.iter().all(|m| m.name != "Carol"));
}

#[test]
fn guild_uninvite_unknown_is_noop() {
    let env = env();
    let before = env.state().borrow().world.guild_members.len();
    env.exec(r#"GuildUninvite("Nobody")"#).unwrap();
    assert_eq!(env.state().borrow().world.guild_members.len(), before);
}

// ── GuildLeave ────────────────────────────────────────────────────────────────

#[test]
fn guild_leave_clears_identity_and_roster() {
    let env = env();
    env.exec(
        r#"GuildInvite("Someone")
               GuildSetMOTD("Farewell")"#,
    )
    .unwrap();
    env.exec("GuildLeave()").unwrap();
    let st = env.state().borrow();
    assert!(st.world.guild_name.is_none());
    assert!(st.world.guild_rank.is_none());
    assert_eq!(st.world.guild_num_members, 0);
    assert!(st.world.guild_members.is_empty());
    assert!(st.world.guild_motd.is_empty());
    drop(st);
    assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
}

// ── GuildPromote ──────────────────────────────────────────────────────────────

#[test]
fn guild_promote_decrements_rank_index() {
    let env = env();
    env.exec(r#"GuildInvite("Alice")"#).unwrap();
    let initial_rank = env
        .state()
        .borrow()
        .world
        .guild_members
        .last()
        .map(|m| m.rank_index)
        .unwrap();
    env.exec(r#"GuildPromote("Alice")"#).unwrap();
    let new_rank = env
        .state()
        .borrow()
        .world
        .guild_members
        .iter()
        .find(|m| m.name == "Alice")
        .map(|m| m.rank_index)
        .unwrap();
    if initial_rank > 1 {
        assert_eq!(new_rank, initial_rank - 1);
        assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
    } else {
        assert_eq!(new_rank, 1, "cannot promote above rank 1");
    }
}

#[test]
fn guild_promote_unknown_is_noop() {
    let env = env();
    env.exec(r#"GuildPromote("Nobody")"#).unwrap();
    // Must not panic; no fired assertion because no event should queue.
}

// ── GuildSetMOTD ──────────────────────────────────────────────────────────────

#[test]
fn guild_set_motd_writes_world_field_and_fires_motd() {
    let env = env();
    env.exec(r#"GuildSetMOTD("Weekly raid Tuesday!")"#).unwrap();
    let st = env.state().borrow();
    assert_eq!(st.world.guild_motd, "Weekly raid Tuesday!");
    drop(st);
    assert!(fired(&env, "GUILD_MOTD"));
}

#[test]
fn guild_set_motd_empty_clears_motd() {
    let env = env();
    env.exec(
        r#"GuildSetMOTD("Old MOTD")
               GuildSetMOTD("")"#,
    )
    .unwrap();
    assert!(env.state().borrow().world.guild_motd.is_empty());
}

// ── RequestGuildRoster / RequestGuildChallengeInfo ────────────────────────────

#[test]
fn request_guild_roster_fires_event() {
    let env = env();
    env.exec("RequestGuildRoster()").unwrap();
    assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
}

#[test]
fn c_guild_info_guild_roster_fires_event() {
    let env = env();
    env.exec("A_Admin.SetManualClock(1); C_GuildInfo.GuildRoster()")
        .unwrap();
    assert!(fired(&env, "GUILD_ROSTER_UPDATE"));
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn c_guild_info_guild_roster_drops_repeated_requests_until_ten_seconds() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        C_GuildInfo.GuildRoster()
        C_GuildInfo.GuildRoster()
        A_Admin.AdvanceTime(9)
        C_GuildInfo.GuildRoster()
        A_Admin.AdvanceTime(1)
        C_GuildInfo.GuildRoster()
        "#,
    )
    .unwrap();
    let roster_events = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .filter(|event| event.name == "GUILD_ROSTER_UPDATE")
        .count();
    assert_eq!(roster_events, 2);
}

#[test]
fn request_guild_challenge_info_dispatches_challenge_updated() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            local got = false
            f:RegisterEvent("GUILD_CHALLENGE_UPDATED")
            f:SetScript("OnEvent", function(self, event)
                if event == "GUILD_CHALLENGE_UPDATED" then got = true end
            end)
            RequestGuildChallengeInfo()
            return got
            "#,
        )
        .unwrap();
    assert!(received);
}
