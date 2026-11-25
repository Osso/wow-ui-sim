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

fn load_original_friends_frame() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let addons = ["Blizzard_FriendsFrame", "Blizzard_QuickJoin", "Blizzard_SocialUI"];
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &addons, &[]);
    for addon in addons {
        assert!(loaded.iter().any(|name| name == addon), "load {addon}");
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn original_friends_and_quick_join_consumers_follow_battle_net_connection() {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_FriendsFrame", "Blizzard_QuickJoin", "Blizzard_SocialUI"],
        &[],
    );
    for addon in ["Blizzard_FriendsFrame", "Blizzard_QuickJoin", "Blizzard_SocialUI"] {
        assert!(loaded.iter().any(|name| name == addon), "TOC selected {addon}");
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env.exec(
        r#"
        A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42, nil, false, false, true, false, false, true)
        assert(BNFeaturesEnabled() and not BNConnected())
        FriendsFrame_CheckBattlenetStatus()
        assert(FriendsFrameBattlenetFrame.UnavailableLabel:IsShown())
        assert(not FriendsFrameBattlenetFrame.Tag:IsShown())
        local menu = setmetatable({}, { __index = SocialUIBattleNetMenuButtonMixin })
        assert(not menu:ShouldShowBroadcastMenuOption())

        A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42, nil, false, false, true, false, true, true)
        assert(BNFeaturesEnabled() and BNConnected())
        FriendsFrame_CheckBattlenetStatus()
        assert(FriendsFrameBattlenetFrame.Tag:IsShown())
        assert(menu:ShouldShowBroadcastMenuOption())
        "#,
    )
    .expect("selected FriendsFrame, QuickJoin, and SocialUI paths follow supplied connection state");
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
        assert(C_AddOns.IsAddOnLoaded("Blizzard_Communities"), "profile TOC selected Communities")
        assert(C_AddOns.IsAddOnLoaded("Blizzard_CommunitiesSecure"), "profile TOC selected create-club dialog")
        A_Admin.SetClubTypeAllowed(Enum.ClubType.BattleNet, false)
        CommunitiesAddDialog:SetAttribute("setshown", true)
        assert(not CommunitiesAddDialog.CreateBattleNetGroupButton:IsEnabled())
        A_Admin.SetClubTypeAllowed(Enum.ClubType.BattleNet, true)
        CommunitiesAddDialog:SetAttribute("setshown", false)
        CommunitiesAddDialog:SetAttribute("setshown", true)
        assert(CommunitiesAddDialog.CreateBattleNetGroupButton:IsEnabled())
        CommunitiesAddDialog:SetAttribute("setshown", false)
        local addedClubs = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CLUB_ADDED")
        listener:SetScript("OnEvent", function(_, _, clubId)
            addedClubs[#addedClubs + 1] = clubId
        end)
        C_Club.CreateClub("Muted community", "", "Muted", 1, 0)
        C_Club.CreateClub("Available community", "", "Available", 1, 0)
        local mutedClubId, availableClubId = addedClubs[1], addedClubs[2]
        assert(mutedClubId and availableClubId and mutedClubId ~= availableClubId, "two clubs were created")
        assert(C_Club.GetClubInfo(mutedClubId).name == "Muted community", "muted club data is available")
        assert(C_Club.GetClubInfo(availableClubId).name == "Available community", "available club data is available")
        A_Admin.SetChatDisabled(false)
        ToggleCommunitiesFrame()
        CommunitiesFrame:SelectClub(availableClubId)
        assert(CommunitiesFrame:GetSelectedClubId() == availableClubId, "unrestricted club selected")
        assert(CommunitiesFrame:IsChatAccessible(), "unrestricted club chat is accessible")
        assert(
            CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.CHAT,
            "unrestricted club opens in chat display mode"
        )

        A_Admin.SetAccountMutedClub(mutedClubId, true)
        CommunitiesFrame:SelectClub(mutedClubId)
        assert(CommunitiesFrame:GetSelectedClubId() == mutedClubId, "muted club selected")
        assert(not CommunitiesFrame:IsChatAccessible(), "account-muted club blocks chat")
        assert(
            CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER,
            "account-muted club stays in roster display mode"
        )

        A_Admin.SetAccountMutedClub(mutedClubId, false)
        A_Admin.SetChatDisabled(true)
        CommunitiesFrame:SelectClub(availableClubId)
        assert(CommunitiesFrame:GetSelectedClubId() == availableClubId, "available club selected")
        assert(not CommunitiesFrame:IsChatAccessible(), "chat-disabled account blocks chat")
        assert(
            CommunitiesFrame:GetDisplayMode() == COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER,
            "chat-disabled account stays in roster display mode"
        )

        A_Admin.SetChatDisabled(false)
        A_Admin.SetAccountMutedClub(mutedClubId, false)
        CommunitiesFrame:SelectClub(mutedClubId)
        assert(CommunitiesFrame:IsChatAccessible(), "unrestricted community chat is accessible")
        "#;

#[test]
fn original_communities_frame_reads_chat_accessibility_inputs() {
    load_full_game_ui()
        .exec(COMMUNITIES_CHAT_ACCESSIBILITY_SCRIPT)
        .expect("original CommunitiesFrame follows supplied club and chat restrictions");
}

#[test]
fn admin_supplies_guild_calendar_availability() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_GuildInfo.AreGuildEventsEnabled() == false)
        A_Admin.SetGuildEventsEnabled(true)
        assert(C_GuildInfo.AreGuildEventsEnabled() == true)
        "#,
    )
    .expect("admin supplies guild calendar availability");
}

