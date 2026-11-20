#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::c_api::club_model::{Club, Member, OWNER};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_original_addon(addon: &str) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &[addon], &[]);
    assert!(loaded.iter().any(|name| name == addon), "load {addon}");
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

fn supply_social_restrictions(env: &WowLuaEnv, muted: bool, age_restricted: bool, disabled: bool) {
    let mut sim = env.state().borrow_mut();
    sim.social_restrictions.is_muted = muted;
    sim.social_restrictions.age_verification_restricted = age_restricted;
    sim.social_restrictions.chat_disabled = disabled;
}

#[test]
fn original_social_settings_reads_chat_restrictions() {
    let env = load_original_addon("Blizzard_SettingsDefinitions_Frame");
    env.exec(r#"
        local setting = Settings.GetSetting("PROXY_DISABLE_CHAT")
        assert(setting ~= nil, "original Social.lua registered the disable-chat setting")
        assert(setting:GetValue() == false, "unrestricted account can disable chat")
    "#)
    .expect("original Social.lua disable-chat consumer");
    supply_social_restrictions(&env, true, false, false);
    assert_social_setting(&env, true);
    supply_social_restrictions(&env, false, true, false);
    assert_social_setting(&env, true);
    supply_social_restrictions(&env, false, false, true);
    assert_social_setting(&env, true);
    supply_social_restrictions(&env, false, false, false);
    assert_social_setting(&env, false);
}

fn assert_social_setting(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!(
        "assert(Settings.GetSetting('PROXY_DISABLE_CHAT'):GetValue() == {expected})"
    ))
    .expect("original Social.lua disable-chat value follows supplied restrictions");
}

fn supply_community(env: &WowLuaEnv) {
    let club = Club {
        id: "club-test".into(),
        name: "Test community".into(),
        description: String::new(),
        club_type: 1,
        avatar_id: 0,
        members: vec![Member::new("self", "Ari", OWNER, true)],
        candidates: Default::default(),
        invitations: Vec::new(),
        streams: Vec::new(),
    };
    env.state().borrow_mut().clubs.clubs.insert(club.id.clone(), club);
}

fn set_community_account_muted(env: &WowLuaEnv, muted: bool) {
    let mut sim = env.state().borrow_mut();
    if muted {
        sim.clubs.account_muted_club_ids.insert("club-test".into());
    } else {
        sim.clubs.account_muted_club_ids.remove("club-test");
    }
}

fn validate_community(env: &WowLuaEnv, expected_accessible: bool) {
    env.exec(&format!(
        r#"
        local source = debug.getinfo(CommunitiesFrameMixin.ValidateDisplayMode, "S").source:gsub("\\\\", "/")
        assert(source:find("Blizzard_Communities/CommunitiesFrame.lua", 1, true), source)
        local frame = setmetatable({
            displayMode = nil,
            ChatTab = {{ IconOverlay = {{ SetShown = function() end }} }},
            GuildMemberListDropdown = {{ SetShown = function() end }},
            CommunityMemberListDropdown = {{ SetShown = function() end }},
        }}, {{ __index = CommunitiesFrameMixin }})
        function frame:GetSelectedClubId() return "club-test" end
        function frame:SetDisplayMode(mode) self.displayMode = mode end
        function frame:HasCommunityFinderPermissions() return false end
        function frame:DisplayReportedAlerts() end
        function frame:UpdateMaximizeMinimizeButton() end
        function frame:CheckForTutorials() end
        CommunitiesFrameMixin.ValidateDisplayMode(frame)
        assert(frame:IsChatAccessible() == {expected_accessible}, "original communities consumer reflects chat restrictions")
        "#
    ))
    .expect("original CommunitiesFrameMixin display-mode consumer");
}

#[test]
fn original_communities_mode_reads_chat_restrictions() {
    let env = load_original_addon("Blizzard_Communities");
    supply_community(&env);
    validate_community(&env, true);
    set_community_account_muted(&env, true);
    validate_community(&env, false);
    set_community_account_muted(&env, false);
    supply_social_restrictions(&env, false, false, true);
    validate_community(&env, false);
    supply_social_restrictions(&env, false, false, false);
    validate_community(&env, true);
}
