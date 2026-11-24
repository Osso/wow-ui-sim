#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/prefork_full_ui_preload.rs"]
mod prefork_full_ui_preload;

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_full_game_ui() -> WowLuaEnv {
    prefork_full_ui_preload::preload_full_game_ui()
        .expect("full Blizzard UI should load for the normal CommunitiesFrame path")
}

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

const COMMUNITIES_CHAT_ACCESSIBILITY_SCRIPT: &str = r#"
        local addedClubId
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CLUB_ADDED")
        listener:SetScript("OnEvent", function(_, _, clubId) addedClubId = clubId end)
        C_Club.CreateClub("Available community", "", "Available", 1, 0)
        assert(addedClubId and addedClubId ~= "guild-0", "two distinct clubs exist")
        assert(C_Club.GetClubInfo(addedClubId).name == "Available community")
        assert(C_Club.GetClubInfo("guild-0") ~= nil, "guild club is supplied by the Rust model")

        A_Admin.SetAccountMutedClub("guild-0", true)
        A_Admin.SetChatDisabled(false)
        ToggleCommunitiesFrame()
        assert(CommunitiesFrame:GetSelectedClubId() == "guild-0")
        assert(not CommunitiesFrame:IsChatAccessible(), "account-muted club blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        ToggleCommunitiesFrame()
        A_Admin.SetAccountMutedClub("guild-0", false)
        A_Admin.SetChatDisabled(true)
        ToggleCommunitiesFrame()
        assert(not CommunitiesFrame:IsChatAccessible(), "chat-disabled account blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        ToggleCommunitiesFrame()
        A_Admin.SetChatDisabled(false)
        ToggleCommunitiesFrame()
        assert(CommunitiesFrame:IsChatAccessible(), "unrestricted community chat is accessible")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.CHAT)
        "#;

#[test]
fn original_communities_frame_reads_chat_accessibility_inputs() {
    load_full_game_ui()
        .exec(COMMUNITIES_CHAT_ACCESSIBILITY_SCRIPT)
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
