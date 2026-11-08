//! Earlier-profile preservation for Forever enum additions; no Forever replay.
#![cfg(not(feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(not(feature = "client-wrath"))]
fn load_original_consumer(env: &WowLuaEnv, relative: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("active profile cache path");
    let path = root.join(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    env.exec(&source)
        .unwrap_or_else(|error| panic!("execute {}: {error}", path.display()));
}

// PTR 12.1.7.70493 declares these values natively, so it is not an earlier
// profile: pinned Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua
// declares EditModeSystem.GamepadMainActionBar at 782,
// EditModeGamepadMainActionBarSetting at 498-509 and EditModePresetLayouts.Gamepad at 648;
// RecentAlliesConstantsDocumentation.lua:6-18 declares the interaction filters.
#[cfg(feature = "client-ptr")]
#[test]
fn ptr_edit_mode_gamepad_publication_matches_pinned_declarations() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Enum.EditModeSystem.ActionBar == 0, "base ActionBar changed")
        assert(Enum.EditModeSystem.GamepadMainActionBar == 30, "PTR gamepad system")
        local systemMeta = Enum.EditModeSystemMeta
        assert(systemMeta.MinValue == 0 and systemMeta.MaxValue == 30)
        assert(systemMeta.NumValues == 31)
        local setting = Enum.EditModeGamepadMainActionBarSetting
        assert(setting.Compact == 0 and setting.ShowEmptyBars == 1)
        assert(setting.ShowButtonPrompts == 2 and setting.ShowHighlight == 3)
        assert(setting.ShouldExpand == 4)
        assert(Enum.EditModePresetLayouts.Gamepad == 2, "PTR gamepad preset")
        local filters = Enum.RecentAlliesInteractionCategoryFilter
        assert(filters.Professions == 0 and filters.PvP == 1 and filters.Raiding == 2)
        assert(filters.Dungeons == 3 and filters.Delves == 4 and filters.Questing == 5)
        "#,
    )
    .unwrap();
}

#[cfg(not(feature = "client-ptr"))]
#[test]
fn forever_gamepad_publication_is_absent_from_earlier_profiles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Enum.EditModeSystem.ActionBar == 0, "base ActionBar changed")
        assert(rawget(Enum.EditModeSystem, "GamepadMainActionBar") == nil,
            "Forever GamepadMainActionBar leaked outside Forever")
        for _, name in ipairs({"EditModeGamepadMainActionBarSetting",
            "EditModeGamepadMainActionBarSettingMeta"}) do
            assert(rawget(Enum, name) == nil, "Forever-only Enum." .. name .. " leaked")
        end
        assert(rawget(Enum.EditModePresetLayouts, "Gamepad") == nil,
            "Forever Gamepad preset leaked outside Forever")
        for _, name in ipairs({"RecentAlliesInteractionCategoryFilter",
            "RecentAlliesInteractionCategoryFilterMeta"}) do
            assert(rawget(Enum, name) == nil, "Forever-only Enum." .. name .. " leaked")
        end
        "#,
    )
    .unwrap();
}

#[cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn classic_profiles_keep_shared_modern_enum_group_absent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({"ActionBarSet", "BattleNetFriendLevel", "VisualAlertType",
            "CooldownViewerSound", "BattleNetFriendTag", "SocialUIPresenceType",
            "SocialSystemType", "SocialUIBlockType", "CombatAudioAlertPulsePercentValues"}) do
            assert(rawget(Enum, name) == nil, "modern Enum." .. name .. " leaked")
            assert(rawget(Enum, name .. "Meta") == nil, "modern metadata leaked: " .. name)
        end
        assert(Enum.RolodexType.PvPKill == 20, "base Rolodex value changed")
        assert(rawget(Enum.RolodexType, "LegacyFriend") == nil,
            "modern RolodexType.LegacyFriend leaked")
        "#,
    )
    .unwrap();
}

