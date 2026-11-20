#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn unit_popup_battle_tag_request_updates_club_request_state(env: &WowLuaEnv) {
    let member_name: String = env
        .eval("return C_Club.GetMemberInfo('guild-0', C_Club.GetClubMembers('guild-0')[2]).name")
        .unwrap();
    env.exec(r#"
        local memberIDs = C_Club.GetClubMembers('guild-0')
        BNGetInfo = function() return nil, 'Fixture#1234' end
        UnitPopupAddBtagFriendButtonMixin:OnClick({
            clubInfo = { clubId = 'guild-0' },
            clubMemberInfo = { memberId = memberIDs[2] },
        })
    "#).unwrap();
    assert_eq!(env.state().borrow().club_battle_tag_friend_requests, vec![member_name]);
}
}

prefork_full_ui_case! {
fn unit_popup_title_friend_request_updates_club_request_state(env: &WowLuaEnv) {
    let member_name: String = env
        .eval("return C_Club.GetMemberInfo('guild-0', C_Club.GetClubMembers('guild-0')[2]).name")
        .unwrap();
    env.exec(r#"
        local memberIDs = C_Club.GetClubMembers('guild-0')
        BNGetInfo = function() return nil, 'Fixture#1234' end
        UnitPopupAddTitleFriendButtonMixin:OnClick({
            clubInfo = { clubId = 'guild-0' },
            clubMemberInfo = { memberId = memberIDs[2] },
        })
    "#).unwrap();
    assert_eq!(env.state().borrow().title_friend_requests, vec![member_name]);
}
}