#[test]
fn admin_supplies_guild_preferred_play_settings_features() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        local canLocale, canDatacenter = C_GuildInfo.GetPreferredPlaySettingsFeatures()
        assert(not canLocale and not canDatacenter)
        A_Admin.SetGuildPreferredPlaySettingsFeatures(true, true)
        canLocale, canDatacenter = C_GuildInfo.GetPreferredPlaySettingsFeatures()
        assert(canLocale and canDatacenter)
        "#,
    )
    .expect("admin supplies guild preferred-play-setting features");
}

#[test]
fn admin_supplies_guild_reputation_availability() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_GuildInfo.IsGuildReputationEnabled() == false)
        A_Admin.SetGuildReputationEnabled(true)
        assert(C_GuildInfo.IsGuildReputationEnabled() == true)
        "#,
    )
    .expect("admin supplies guild reputation availability");
}

#[test]
fn admin_supplies_voice_chat_accessibility() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_VoiceChat.CanPlayerUseVoiceChat() == true)
        A_Admin.SetCanPlayerUseVoiceChat(false)
        assert(C_VoiceChat.CanPlayerUseVoiceChat() == false)
        "#,
    )
    .expect("admin supplies the voice-chat accessibility result");
}

#[test]
fn admin_supplies_community_voice_channel_errors() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_VoiceChat.GetJoinClubVoiceChannelError("club") == nil)
        A_Admin.SetJoinClubVoiceChannelError("club", 7)
        assert(C_VoiceChat.GetJoinClubVoiceChannelError("club") == 7)
        A_Admin.SetJoinClubVoiceChannelError("club", nil)
        assert(C_VoiceChat.GetJoinClubVoiceChannelError("club") == nil)
        "#,
    )
    .expect("admin supplies community voice-channel error state");
}

#[test]
fn admin_supplies_pending_community_voice_joins() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        local channelType = Enum.ChatChannelType.Communities
        assert(not C_VoiceChat.IsChannelJoinPending(channelType, "club", 20))
        A_Admin.SetChannelJoinPending(channelType, "club", 20, true)
        assert(C_VoiceChat.IsChannelJoinPending(channelType, "club", 20))
        A_Admin.SetChannelJoinPending(channelType, "club", 20, false)
        assert(not C_VoiceChat.IsChannelJoinPending(channelType, "club", 20))
        "#,
    )
    .expect("admin supplies pending community voice joins");
}

#[test]
fn admin_supplies_allowed_community_types() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_Club.ShouldAllowClubType(Enum.ClubType.BattleNet))
        assert(C_Club.ShouldAllowClubType(Enum.ClubType.Character))
        assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Other))
        A_Admin.SetClubTypeAllowed(Enum.ClubType.Character, false)
        assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Character))
        A_Admin.SetClubTypeAllowed(Enum.ClubType.Guild, false)
        assert(C_Club.ShouldAllowClubType(Enum.ClubType.Guild))
        A_Admin.SetClubTypeAllowed(Enum.ClubType.Other, true)
        assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Other))
        A_Admin.SetClubTypeAllowed(9999, true)
        assert(not C_Club.ShouldAllowClubType(9999))
        "#,
    )
    .expect("Guild and Other policy overrides supplied club-type state");
}

