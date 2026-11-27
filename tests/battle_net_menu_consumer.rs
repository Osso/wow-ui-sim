#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const BATTLE_NET_MENU_CASES: &str = r##"
A_Admin.SetBattleNetFriendState(1, true, false, false)
local accountInfo = C_BattleNet.GetFriendAccountInfo(1)
assert(accountInfo and accountInfo.bnetAccountID, "a supplied account can open its friend menu")
local contextData = { friendsList = true, accountInfo = accountInfo }
local button = UnitPopupBnetFriendTagsButtonMixin

A_Admin.SetBattleNetAccountFeatures(false, true, true, true, true)
assert(not button:CanShow(contextData), "disabled friend-tag feature hides its consumer entry")

A_Admin.SetBattleNetAccountFeatures(true, true, true, true, true)
assert(button:CanShow(contextData), "enabled friend-tag feature shows its consumer entry")
"##;

#[test]
fn battle_net_friend_menu_tracks_the_supplied_tag_feature() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        env.exec(BATTLE_NET_MENU_CASES)
            .expect("original Battle.net friend-menu consumer should follow the account feature");
    });
}
