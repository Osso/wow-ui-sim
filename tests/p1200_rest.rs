//! Behavioral coverage for remaining 12.0.0 namespace producers.
#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_battle_net_outbound() {
    let env = WowLuaEnv::new().unwrap();
    let (account, game) = {
        let state = env.state().borrow();
        let friend = &state.bnet_friends[0];
        (
            friend.bnet_account_id,
            friend.game_accounts[0].game_account_id,
        )
    };
    env.exec(&format!(
        r#"
        assert(C_BattleNet.SendGameData({game}, 'SIM', 'payload') == 0)
        assert(C_BattleNet.SendGameData(-1, 'SIM', 'payload') == 9)
        assert(C_BattleNet.SendWhisper({account}, 'hello') == true)
        assert(C_BattleNet.SendWhisper(-1, 'hello') == false)
        assert(C_BattleNet.SetCustomMessage('Ready for raid') == true)
    "#
    ))
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.bnet_custom_message, "Ready for raid");
    assert_eq!(state.message_log.len(), 2);
    assert_eq!(state.message_log[0].kind, "bnet_game_data");
    assert_eq!(state.message_log[0].message, "payload");
    assert_eq!(state.message_log[1].target, account.to_string());
    assert_eq!(state.message_log[1].message, "hello");
}