#[test]
fn original_friends_frame_renders_supplied_battle_net_counts_and_invites() {
    let env = load_original_friends_frame();
    env.exec(
        r#"
        A_Admin.SetBattleNetFriendInvite(1, 91, "Jaina", "Jaina#1234", 1, 1760000091)
        local total, online, favorite, favoriteOnline = BNGetNumFriends()
        assert(total == 2 and online == 1 and favorite == 1 and favoriteOnline == 0)
        assert(BNGetNumFriendInvites() == 1)
        local invite = C_BattleNet.GetFriendInviteInfo(1)
        assert(invite.inviteID == 91 and invite.accountName == "Jaina")
        assert(invite.battleTag == "Jaina#1234" and invite.friendLevel == 1)
        assert(invite.creationTimestamp == 1760000091)

        ToggleFriendsFrame(1)
        assert(FriendsFrame:IsShown() and FriendsListFrame:IsShown())
        FriendsList_Update(true)
        local friends, inviteHeaders, invites = 0, 0, 0
        for _, row in FriendsListFrame.ScrollBox:EnumerateDataProviderEntireRange() do
            if row.buttonType == FRIENDS_BUTTON_TYPE_BNET then friends = friends + 1 end
            if row.buttonType == FRIENDS_BUTTON_TYPE_INVITE_HEADER then inviteHeaders = inviteHeaders + 1 end
            if row.buttonType == FRIENDS_BUTTON_TYPE_INVITE then invites = invites + 1 end
        end
        assert(friends == 2 and inviteHeaders == 1 and invites == 1)
        "#,
    )
    .expect("original FriendsFrame renders supplied Battle.net friends and invite");
}

#[test]
fn battle_net_globals_follow_supplied_connection_and_account_state() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(BNFeaturesEnabled() == true)
        assert(BNConnected() == false)
        assert(BNFeaturesEnabledAndConnected() == false)
        assert(select(string.char(35), BNGetInfo()) == 0)

        A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42, nil, false, false, true, false, true, true)
        assert(BNConnected() == true)
        assert(BNFeaturesEnabledAndConnected() == true)
        assert(select(string.char(35), BNGetInfo()) == 8)
        local _, tag = BNGetInfo()
        assert(tag == "Malfurion#1234")

        A_Admin.SetPlayerBattleNetInfo(nil, 42, nil, false, false, true, false, true, true)
        local count, _, tag = select(string.char(35), BNGetInfo()), BNGetInfo()
        assert(count == 8 and tag == nil)
        A_Admin.SetPlayerBattleNetInfo("", 42, nil, false, false, true, false, true, true)
        local _, emptyTag = BNGetInfo()
        assert(emptyTag == "")

        A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42, nil, false, false, true, false, true, false)
        assert(BNFeaturesEnabled() == false)
        assert(BNConnected() == false)
        assert(select(string.char(35), BNGetInfo()) == 0)
        "#,
    )
    .expect("Battle.net globals follow supplied account and connection state");
}

#[test]
fn unmatched_community_voice_stream_accepts_numeric_stream_id() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"assert(C_VoiceChat.GetChannelForCommunityStream("club", 20) == nil)"#,
    )
    .expect("unmatched community voice stream returns nil");
}

fn assert_social_setting(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!(
        "assert(Settings.GetSetting('PROXY_DISABLE_CHAT'):GetValue() == {expected})"
    ))
    .expect("original Social.lua disable-chat value follows supplied restrictions");
}

#[test]
fn club_stream_focus_requires_a_supplied_club_stream() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r##"
        assert(C_Club.FocusStream("guild-0", 1) == true, "supplied guild stream focuses")
        assert(C_Club.FocusStream("guild-0", 999) == false, "missing stream does not focus")
        assert(C_Club.FocusStream("missing-club", 1) == false, "missing club does not focus")
        assert(select("#", C_Club.SetAutoAdvanceStreamViewMarker("guild-0", 1)) == 0)
        assert(select("#", C_Club.UnfocusStream("guild-0", 1)) == 0)
        assert(C_Club.FocusStream("guild-0", 1) == true)
        "##,
    )
    .expect("C_Club stream focus follows the supplied Rust club model");
}

#[test]
fn communities_finder_queries_report_unmodeled_postings() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        assert(C_ClubFinder.GetRecruitingClubInfoFromClubID("guild-0") == nil)
        C_ClubFinder.SetPlayerApplicantLocaleFlags(5)
        assert(C_ClubFinder.GetPlayerApplicantLocaleFlags() == 5)
        C_ClubFinder.SetPlayerApplicantLocaleFlags(9)
        assert(C_ClubFinder.GetPlayerApplicantLocaleFlags() == 9)
        "#,
    )
    .expect("C_ClubFinder returns supplied posting state or its empty default");
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
