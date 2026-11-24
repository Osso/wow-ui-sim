#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/prefork_full_ui_preload.rs"]
mod prefork_full_ui_preload;

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_full_game_ui() -> WowLuaEnv {
    prefork_full_ui_preload::preload_full_game_ui()
        .expect("full Blizzard UI should load for the normal CommunitiesFrame path")
}

fn load_original_communities() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let addons = ["Blizzard_Communities", "Blizzard_CommunitiesSecure"];
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &addons, &[]);
    for addon in addons {
        assert!(loaded.iter().any(|name| name == addon), "load {addon}");
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
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

#[test]
fn original_communities_frame_reads_chat_accessibility_inputs() {
    let env = load_full_game_ui();
    env.exec(
        r#"
        local addedClubs = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CLUB_ADDED")
        listener:SetScript("OnEvent", function(_, _, clubId)
            addedClubs[#addedClubs + 1] = clubId
        end)
        C_Club.CreateClub("Muted community", "", "Muted", 1, 0)
        C_Club.CreateClub("Available community", "", "Available", 1, 0)
        local mutedClubId, availableClubId = addedClubs[1], addedClubs[2]
        assert(mutedClubId and availableClubId, "two supplied clubs publish distinct IDs")
        assert(mutedClubId ~= availableClubId, "club IDs are distinct")
        assert(C_Club.GetClubInfo(mutedClubId).name == "Muted community")
        assert(C_Club.GetClubInfo(availableClubId).name == "Available community")
        A_Admin.SetChatDisabled(true)
        ToggleCommunitiesFrame()
        local selectedClubId = CommunitiesFrame:GetSelectedClubId()
        assert(selectedClubId == mutedClubId or selectedClubId == availableClubId)

        A_Admin.SetAccountMutedClub(selectedClubId, true)
        CommunitiesFrame:ValidateDisplayMode()
        assert(not CommunitiesFrame:IsChatAccessible(), "account-muted club blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        A_Admin.SetAccountMutedClub(selectedClubId, false)
        CommunitiesFrame:ValidateDisplayMode()
        assert(not CommunitiesFrame:IsChatAccessible(), "chat-disabled account blocks chat")
        assert(CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER)

        A_Admin.SetChatDisabled(false)
        CommunitiesFrame:ValidateDisplayMode()
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
