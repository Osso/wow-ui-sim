//! Retail 12.1.5 / Forever 1.60.1 FriendListDocumentation.lua friend and
//! ignore edits over the social friend and ignore lists.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const FRIENDS_FRAME_CONSUMER: &str = r#"
    C_FriendList.AddFriend("Varian")
    C_FriendList.AddFriend("Jaina")
    ShowUIPanel(FriendsFrame)
    for _, expected in ipairs({ "Varian", "Jaina" }) do
        local button = FriendsListFrame.ScrollBox:FindFrameByPredicate(function(frame)
            return frame.name:GetText() == expected
        end)
        assert(button and button.name:GetText() == expected, expected)
    end
"#;

const FRIEND_EDITS: &str = r#"
    local before = C_FriendList.GetNumFriends()
    C_FriendList.AddFriend("Varian", "king")
    C_FriendList.AddFriend("Jaina")
    assert(C_FriendList.GetNumFriends() == before + 2)
    local varian = C_FriendList.GetFriendInfo("Varian")
    assert(varian.notes == "king" and varian.connected == false, varian.notes)
    assert(C_FriendList.GetFriendInfo("Jaina").notes == "")
    C_FriendList.AddFriend("Varian", "again")
    assert(C_FriendList.GetNumFriends() == before + 2)
    assert(C_FriendList.RemoveFriend("Jaina") == true)
    assert(C_FriendList.RemoveFriend("Jaina") == false)
    C_FriendList.AddOrRemoveFriend("Anduin", "prince")
    assert(C_FriendList.GetFriendInfo("Anduin").notes == "prince")
    C_FriendList.AddOrRemoveFriend("Anduin", "")
    assert(C_FriendList.GetFriendInfo("Anduin") == nil)
    assert(C_FriendList.GetNumFriends() == before + 1)
"#;

const FRIEND_INDICES: &str = r#"
    C_FriendList.AddFriend("Varian", "king")
    C_FriendList.AddFriend("Jaina", "mage")
    local count = C_FriendList.GetNumFriends()
    C_FriendList.SetSelectedFriend(count)
    local last = C_FriendList.GetFriendInfoByIndex(count).name
    assert(C_FriendList.GetSelectedFriend() == count)
    C_FriendList.SetFriendNotesByIndex(count, "renamed")
    assert(C_FriendList.GetFriendInfo(last).notes == "renamed")
    C_FriendList.RemoveFriendByIndex(count)
    assert(C_FriendList.GetFriendInfo(last) == nil)
    assert(C_FriendList.GetSelectedFriend() == nil)
"#;

const IGNORE_EDITS: &str = r#"
    local before = C_FriendList.GetNumIgnores()
    assert(C_FriendList.AddIgnore("Garrosh") == true)
    assert(C_FriendList.AddIgnore("garrosh") == false)
    assert(C_FriendList.IsIgnored("Garrosh") and C_FriendList.IsOnIgnoredList("Garrosh"))
    assert(not C_FriendList.IsIgnored("Sylvanas"))
    C_FriendList.AddOrDelIgnore("Sylvanas")
    assert(C_FriendList.IsOnIgnoredList("Sylvanas"))
    C_FriendList.AddOrDelIgnore("Sylvanas")
    assert(not C_FriendList.IsOnIgnoredList("Sylvanas"))
    assert(C_FriendList.GetNumIgnores() == before + 1)
    assert(C_FriendList.DelIgnore("Garrosh") == true)
    assert(C_FriendList.DelIgnore("Garrosh") == false)
    C_FriendList.AddIgnore("Gul'dan")
    C_FriendList.DelIgnoreByIndex(C_FriendList.GetNumIgnores())
    assert(not C_FriendList.IsIgnored("Gul'dan"))
"#;

#[test]
fn friends_frame_renders_friends_added_through_c_friend_list() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(FRIENDS_FRAME_CONSUMER)
            .expect("FriendsFrame renders both supplied friend names");
    });
}

#[test]
fn friend_edits_follow_the_friend_list() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(FRIEND_EDITS).expect("friend edits follow the friend list");
}

#[test]
fn friend_index_edits_follow_the_display_order() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(FRIEND_INDICES).expect("index edits follow the display order");
}

#[test]
fn ignore_edits_follow_the_ignore_list() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(IGNORE_EDITS).expect("ignore edits follow the ignore list");
}