// Wrath has no matching CommunitiesUtil file in the current profile cache.
#[cfg(feature = "client-wrath")]
#[test]
fn wrath_keeps_original_stream_numbers_without_discord() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(Enum.ClubStreamType.General == 0 and Enum.ClubStreamType.Guild == 1)
        assert(Enum.ClubStreamType.Officer == 2 and Enum.ClubStreamType.Other == 3)
        assert(rawget(Enum.ClubStreamType, "Discord") == nil)
        local meta = Enum.ClubStreamTypeMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 3 and meta.NumValues == 4)
        "#,
    )
    .unwrap();
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn original_communities_sort_preserves_profile_stream_ids() {
    let env = WowLuaEnv::new().unwrap();
    load_original_consumer(&env, "Blizzard_FrameXMLUtil/CommunitiesUtil.lua");
    let cases = if cfg!(any(feature = "profile-retail", feature = "client-ptr")) {
        r#"
        local streams = {{streamType=4, creationTime=2}, {streamType=3, creationTime=2},
            {streamType=2, creationTime=2}, {streamType=0, creationTime=2},
            {streamType=1, creationTime=2}, {streamType=3, creationTime=1}}
        CommunitiesUtil.SortStreams(streams)
        local expected = {1, 0, 2, 3, 3, 4}
        for index, value in ipairs(expected) do assert(streams[index].streamType == value) end
        assert(streams[4].creationTime == 1 and streams[5].creationTime == 2)
        assert(Enum.ClubStreamType.Discord == 3 and Enum.ClubStreamType.Other == 4)
        local meta = Enum.ClubStreamTypeMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 4 and meta.NumValues == 5)
        "#
    } else {
        r#"
        local streams = {{streamType=3, creationTime=2}, {streamType=2, creationTime=2},
            {streamType=0, creationTime=2}, {streamType=1, creationTime=2},
            {streamType=3, creationTime=1}}
        CommunitiesUtil.SortStreams(streams)
        local expected = {1, 0, 2, 3, 3}
        for index, value in ipairs(expected) do assert(streams[index].streamType == value) end
        assert(streams[4].creationTime == 1 and streams[5].creationTime == 2)
        assert(rawget(Enum.ClubStreamType, "Discord") == nil)
        assert(Enum.ClubStreamType.Other == 3)
        local meta = Enum.ClubStreamTypeMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 3 and meta.NumValues == 4)
        "#
    };
    env.exec(cases).unwrap();
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
#[test]
fn original_account_consumer_keeps_friend_levels_and_ranks() {
    let env = WowLuaEnv::new().unwrap();
    load_original_consumer(&env, "Blizzard_SharedXML/AccountUtil.lua");
    env.exec(
        r#"
        assert(BNet_GetFriendLevelRank(1) == 2)
        assert(BNet_GetFriendLevelRank(2) == 3)
        assert(BNet_GetFriendLevelRank(3) == 1)
        assert(BNet_IsFriendLevelEqualOrHigher(2, 3))
        assert(not BNet_IsFriendLevelEqualOrHigher(3, 2))
        assert(Enum.BattleNetFriendLevel.BattleTag == 1)
        assert(Enum.BattleNetFriendLevel.RealID == 2 and Enum.BattleNetFriendLevel.Title == 3)
        local meta = Enum.BattleNetFriendLevelMeta
        assert(meta.MinValue == 1 and meta.MaxValue == 3 and meta.NumValues == 3)
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
#[test]
fn original_sound_data_keeps_numeric_sound_ids_and_kits() {
    let env = WowLuaEnv::new().unwrap();
    load_original_consumer(
        &env,
        "Blizzard_CooldownViewer/CooldownViewerSettingsConstants.lua",
    );
    load_original_consumer(
        &env,
        "Blizzard_CooldownViewer/CooldownViewerSoundAlertData.lua",
    );
    env.exec(
        r#"
        local cat = CooldownViewerSoundData[1][1]
        assert(cat.soundEnum == 1 and cat.soundKitID == 316401)
        assert(Enum.CooldownViewerSound.TextToSpeech == 0)
        assert(Enum.CooldownViewerSound.ShortWoodXylophone == 93)
        local meta = Enum.CooldownViewerSoundMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 93 and meta.NumValues == 94)
        "#,
    )
    .unwrap();
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
#[test]
fn original_visual_alert_lookup_keeps_numeric_style_ids() {
    let env = WowLuaEnv::new().unwrap();
    load_original_consumer(&env, "Blizzard_VisualAlerts/VisualAlert.lua");
    load_original_consumer(&env, "Blizzard_VisualAlerts/VisualAlertTemplates.lua");
    env.exec(
        r#"
        assert(VisualAlert_GetTypeTemplate(1) == "VisualAlertMarchingAntsBaseTemplate")
        assert(VisualAlert_GetTypeTemplate(6) == "VisualAlertFlashBaseTemplate")
        assert(VisualAlert_GetTypeTemplate(10) == "VisualAlertFlashBlueTemplate")
        assert(Enum.VisualAlertType.MarchingAnts == 1 and Enum.VisualAlertType.Flash == 6)
        assert(Enum.VisualAlertType.FlashBlue == 10)
        local meta = Enum.VisualAlertTypeMeta
        assert(meta.MinValue == 1 and meta.MaxValue == 10 and meta.NumValues == 10)
        "#,
    )
    .unwrap();
}
