#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
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
    env.exec(
        r#"
        local setting = Settings.GetSetting("PROXY_DISABLE_CHAT")
        assert(setting ~= nil, "original Social.lua registered the disable-chat setting")
        assert(setting:GetValue() == false, "unrestricted account can disable chat")
    "#,
    )
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

#[test]
fn original_communities_frame_reads_chat_accessibility_inputs() {
    let env = load_original_addon("Blizzard_Communities");
    env.exec(
        r#"
        C_Club.CreateClub("Muted community", "", "Muted", 1, 0)
        C_Club.CreateClub("Available community", "", "Available", 1, 0)
        local clubs = C_Club.GetSubscribedClubs()
        assert(#clubs == 2, "two distinct supplied clubs exist")
        local mutedClubId, availableClubId
        for _, club in ipairs(clubs) do
            if club.name == "Muted community" then mutedClubId = club.clubId end
            if club.name == "Available community" then availableClubId = club.clubId end
        end
        assert(mutedClubId ~= availableClubId, "club IDs are distinct")
        A_Admin.SetAccountMutedClub(mutedClubId, true)
        A_Admin.SetChatDisabled(true)

        ToggleCommunitiesFrame()
        CommunitiesFrame:SelectClub(mutedClubId, true)
        assert(not CommunitiesFrame:IsChatAccessible(), "account-muted club blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        A_Admin.SetAccountMutedClub(availableClubId, false)
        CommunitiesFrame:SelectClub(availableClubId, true)
        assert(not CommunitiesFrame:IsChatAccessible(), "chat-disabled account blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        A_Admin.SetChatDisabled(false)
        CommunitiesFrame:SelectClub(availableClubId, true)
        assert(CommunitiesFrame:IsChatAccessible(), "unrestricted community chat is accessible")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.CHAT)
        "#,
    )
    .expect("original CommunitiesFrame follows supplied club and chat restrictions");
}

fn assert_social_setting(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!(
        "assert(Settings.GetSetting('PROXY_DISABLE_CHAT'):GetValue() == {expected})"
    ))
    .expect("original Social.lua disable-chat value follows supplied restrictions");
}

#[test]
fn original_social_settings_reads_chat_disabled_admin_input() {
    let env = load_original_addon("Blizzard_SettingsDefinitions_Frame");
    env.exec(
        r#"
        local setting = Settings.GetSetting("PROXY_DISABLE_CHAT")
        assert(setting ~= nil, "original Social.lua registered the disable-chat setting")
        A_Admin.SetChatDisabled(false)
        assert(setting:GetValue() == false, "enabled chat remains enabled")
        A_Admin.SetChatDisabled(true)
        assert(setting:GetValue() == true, "disabled chat is reflected in the original setting")
        "#,
    )
    .expect("original Social.lua setting follows A_Admin-supplied chat-disabled state");
}
