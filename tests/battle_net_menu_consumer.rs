#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const BATTLE_NET_MENU_CASES: &str = r##"
ToggleFriendsFrame(FRIEND_TAB_FRIENDS)
assert(FriendsFrame:IsShown(), "the public friends-frame toggle opens the list")

local function first_online_friend()
    return FriendsListFrame.ScrollBox:FindFrameByPredicate(function(_, data)
        return data.accountInfo and data.accountInfo.gameAccountInfo.isOnline
    end)
end

local function has_friend_tags_entry()
    local menu = Menu.GetManager():GetOpenMenu()
    if not menu then return false end
    for _, child in ipairs({ menu:GetChildren() }) do
        local description = child.GetElementDescription and child:GetElementDescription()
        if description and description:GetText() == SOCIAL_UI_BATTLE_NET_FRIEND_TAGS_LABEL:format(0) then
            return true
        end
    end
    return false
end

local friend = first_online_friend()
assert(friend, "an online Battle.net friend card is visible")
A_Admin.SetBattleNetAccountFeatures(false, true, true, true, true)
friend:Click("RightButton")
assert(Menu.GetManager():IsAnyMenuOpen(), "right-click opens the friend context menu")
assert(not has_friend_tags_entry(), "disabled friend-tag feature hides its menu entry")

Menu.GetManager():CloseMenus()
A_Admin.SetBattleNetAccountFeatures(true, true, true, true, true)
friend:Click("RightButton")
assert(Menu.GetManager():IsAnyMenuOpen(), "right-click reopens the friend context menu")
assert(has_friend_tags_entry(), "enabled friend-tag feature adds its menu entry")
"##;

#[test]
fn battle_net_friend_menu_tracks_the_supplied_tag_feature() {
    with_blizzard_addon_closure(
        &["Blizzard_FriendsFrame", "Blizzard_UnitPopup"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|addon| addon == "Blizzard_FriendsFrame"));
            env.exec(BATTLE_NET_MENU_CASES)
                .expect("original Battle.net friend menu should follow the account feature");
        },
    );
}
