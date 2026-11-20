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

#[test]
fn original_social_settings_reads_chat_restrictions() {
    let env = load_original_addon("Blizzard_SettingsDefinitions_Frame");
    env.exec(r#"
        local setting = Settings.GetSetting("PROXY_DISABLE_CHAT")
        assert(setting ~= nil, "original Social.lua registered the disable-chat setting")
        assert(setting:GetValue() == false, "unrestricted account can disable chat")
    "#)
    .expect("original Social.lua disable-chat consumer");
}

#[test]
fn original_communities_mode_reads_chat_restrictions() {
    let env = load_original_addon("Blizzard_Communities");
    env.exec(r#"
        local source = debug.getinfo(CommunitiesFrameMixin.ValidateDisplayMode, "S").source:gsub("\\", "/")
        assert(source:find("Blizzard_Communities/CommunitiesFrame.lua", 1, true), source)
        local frame = setmetatable({
            displayMode = nil,
            ChatTab = { IconOverlay = { SetShown = function() end } },
            GuildMemberListDropdown = { SetShown = function() end },
            CommunityMemberListDropdown = { SetShown = function() end },
        }, { __index = CommunitiesFrameMixin })
        function frame:GetSelectedClubId() return "club-test" end
        function frame:SetDisplayMode(mode) self.displayMode = mode end
        function frame:HasCommunityFinderPermissions() return false end
        function frame:DisplayReportedAlerts() end
        function frame:UpdateMaximizeMinimizeButton() end
        function frame:CheckForTutorials() end
        CommunitiesFrameMixin.ValidateDisplayMode(frame)
        assert(frame:IsChatAccessible() == true, "unrestricted account can access club chat")
    "#)
    .expect("original CommunitiesFrameMixin display-mode consumer");
}
