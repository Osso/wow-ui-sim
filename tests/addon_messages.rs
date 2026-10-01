//! Outbound intent records, not network delivery. Docs: pinned Forever
//! ChatInfoDocumentation.lua:516/535, BattleNetDocumentation.lua:279,
//! ChatConstantsDocumentation.lua:144. Byte limits and state/channel rejection
//! mappings below are inferred simulator policy, not native-verified semantics.
#![cfg(feature = "client-wowforever")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_string;
use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PartyMember;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("messaging environment")
}

fn require_senders(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(C_ChatInfo.SendAddonMessage) == 'function', 'missing SendAddonMessage')
        assert(type(C_ChatInfo.SendAddonMessageLogged) == 'function', 'missing SendAddonMessageLogged')
        assert(type(C_BattleNet.SendGameData) == 'function', 'missing SendGameData')
        "#,
    )
    .expect("all documented messaging senders are published");
}

fn set_group(env: &WowLuaEnv, members: usize) {
    let mut sim = env.state().borrow_mut();
    sim.party_group_active = members > 0;
    sim.party_members = (0..members)
        .map(|index| PartyMember {
            name: format!("MessageMember{index}"),
            class_index: 2,
            level: 60,
            health: 100,
            health_max: 100,
            power: 100,
            power_max: 100,
            power_type: 0,
            power_type_name: "MANA".into(),
            is_leader: false,
            dead_since: None,
            buffs: vec![],
            debuffs: vec![],
        })
        .collect();
}

fn records(env: &WowLuaEnv) -> Vec<(String, String, String, String, String)> {
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

fn inject_secret(env: &WowLuaEnv, name: &str, payload: &str) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = wrap_host_secret_string(lua.state_mut(), payload);
    lua.state_mut().push(value);
    lua.set_global_val(name, value)
        .expect("root secret fixture");
    lua.state_mut().pop();
}

#[test]
fn documented_result_enum_values_are_published() {
    let env = env();
    env.exec(
        r#"
        local expected = {
            Success = 0, InvalidPrefix = 1, InvalidMessage = 2,
            AddonMessageThrottle = 3, InvalidChatType = 4, NotInGroup = 5,
            TargetRequired = 6, InvalidChannel = 7, ChannelThrottle = 8,
            GeneralError = 9, NotInGuild = 10, AddOnMessageLockdown = 11,
            TargetOffline = 12,
        }
        for name, value in pairs(expected) do
            assert(Enum.SendAddonMessageResult[name] == value, name)
        end
        "#,
    )
    .expect("documented result values");
}

#[test]
fn chat_senders_accept_ordered_records_without_inbound_echoes() {
    let env = env();
    require_senders(&env);
    set_group(&env, 1);
    let events_before = env.state().borrow().events.pending().len();
    env.exec(
        r#"
        assert(C_ChatInfo.SendAddonMessage('ACE', 'one') == 0)
        assert(C_ChatInfo.SendAddonMessageLogged('BUG', 'plain text', nil, nil) == 0)
        assert(C_ChatInfo.SendAddonMessage('ACE', 'three', 'WHISPER', 'Bob-Realm') == 0)
        "#,
    )
    .expect("accepted sends report Success");
    assert_eq!(
        records(&env),
        vec![
            (
                "addon".into(),
                "ACE".into(),
                "one".into(),
                "PARTY".into(),
                "".into()
            ),
            (
                "addon_logged".into(),
                "BUG".into(),
                "plain text".into(),
                "PARTY".into(),
                "".into()
            ),
            (
                "addon".into(),
                "ACE".into(),
                "three".into(),
                "WHISPER".into(),
                "Bob-Realm".into()
            ),
        ]
    );
    assert_eq!(env.state().borrow().events.pending().len(), events_before);
}

#[test]
fn chat_rejections_preserve_preexisting_records_and_events() {
    let env = env();
    require_senders(&env);
    env.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'kept', 'WHISPER', 'Bob') == 0)")
        .unwrap();
    let before = records(&env);
    let events_before = env.state().borrow().events.pending().len();
    env.exec(
        r#"
        for _, send in ipairs({C_ChatInfo.SendAddonMessage, C_ChatInfo.SendAddonMessageLogged}) do
            assert(send('', 'data', 'WHISPER', 'Bob') == 1)
            assert(send(string.rep('p', 17), 'data', 'WHISPER', 'Bob') == 1)
            assert(send('ACE', string.rep('x', 256), 'WHISPER', 'Bob') == 2)
            assert(send('ACE', 'data', 'SAY') == 4)
            assert(send('ACE', 'data', 'WHISPER') == 6)
            assert(send('ACE', 'data', 'WHISPER', '') == 6)
            -- Unsupported local routing has no acceptance record; inferred policy.
            assert(send('ACE', 'data', 'CHANNEL', '1') == 9)
            assert(send('ACE', 'data', 'INSTANCE_CHAT') == 9)
        end
        "#,
    )
    .expect("content and channel rejection codes");
    assert_eq!(records(&env), before);
    assert_eq!(env.state().borrow().events.pending().len(), events_before);
}

