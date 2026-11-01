//! Integration tests for `src/lua_api/globals/channel_verbs.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn channel_count(env: &WowLuaEnv) -> usize {
    env.state().borrow().chat_channels.len()
}

// ── JoinChannelByName ─────────────────────────────────────────────────────────

#[test]
fn join_channel_by_name_adds_channel_once() {
    let env = env();
    env.exec(r#"JoinChannelByName("Trade")"#).unwrap();
    assert_eq!(channel_count(&env), 1);
    // Idempotent — same name does not duplicate.
    env.exec(r#"JoinChannelByName("Trade")"#).unwrap();
    assert_eq!(channel_count(&env), 1);
}

#[test]
fn join_channel_empty_name_is_noop() {
    let env = env();
    env.exec(r#"JoinChannelByName("")"#).unwrap();
    assert_eq!(channel_count(&env), 0);
}

#[test]
fn join_temporary_channel_aliases_join() {
    let env = env();
    env.exec(r#"JoinTemporaryChannel("LookingForGroup")"#)
        .unwrap();
    assert_eq!(channel_count(&env), 1);
    assert_eq!(
        env.state().borrow().chat_channels[0].name,
        "LookingForGroup"
    );
}

#[test]
fn get_channel_list_returns_no_values_when_empty() {
    let env = env();
    let returned_values: i32 = env
        .eval(r##"return select("#", GetChannelList())"##)
        .unwrap();
    assert_eq!(returned_values, 0);
}

#[test]
fn get_channel_list_returns_channel_triples() {
    let env = env();
    let (count, first_id, first_name, first_disabled, second_id, second_name, second_disabled): (
        i32,
        i32,
        String,
        bool,
        i32,
        String,
        bool,
    ) = env
        .eval(
            r##"
            JoinChannelByName("General")
            JoinChannelByName("Trade")
            return select("#", GetChannelList()), GetChannelList()
            "##,
        )
        .unwrap();
    assert_eq!(count, 6);
    assert_eq!(
        (first_id, first_name, first_disabled),
        (1, "General".to_string(), false)
    );
    assert_eq!(
        (second_id, second_name, second_disabled),
        (2, "Trade".to_string(), false)
    );
}

#[test]
fn chat_info_channel_identifier_matches_joined_channel() {
    let env = env();
    let (name, shortcut, local_id, instance_id, zone_channel_id, channel_type, enabled): (
        String,
        String,
        i32,
        i32,
        i32,
        i32,
        String,
    ) = env
        .eval(
            r#"
            JoinChannelByName("General")
            local info = C_ChatInfo.GetChannelInfoFromIdentifier("General")
            local before = C_TTSSettings.GetChannelEnabled(info)
            C_TTSSettings.SetChannelEnabled(info, true)
            local after = C_TTSSettings.GetChannelEnabled(C_ChatInfo.GetChannelInfoFromIdentifier("General"))
            return info.name, info.shortcut, info.localID, info.instanceID,
                info.zoneChannelID, info.channelType, tostring(before) .. "," .. tostring(after)
            "#,
        )
        .unwrap();
    assert_eq!(name, "General");
    assert_eq!(shortcut, "1");
    assert_eq!(local_id, 1);
    assert_eq!(instance_id, 0);
    assert_eq!(zone_channel_id, 1);
    assert_eq!(channel_type, 1);
    // Legacy TTS defaults have no enabled channels; the returned record round-trips
    // through C_TTSSettings (docs/specs/c-factory-cf2-tts.md).
    assert_eq!(enabled, "false,true");
}

// ── ChannelLeave ──────────────────────────────────────────────────────────────

#[test]
fn channel_leave_removes_channel() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Trade")
               JoinChannelByName("General")
               ChannelLeave("Trade")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.chat_channels.len(), 1);
    assert_eq!(st.chat_channels[0].name, "General");
}

// ── ChannelInvite / ChannelKick ───────────────────────────────────────────────

#[test]
fn channel_invite_adds_member_idempotently() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Guild")
               ChannelInvite("Guild", "Alice")
               ChannelInvite("Guild", "Alice")
               ChannelInvite("Guild", "Bob")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    let members = &st.chat_channels[0].members;
    assert_eq!(members.len(), 2);
    assert!(members.contains("Alice"));
    assert!(members.contains("Bob"));
}

#[test]
fn channel_kick_drops_member_and_moderator_entry() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Guild")
               ChannelInvite("Guild", "Alice")
               ChannelModerator("Guild", "Alice")
               ChannelKick("Guild", "Alice")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    let ch = &st.chat_channels[0];
    assert!(!ch.members.contains("Alice"));
    assert!(!ch.moderators.contains("Alice"));
}

// ── ChannelBan ────────────────────────────────────────────────────────────────

#[test]
fn channel_ban_evicts_and_records_ban() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Trade")
               ChannelInvite("Trade", "Troll")
               ChannelBan("Trade", "Troll")"#,
    )
    .unwrap();
    let st = env.state().borrow();
    let ch = &st.chat_channels[0];
    assert!(!ch.members.contains("Troll"));
    assert!(ch.banned.contains("Troll"));
}

#[test]
fn banned_player_cannot_rejoin_via_invite() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Trade")
               ChannelBan("Trade", "Troll")
               ChannelInvite("Trade", "Troll")"#,
    )
    .unwrap();
    let ch = &env.state().borrow().chat_channels[0].clone();
    assert!(!ch.members.contains("Troll"));
}

// ── ChannelModerator / ChannelUnmoderator ─────────────────────────────────────

#[test]
fn channel_moderator_requires_membership() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Guild")
               ChannelModerator("Guild", "NotAMember")"#,
    )
    .unwrap();
    let ch = env.state().borrow().chat_channels[0].clone();
    assert!(
        !ch.moderators.contains("NotAMember"),
        "moderator grant must require prior membership"
    );
}

#[test]
fn channel_moderator_and_unmoderator_round_trip() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Guild")
               ChannelInvite("Guild", "Alice")
               ChannelModerator("Guild", "Alice")"#,
    )
    .unwrap();
    assert!(
        env.state().borrow().chat_channels[0]
            .moderators
            .contains("Alice")
    );
    env.exec(r#"ChannelUnmoderator("Guild", "Alice")"#).unwrap();
    assert!(
        !env.state().borrow().chat_channels[0]
            .moderators
            .contains("Alice")
    );
}

// ── SwapChatChannelLinks ──────────────────────────────────────────────────────

#[test]
fn swap_chat_channel_links_swaps_positions() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Trade")
               JoinChannelByName("General")
               JoinChannelByName("LocalDefense")
               SwapChatChannelLinks(1, 3)"#,
    )
    .unwrap();
    let st = env.state().borrow();
    assert_eq!(st.chat_channels[0].name, "LocalDefense");
    assert_eq!(st.chat_channels[1].name, "General");
    assert_eq!(st.chat_channels[2].name, "Trade");
}

#[test]
fn swap_chat_channel_links_out_of_range_is_noop() {
    let env = env();
    env.exec(
        r#"JoinChannelByName("Trade")
               SwapChatChannelLinks(1, 99)"#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().chat_channels[0].name, "Trade");
}
