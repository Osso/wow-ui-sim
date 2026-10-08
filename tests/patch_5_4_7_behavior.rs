//! Bounded current backing behavior, not native 2014 transport parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_5_4_7_cached_bnet_sender_preserves_intent_and_void_return(env: &WowLuaEnv) {
    let game_account = {
        let mut sim = env.state().borrow_mut();
        sim.message_log.clear();
        let game = &mut sim.bnet_friends[0].game_accounts[0];
        game.is_online = true;
        game.game_account_id
    };
    let returned: i32 = env.eval(&format!(
        "return select('#', BNSendGameData({game_account}, 'P547', 'first payload'))"
    )).expect("cached legacy sender accepts online account");
    assert_eq!(returned, 0, "legacy wrapper discards namespace result");
    {
        let sim = env.state().borrow();
        assert_eq!(sim.message_log.len(), 1);
        let intent = &sim.message_log[0];
        assert_eq!(intent.kind, "bnet_game_data");
        assert_eq!(intent.prefix, "P547");
        assert_eq!(intent.message, "first payload");
        assert_eq!(intent.target, game_account.to_string());
        assert_eq!(intent.channel, "");
    }
    env.state().borrow_mut().bnet_friends[0].game_accounts[0].is_online = false;
    let returned: i32 = env.eval(&format!(
        "return select('#', BNSendGameData({game_account}, 'P547', 'offline payload'))"
    )).expect("cached wrapper accepts namespace rejection without exposing status");
    assert_eq!(returned, 0);
    let sim = env.state().borrow();
    assert_eq!(sim.message_log.len(), 1, "offline intent is rejected");
    assert_eq!(sim.message_log[0].message, "first payload");
}
}