#[test]
fn chat_group_and_guild_acceptance_follows_existing_state() {
    let env = env();
    require_senders(&env);
    set_group(&env, 0);
    env.state().borrow_mut().world.guild_name = None;
    env.exec(
        r#"
        for _, send in ipairs({C_ChatInfo.SendAddonMessage, C_ChatInfo.SendAddonMessageLogged}) do
            assert(send('ACE', 'data') == 5)
            assert(send('ACE', 'data', 'RAID') == 5)
            assert(send('ACE', 'data', 'GUILD') == 10)
        end
        "#,
    )
    .unwrap();
    assert!(records(&env).is_empty());
    set_group(&env, 1);
    env.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'party') == 0); assert(C_ChatInfo.SendAddonMessage('ACE', 'raid', 'RAID') == 5)")
        .unwrap();
    set_group(&env, 6); // Existing IsInRaid model's threshold, not native raid semantics.
    env.state().borrow_mut().world.guild_name = Some("Messaging Guild".into());
    env.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'raid', 'RAID') == 0); assert(C_ChatInfo.SendAddonMessageLogged('ACE', 'guild', 'GUILD') == 0)")
        .unwrap();
    assert_eq!(
        records(&env)
            .iter()
            .map(|entry| entry.3.as_str())
            .collect::<Vec<_>>(),
        vec!["PARTY", "RAID", "GUILD"]
    );
    env.state().borrow_mut().party_group_active = false;
    env.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'stale roster') == 5)")
        .unwrap();
    assert_eq!(records(&env).len(), 3);
}

#[test]
fn consumer_byte_boundaries_accept_exact_limits_and_reject_overflow() {
    let env = env();
    require_senders(&env);
    env.exec(
        r#"
        for _, send in ipairs({C_ChatInfo.SendAddonMessage, C_ChatInfo.SendAddonMessageLogged}) do
            assert(send(string.rep('p', 16), string.rep('x', 255), 'WHISPER', 'Bob') == 0)
            assert(send(string.rep('é', 8), string.rep('é', 127) .. 'x', 'WHISPER', 'Bob') == 0)
            assert(send(string.rep('é', 9), 'data', 'WHISPER', 'Bob') == 1)
            assert(send('ACE', string.rep('é', 128), 'WHISPER', 'Bob') == 2)
        end
        "#,
    )
    .unwrap();
    let entries = records(&env);
    assert_eq!(entries.len(), 4);
    for entry in entries {
        assert_eq!(entry.1.len(), 16);
        assert_eq!(entry.2.len(), 255);
    }
}

#[test]
fn chat_typed_argument_errors_do_not_append() {
    let env = env();
    require_senders(&env);
    env.exec(
        r#"
        for _, send in ipairs({C_ChatInfo.SendAddonMessage, C_ChatInfo.SendAddonMessageLogged}) do
            assert(not pcall(send))
            assert(not pcall(send, nil, 'data', 'WHISPER', 'Bob'))
            assert(not pcall(send, 'ACE', nil, 'WHISPER', 'Bob'))
            for _, invalid in ipairs({17, false, {}, CreateFrame('Frame')}) do
                assert(not pcall(send, invalid, 'data', 'WHISPER', 'Bob'))
                assert(not pcall(send, 'ACE', invalid, 'WHISPER', 'Bob'))
                assert(not pcall(send, 'ACE', 'data', invalid, 'Bob'))
                assert(not pcall(send, 'ACE', 'data', 'WHISPER', invalid))
            end
        end
        "#,
    )
    .expect("strict required and optional string arguments");
    assert!(records(&env).is_empty());
}

