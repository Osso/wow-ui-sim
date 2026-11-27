//! C_BattleNet title-friend names, friend tags, verified invites and the
//! appear-offline flag over the default Battle.net friend list. Retail 12.1 and
//! Forever declare these BattleNetDocumentation.lua blocks byte-identically.
#![cfg(feature = "bnet-friend-social")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn custom_title_friend_names_are_kept_per_friend() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        C_BattleNet.SetCustomTitleFriendName(1, "Raid Tank")
        C_BattleNet.SetCustomTitleFriendName(2, "   ")
        "#,
    )
    .unwrap();
    let names: (String, bool) = env
        .eval(
            "return C_BattleNet.GetCustomTitleFriendName(1), \
             C_BattleNet.GetCustomTitleFriendName(2) == nil",
        )
        .unwrap();
    assert_eq!(names, ("Raid Tank".to_string(), true));
    env.exec("C_BattleNet.SetCustomTitleFriendName(1, nil)").unwrap();
    let cleared: bool = env
        .eval("return C_BattleNet.GetCustomTitleFriendName(1) == nil")
        .unwrap();
    assert!(cleared);
}

#[test]
fn friend_tags_publish_through_account_info() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"C_BattleNet.SetFriendTags(2, { "raid", "pvp" })"#)
        .unwrap();
    let tags: (String, i64) = env
        .eval(
            "return table.concat(C_BattleNet.GetFriendAccountInfo(2).friendTags, ','), \
             #C_BattleNet.GetFriendAccountInfo(1).friendTags",
        )
        .unwrap();
    assert_eq!(tags, ("raid,pvp".to_string(), 0));
    env.exec("C_BattleNet.SetFriendTags(2, {})").unwrap();
    let remaining: i64 = env
        .eval("return #C_BattleNet.GetFriendAccountInfo(2).friendTags")
        .unwrap();
    assert_eq!(remaining, 0);
}

const INVITES_FOR: &str = r#"
    local function invitesFor(tag)
        local found = {}
        for index = 1, 50 do
            local invite = C_BattleNet.GetFriendInviteInfo(index)
            if invite and invite.battleTag:lower() == tag:lower() then
                found[#found + 1] = invite
            end
        end
        return found
    end
"#;

#[test]
fn verified_invites_record_each_battle_tag_once() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(&format!(
        r#"{INVITES_FOR}
        C_BattleNet.SendVerifiedBattleNetFriendInvite("  Thrall#1234 ")
        C_BattleNet.SendVerifiedBattleNetFriendInvite("thrall#1234")
        C_BattleNet.SendVerifiedBattleNetFriendInvite("")
        C_BattleNet.SendVerifiedBattleNetFriendInvite("Jaina#77")
        local thrall, jaina = invitesFor("Thrall#1234"), invitesFor("Jaina#77")
        assert(#thrall == 1 and #jaina == 1, "one pending invite per tag")
        assert(thrall[1].battleTag == "Thrall#1234" and thrall[1].accountName == "Thrall")
        assert(jaina[1].accountName == "Jaina")
        assert(jaina[1].inviteID == thrall[1].inviteID + 1, "invite IDs increase")
        "#
    ))
    .unwrap();
}

#[test]
fn appear_offline_and_title_invite_probe_have_documented_returns() {
    let env = WowLuaEnv::new().unwrap();
    let returns: i64 = env
        .eval("return select('#', C_BattleNet.SetAppearOffline(true))")
        .unwrap();
    assert_eq!(returns, 0);
    assert!(env.state().borrow().bnet_appear_offline);
    env.exec("C_BattleNet.SetAppearOffline(false)").unwrap();
    assert!(!env.state().borrow().bnet_appear_offline);
    let probe: (i64, bool) = env
        .eval(
            "return select('#', C_BattleNet.BNCheckTitleFriendInviteToUnit('target')), \
             C_BattleNet.BNCheckTitleFriendInviteToUnit('player')",
        )
        .unwrap();
    assert_eq!(probe, (1, false));
}
