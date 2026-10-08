//! Current successor model only; not the historical BN* tuple or numeric-ID contract.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_account_and_game_identity_state(env: &WowLuaEnv) {
    {
        let mut state = env.state().borrow_mut();
        let mut friend = state.bnet_friends[0].clone();
        friend.friend_index = 1;
        friend.bnet_account_id = 91017;
        friend.bnet_account_guid = "BNet-0-91017".into();
        friend.game_accounts[0].game_account_id = 62041;
        friend.game_accounts[0].wow_account_guid = "Player-7-62041".into();
        friend.game_accounts[0].character_name = "Primary".into();
        friend.game_accounts[1].game_account_id = 62042;
        friend.game_accounts[1].wow_account_guid = "Player-7-62042".into();
        friend.game_accounts[1].character_name = "Secondary".into();
        state.bnet_friends = vec![friend];
    }
    let result: bool = env
        .eval(
            r#"
            local parent = C_BattleNet.GetAccountInfoByGUID('BNet-0-91017')
            local first = C_BattleNet.GetFriendAccountInfo(1)
            local second = C_BattleNet.GetGameAccountInfoByGUID('Player-7-62042')
            assert(parent.bnetAccountID == 91017)
            assert(first.bnetAccountID == 91017)
            assert(first.gameAccountInfo.gameAccountID == 62041)
            assert(first.gameAccountInfo.characterName == 'Primary')
            assert(second.gameAccountID == 62042 and second.characterName == 'Secondary')
            assert(C_BattleNet.GetFriendNumAccounts(1) == 2)
            assert(C_BattleNet.GetFriendNumAccounts(91017) == 0)
            assert(C_BattleNet.GetAccountInfoByGUID('Player-7-62042') == nil)
            assert(C_BattleNet.GetGameAccountInfoByGUID('BNet-0-91017') == nil)
            return true
            "#,
        )
        .expect("distinct account/game GUIDs, numeric output IDs and friend indices");
    assert!(result);

    env.state().borrow_mut().bnet_friends[0]
        .game_accounts
        .remove(1);
    let result: bool = env
        .eval(
            r#"
            assert(C_BattleNet.GetFriendNumAccounts(1) == 1)
            assert(C_BattleNet.GetGameAccountInfoByGUID('Player-7-62042') == nil)
            assert(C_BattleNet.GetAccountInfoByGUID('BNet-0-91017').bnetAccountID == 91017)
            return true
            "#,
        )
        .expect("game account removal updates count and lookup without losing its parent");
    assert!(result);
}

#[test]
fn patch_6_2_4_bare_successor_identity_state() {
    let env = WowLuaEnv::new().expect("bare environment");
    assert_account_and_game_identity_state(&env);
}

prefork_full_ui_case! {
fn patch_6_2_4_cached_successor_identity_state(env: &WowLuaEnv) {
    assert_account_and_game_identity_state(env);
}
}
