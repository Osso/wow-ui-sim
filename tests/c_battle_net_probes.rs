//! Tests for `C_BattleNet` probes backed by `SimState.bnet_friends`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BnetFriend, BnetGameAccount};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_num_friends_returns_seeded_count() {
    let env = env();
    let count: i32 = env.eval("return C_BattleNet.GetNumFriends()").unwrap();
    assert_eq!(count, 2, "two seeded friends");
}

#[test]
fn get_friend_account_info_returns_table_for_valid_index() {
    let env = env();
    let (battle_tag, account_name, is_friend, is_battle_tag): (String, String, bool, bool) = env
        .eval(
            r#"
            local info = C_BattleNet.GetFriendAccountInfo(1)
            return info.battleTag, info.accountName, info.isFriend, info.isBattleTagFriend
            "#,
        )
        .unwrap();
    assert_eq!(battle_tag, "Uther#1000");
    assert_eq!(account_name, "Uther");
    assert!(is_friend);
    assert!(is_battle_tag);
}

#[test]
fn get_friend_account_info_returns_nested_game_account() {
    let env = env();
    let (character_name, realm_name, class_id, level, faction): (String, String, i32, i32, String) =
        env.eval(
            r#"
            local info = C_BattleNet.GetFriendAccountInfo(1)
            local ga = info.gameAccountInfo
            return ga.characterName, ga.realmName, ga.classID, ga.characterLevel, ga.factionName
            "#,
        )
        .unwrap();
    assert_eq!(character_name, "Uther");
    assert_eq!(realm_name, "Stormwind");
    assert_eq!(class_id, 2, "Paladin");
    assert_eq!(level, 70);
    assert_eq!(faction, "Alliance");
}

#[test]
fn get_friend_account_info_returns_nil_for_index_zero() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_BattleNet.GetFriendAccountInfo(0) == nil")
        .unwrap();
    assert!(is_nil, "index 0 is out of range");
}

#[test]
fn get_friend_account_info_returns_nil_for_out_of_range_index() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_BattleNet.GetFriendAccountInfo(999) == nil")
        .unwrap();
    assert!(is_nil, "index beyond list length returns nil");
}

#[test]
fn get_account_info_by_guid_found() {
    let env = env();
    let (battle_tag, bnet_id): (String, i32) = env
        .eval(
            r#"
            local info = C_BattleNet.GetAccountInfoByGUID("BNet-0-100001")
            return info.battleTag, info.bnetAccountID
            "#,
        )
        .unwrap();
    assert_eq!(battle_tag, "Uther#1000");
    assert_eq!(bnet_id, 100001);
}

#[test]
fn get_account_info_by_guid_not_found_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return C_BattleNet.GetAccountInfoByGUID("BNet-0-UNKNOWN") == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_game_account_info_by_guid_found() {
    let env = env();
    let (character_name, is_online, client): (String, bool, String) = env
        .eval(
            r#"
            local ga = C_BattleNet.GetGameAccountInfoByGUID("Player-1-00000001")
            return ga.characterName, ga.isOnline, ga.clientProgram
            "#,
        )
        .unwrap();
    assert_eq!(character_name, "Uther");
    assert!(is_online);
    assert_eq!(client, "WoW");
}

#[test]
fn get_game_account_info_by_guid_not_found_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(r#"return C_BattleNet.GetGameAccountInfoByGUID("Player-0-DEADBEEF") == nil"#)
        .unwrap();
    assert!(is_nil);
}

#[test]
fn get_friend_num_accounts_returns_correct_count() {
    let env = env();
    let (first, second, out_of_range): (i32, i32, i32) = env
        .eval(
            r#"
            return C_BattleNet.GetFriendNumAccounts(1),
                   C_BattleNet.GetFriendNumAccounts(2),
                   C_BattleNet.GetFriendNumAccounts(99)
            "#,
        )
        .unwrap();
    assert_eq!(first, 2, "friend 1 has two game accounts");
    assert_eq!(second, 0, "friend 2 is offline with no game accounts");
    assert_eq!(out_of_range, 0, "out-of-range returns 0");
}

#[test]
fn bnet_friends_reflect_sim_state_mutation() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.bnet_friends.clear();
        state.bnet_friends.push(BnetFriend {
            friend_index: 1,
            bnet_account_guid: "BNet-0-999".into(),
            bnet_account_id: 999,
            battle_tag: "Custom#9999".into(),
            account_name: "Custom".into(),
            note: String::new(),
            custom_title_friend_name: None,
            friend_tags: Vec::new(),
            custom_message: String::new(),
            custom_message_time: 0,
            appear_offline: false,
            is_battle_tag_friend: true,
            friend_level: 1,
            is_friend: true,
            is_favorite: false,
            is_afk: false,
            is_dnd: false,
            last_online_time: 0,
            raf_link_type: 0,
            game_accounts: vec![BnetGameAccount {
                wow_account_guid: "Player-9-00000099".into(),
                game_account_id: 9001,
                character_name: "CustomChar".into(),
                realm_name: "Test".into(),
                realm_display_name: "Test".into(),
                realm_id: 9,
                class_id: 5,
                class_name: "Priest".into(),
                character_level: 60,
                area_name: "Test Zone".into(),
                is_online: true,
                is_game_afk: false,
                is_game_busy: false,
                client_program: "WoW".into(),
                faction_name: "Alliance".into(),
                race_name: "Night Elf".into(),
                rich_presence: String::new(),
                can_summon: false,
                is_in_current_region: true,
                has_focus: false,
                wow_project_id: 1,
                timerunning_season_id: 0,
                region_id: 1,
                player_guid: String::new(),
            }],
        });
    }
    let (count, tag, char_name, num_accounts): (i32, String, String, i32) = env
        .eval(
            r#"
            local info = C_BattleNet.GetFriendAccountInfo(1)
            return C_BattleNet.GetNumFriends(),
                   info.battleTag,
                   info.gameAccountInfo.characterName,
                   C_BattleNet.GetFriendNumAccounts(1)
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(tag, "Custom#9999");
    assert_eq!(char_name, "CustomChar");
    assert_eq!(num_accounts, 1);
}