#[test]
fn chat_not_allowed_secret_arguments_reject_even_untainted_callers() {
    let env = env();
    require_senders(&env);
    inject_secret(&env, "MessageSecret", "ACE");
    env.exec(
        r#"
        assert(issecure() and issecretvalue(MessageSecret))
        for _, send in ipairs({C_ChatInfo.SendAddonMessage, C_ChatInfo.SendAddonMessageLogged}) do
            assert(not pcall(send, MessageSecret, 'data', 'WHISPER', 'Bob'))
            assert(not pcall(send, 'ACE', MessageSecret, 'WHISPER', 'Bob'))
            assert(not pcall(send, 'ACE', 'data', MessageSecret, 'Bob'))
            assert(not pcall(send, 'ACE', 'data', 'WHISPER', MessageSecret))
        end
        "#,
    )
    .unwrap();
    assert!(records(&env).is_empty());
}

#[test]
fn bnet_send_records_online_game_account_not_friend_account() {
    let env = env();
    require_senders(&env);
    let events_before = env.state().borrow().events.pending().len();
    env.exec("assert(C_BattleNet.SendGameData(200001, 'ACE', 'bnet payload') == 0)")
        .unwrap();
    assert_eq!(
        records(&env),
        vec![(
            "bnet_game_data".into(),
            "ACE".into(),
            "bnet payload".into(),
            "".into(),
            "200001".into()
        )]
    );
    assert_eq!(env.state().borrow().events.pending().len(), events_before);
    env.exec("assert(C_BattleNet.SendGameData(100001, 'ACE', 'friend id is not game id') == 12)")
        .unwrap();
    assert_eq!(records(&env).len(), 1);
}

#[test]
fn bnet_rejections_and_online_transition_preserve_accepted_records() {
    let env = env();
    require_senders(&env);
    env.exec("assert(C_BattleNet.SendGameData(200001, 'ACE', 'kept') == 0)")
        .unwrap();
    let before = records(&env);
    let events_before = env.state().borrow().events.pending().len();
    env.exec(
        r#"
        assert(C_BattleNet.SendGameData(-1, 'ACE', 'data') == 9)
        assert(C_BattleNet.SendGameData(999999, 'ACE', 'data') == 12)
        assert(C_BattleNet.SendGameData(200002, 'ACE', 'data') == 12)
        assert(C_BattleNet.SendGameData(200001, '', 'data') == 1)
        assert(C_BattleNet.SendGameData(200001, string.rep('p', 17), 'data') == 1)
        assert(C_BattleNet.SendGameData(200001, 'ACE', string.rep('x', 4079)) == 2)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().bnet_friends[0].game_accounts[0].is_online = false;
    env.exec("assert(C_BattleNet.SendGameData(200001, 'ACE', 'now offline') == 12)")
        .unwrap();
    assert_eq!(records(&env), before);
    assert_eq!(env.state().borrow().events.pending().len(), events_before);
    env.state().borrow_mut().bnet_friends[0].game_accounts[1].is_online = true;
    env.exec(
        "assert(C_BattleNet.SendGameData(200002, string.rep('p', 16), string.rep('x', 4078)) == 0)",
    )
    .unwrap();
    let entries = records(&env);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].4, "200002");
    assert_eq!(entries[1].1.len(), 16);
    assert_eq!(entries[1].2.len(), 4078);
}

#[test]
fn empty_payloads_are_accepted_and_bnet_limit_counts_bytes() {
    let env = env();
    require_senders(&env);
    env.exec(
        r#"
        assert(C_ChatInfo.SendAddonMessage('ACE', '', 'WHISPER', 'Bob') == 0)
        assert(C_ChatInfo.SendAddonMessageLogged('ACE', '', 'WHISPER', 'Bob') == 0)
        assert(C_BattleNet.SendGameData(200001, 'ACE', '') == 0)
        assert(C_BattleNet.SendGameData(200001, 'ACE', string.rep('é', 2039)) == 0)
        assert(C_BattleNet.SendGameData(200001, 'ACE', string.rep('é', 2040)) == 2)
        "#,
    )
    .unwrap();
    let entries = records(&env);
    assert_eq!(entries.len(), 4);
    assert!(entries[..3].iter().all(|entry| entry.2.is_empty()));
    assert_eq!(entries[3].2.len(), 4078);
}

#[test]
fn bnet_typed_argument_errors_do_not_append() {
    let env = env();
    require_senders(&env);
    env.exec(
        r#"
        local send = C_BattleNet.SendGameData
        assert(not pcall(send))
        assert(not pcall(send, nil, 'ACE', 'data'))
        for _, invalid in ipairs({'200001', false, {}, CreateFrame('Frame')}) do
            assert(not pcall(send, invalid, 'ACE', 'data'))
        end
        for _, invalid in ipairs({17, false, {}, CreateFrame('Frame')}) do
            assert(not pcall(send, 200001, invalid, 'data'))
            assert(not pcall(send, 200001, 'ACE', invalid))
        end
        assert(not pcall(send, 200001, nil, 'data'))
        assert(not pcall(send, 200001, 'ACE', nil))
        "#,
    )
    .unwrap();
    assert!(records(&env).is_empty());
}

