#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const BATTLE_NET_MENU_CASES: &str = r##"
A_Admin.SetBattleNetFriendState(1, true, false, false)
local accountInfo = C_BattleNet.GetFriendAccountInfo(1)
assert(accountInfo and accountInfo.bnetAccountID, "a supplied account can open its friend menu")
local menuContext = {
    friendsList = true,
    bnetIDAccount = accountInfo.bnetAccountID,
    name = accountInfo.accountName,
}
local tagLabel = SOCIAL_UI_BATTLE_NET_FRIEND_TAGS_LABEL:format(#(accountInfo.friendTags or {}))

local function has_tag_entry()
    local menu = Menu.GetManager():GetOpenMenu()
    if not menu then return false end
    for _, child in ipairs({ menu:GetChildren() }) do
        local description = child.GetElementDescription and child:GetElementDescription()
        if description and description:GetText() == tagLabel then return true end
    end
    return false
end

A_Admin.SetBattleNetAccountFeatures(false, true, true, true, true)
UnitPopup_OpenMenu("BN_FRIEND_OFFLINE", menuContext)
assert(Menu.GetManager():IsAnyMenuOpen(), "the public UnitPopup path opens the BNet friend menu")
assert(not has_tag_entry(), "disabled friend-tag feature hides its menu entry")

Menu.GetManager():CloseMenus()
A_Admin.SetBattleNetAccountFeatures(true, true, true, true, true)
UnitPopup_OpenMenu("BN_FRIEND_OFFLINE", menuContext)
assert(Menu.GetManager():IsAnyMenuOpen(), "the public UnitPopup path reopens the BNet friend menu")
assert(has_tag_entry(), "enabled friend-tag feature adds its menu entry")
"##;

#[test]
fn battle_net_friend_menu_tracks_the_supplied_tag_feature() {
    with_blizzard_addon_closure(&["Blizzard_UnitPopup"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitPopup"));
        env.exec(BATTLE_NET_MENU_CASES)
            .expect("original Battle.net friend menu should follow the account feature");
    });
}
