#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

type ChatRecord = (String, String, String, String, String);

fn read_chat_records(env: &WowLuaEnv) -> Vec<ChatRecord> {
    env.state()
        .borrow()
        .message_log
        .iter()
        .map(|entry| {
            (
                entry.kind.clone(),
                entry.prefix.clone(),
                entry.message.clone(),
                entry.channel.clone(),
                entry.target.clone(),
            )
        })
        .collect()
}

#[test]
fn forever_chat_senders_append_ordered_environment_local_records() {
    let env = WowLuaEnv::new().expect("Forever chat environment");
    env.state().borrow_mut().party_group_active = true;
    env.exec(
        r#"
        assert(type(C_ChatInfo.SendAddonMessage) == 'function')
        assert(type(C_ChatInfo.SendAddonMessageLogged) == 'function')
        assert(C_ChatInfo.SendAddonMessage('ACE', 'one') == 0)
        assert(C_ChatInfo.SendAddonMessageLogged('BUG', 'plain text', nil, nil) == 0)
        assert(C_ChatInfo.SendAddonMessage('ACE', 'three', 'WHISPER', 'Bob-Realm') == 0)
        "#,
    )
    .expect("successful Forever sends");

    let expected = [
        ("addon", "ACE", "one", "PARTY", ""),
        ("addon_logged", "BUG", "plain text", "PARTY", ""),
        ("addon", "ACE", "three", "WHISPER", "Bob-Realm"),
    ]
    .map(|(kind, prefix, message, channel, target)| {
        (
            kind.into(),
            prefix.into(),
            message.into(),
            channel.into(),
            target.into(),
        )
    });
    assert_eq!(read_chat_records(&env), expected);

    let other_env = WowLuaEnv::new().expect("independent Forever chat environment");
    assert!(read_chat_records(&other_env).is_empty());
}

#[test]
fn forever_invalid_chat_prefix_preserves_existing_records() {
    let env = WowLuaEnv::new().expect("Forever chat validation environment");
    env.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'kept', 'WHISPER', 'Bob') == 0)")
        .expect("accepted control message");
    let before = read_chat_records(&env);
    assert_eq!(before.len(), 1);

    env.exec(
        r#"
        assert(C_ChatInfo.SendAddonMessage('', 'rejected', 'WHISPER', 'Bob') == 1)
        assert(C_ChatInfo.SendAddonMessageLogged('', 'rejected', 'WHISPER', 'Bob') == 1)
        "#,
    )
    .expect("empty prefixes return InvalidPrefix");
    assert_eq!(read_chat_records(&env), before);
}

#[test]
fn forever_removed_public_combat_log_getters_are_absent() {
    let env = WowLuaEnv::new().expect("Forever combat-log publication environment");
    env.exec(
        r#"
        assert(rawget(C_CombatLog, 'GetCurrentEventInfo') == nil)
        assert(C_CombatLog.GetCurrentEventInfo == nil)
        assert(rawget(_G, 'CombatLogGetCurrentEventInfo') == nil)
        "#,
    )
    .expect("explicitly removed public combat-log getters");
}