#[test]
fn bnet_allowed_when_untainted_secret_strings_accept_without_clearing_taint() {
    let env = env();
    require_senders(&env);
    inject_secret(&env, "SecretPrefix", "ACE");
    inject_secret(&env, "SecretData", "secret payload");
    env.exec(
        r#"
        assert(issecure() and issecretvalue(SecretPrefix) and issecretvalue(SecretData))
        assert(C_BattleNet.SendGameData(200001, SecretPrefix, SecretData) == 0)
        local function addon()
            assert(not issecure())
            assert(not pcall(C_BattleNet.SendGameData, 200001, SecretPrefix, 'plain'))
            assert(not pcall(C_BattleNet.SendGameData, 200001, 'ACE', SecretData))
            assert(not issecure(), 'sender must not clear caller taint')
            assert(C_BattleNet.SendGameData(200001, 'ACE', 'ordinary addon payload') == 0)
        end
        debug.setobjecttaint(addon, 'MessagingProbe')
        addon()
        "#,
    )
    .unwrap();
    let entries = records(&env);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].1, "ACE");
    assert_eq!(entries[0].2, "secret payload");
    assert_eq!(entries[1].2, "ordinary addon payload");
}

#[test]
fn bnet_secret_numeric_target_requires_untainted_access() {
    let env = env();
    require_senders(&env);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let value = rilua::table_security::wrap_secret(lua.state_mut(), rilua::Val::Num(200001.0))
            .expect("host secret numeric account id");
        lua.state_mut().push(value);
        lua.set_global_val("SecretGameAccount", value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(
        r#"
        assert(issecure() and issecretvalue(SecretGameAccount))
        assert(C_BattleNet.SendGameData(SecretGameAccount, 'ACE', 'secure target') == 0)
        local function addon()
            assert(not issecure())
            assert(not pcall(C_BattleNet.SendGameData, SecretGameAccount, 'ACE', 'blocked'))
            assert(not issecure())
        end
        debug.setobjecttaint(addon, 'MessagingProbe')
        addon()
        "#,
    )
    .unwrap();
    let entries = records(&env);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].4, "200001");
    assert_eq!(entries[0].2, "secure target");
}

#[test]
fn outbound_records_and_recipient_state_are_environment_local() {
    let first = env();
    let second = env();
    require_senders(&first);
    require_senders(&second);
    set_group(&first, 1);
    set_group(&second, 0);
    second.state().borrow_mut().bnet_friends.clear();
    first.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'first') == 0); assert(C_BattleNet.SendGameData(200001, 'ACE', 'first bnet') == 0)")
        .unwrap();
    second.exec("assert(C_ChatInfo.SendAddonMessage('ACE', 'second') == 5); assert(C_BattleNet.SendGameData(200001, 'ACE', 'second bnet') == 12)")
        .unwrap();
    assert_eq!(records(&first).len(), 2);
    assert!(records(&second).is_empty());
    second.exec("assert(C_ChatInfo.SendAddonMessageLogged('ACE', 'second whisper', 'WHISPER', 'Bob') == 0)")
        .unwrap();
    assert_eq!(records(&first).len(), 2);
    assert_eq!(records(&second).len(), 1);
    assert_eq!(records(&second)[0].2, "second whisper");
}

#[test]
fn legacy_global_keeps_nil_return_permissive_log_and_four_event_arguments() {
    let env = env();
    let count: i32 = env
        .eval("return select('#', SendAddonMessage('LEGACY', 'payload', 'WHISPER', 'Bob'))")
        .unwrap();
    assert_eq!(count, 0);
    let sim = env.state().borrow();
    let event = sim
        .events
        .pending()
        .iter()
        .find(|event| event.name == "CHAT_MSG_ADDON")
        .unwrap();
    assert_eq!(event.args.len(), 4);
    for (argument, expected) in event
        .args
        .iter()
        .zip(["LEGACY", "payload", "WHISPER", "Bob"])
    {
        assert!(matches!(argument, EventArg::String(value) if value == expected));
    }
    drop(sim);
    env.exec("assert(SendAddonMessage(nil, nil, nil, nil) == nil)")
        .unwrap();
    assert_eq!(
        records(&env)[1],
        ("addon".into(), "".into(), "".into(), "".into(), "".into())
    );
}
