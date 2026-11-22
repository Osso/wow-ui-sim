#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const IGNORE_LIST_SCENARIO: &str = r#"
    A_Admin.SetSocialIgnoreName(1, "R88eh-IgnoreAlpha")
    A_Admin.SetSocialIgnoreName(2, "R88eh-IgnoreBeta")
    A_Admin.SetSocialBlockedEntry(1, 92001, "R88eh-BlockedAlpha")
    assert(C_FriendList.GetNumIgnores() == 2, "both supplied ignores must be counted")
    assert(C_FriendList.GetIgnoreName(1) == "R88eh-IgnoreAlpha", "first supplied ignore")
    assert(C_FriendList.GetIgnoreName(2) == "R88eh-IgnoreBeta", "second supplied ignore")

    FriendsFrame:Show()
    FriendsFrame.IgnoreListWindow:Show()
    local scrollBox = FriendsFrame.IgnoreListWindow.ScrollBox
    local provider = scrollBox:GetDataProvider()
    assert(provider:GetSize() == 5, "provider must contain two headers and three rows")

    local function check(squelchType, index, name)
        local row = assert(scrollBox:FindFrameByPredicate(function(_, elementData)
            return elementData.squelchType == squelchType and elementData.index == index
        end), "original ignore row not materialized: " .. index)
        assert(row.name:GetText() == name, "original row must display supplied name")
        assert(row.type == squelchType, "original row must retain its row type")
        return row
    end

    local firstIgnore = check(SQUELCH_TYPE_IGNORE, 1, "R88eh-IgnoreAlpha")
    check(SQUELCH_TYPE_IGNORE, 2, "R88eh-IgnoreBeta")
    local blocked = check(SQUELCH_TYPE_BLOCK_INVITE, 1, "R88eh-BlockedAlpha")

    local selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_IGNORE and selectedIndex == 1,
        "original update must select the first ignore row")
    assert(firstIgnore:IsHighlightLocked(), "selected ignore row must be highlighted")

    FriendsFrame_SelectSquelched(SQUELCH_TYPE_BLOCK_INVITE, 1)
    selectedType, selectedIndex = IgnoreList_GetSelected()
    assert(selectedType == SQUELCH_TYPE_BLOCK_INVITE and selectedIndex == 1,
        "original selector must select the blocked row")
    assert(blocked:IsHighlightLocked(), "selected blocked row must be highlighted")
"#;

fn load_friends_frame() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("friend-list fixture environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env
}

#[test]
fn real_friends_frame_ignore_scrollbox_uses_supplied_names_and_selection() {
    load_friends_frame()
        .exec(IGNORE_LIST_SCENARIO)
        .expect("original FriendsFrame ignore ScrollBox should display supplied rows");
}
