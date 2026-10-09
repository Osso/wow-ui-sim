//! Local Battle.net model proof, not native-client signature/error parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_4_2_0_friend_index_tracks_account_ids_and_current_order() {
    let env = WowLuaEnv::new().unwrap();
    let (first, second): (i32, i32) = env
        .eval("return BNGetFriendIndex(100001), BNGetFriendIndex(100002)")
        .unwrap();
    assert_eq!((first, second), (1, 2));

    env.state().borrow_mut().bnet_friends.swap(0, 1);
    let (index, account_id): (i32, i32) = env
        .eval(
            "local index = BNGetFriendIndex(100001); \
             return index, C_BattleNet.GetFriendAccountInfo(index).bnetAccountID",
        )
        .unwrap();
    assert_eq!((index, account_id), (2, 100001));

    env.state().borrow_mut().bnet_friends.remove(0);
    let (index, removed_is_nil): (i32, bool) = env
        .eval("return BNGetFriendIndex(100001), BNGetFriendIndex(100002) == nil")
        .unwrap();
    assert_eq!(index, 1);
    assert!(removed_is_nil);
}

#[test]
fn patch_4_2_0_friend_index_returns_nil_for_unknown_or_empty_list() {
    let env = WowLuaEnv::new().unwrap();
    let unknown_is_nil: bool = env.eval("return BNGetFriendIndex(999999) == nil").unwrap();
    assert!(unknown_is_nil);

    env.state().borrow_mut().bnet_friends.clear();
    let empty_is_nil: bool = env.eval("return BNGetFriendIndex(100001) == nil").unwrap();
    assert!(empty_is_nil);
}
