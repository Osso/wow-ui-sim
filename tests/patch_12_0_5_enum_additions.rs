#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_publication(enum_name: &str, expected: &str, count: usize, min: i32, max: i32) {
    let env = WowLuaEnv::new().unwrap();
    assert_publication_in_env(&env, enum_name, expected, count, min, max);
}

fn assert_publication_in_env(
    env: &WowLuaEnv,
    enum_name: &str,
    expected: &str,
    count: usize,
    min: i32,
    max: i32,
) {
    env.exec(&format!(
        r#"
        local values = Enum.{enum_name}
        local expected = {expected}
        for name, value in pairs(expected) do
            assert(values[name] == value, name .. " missing or wrong")
        end
        local count, min, max = 0, math.huge, -math.huge
        for _, value in pairs(values) do
            assert(type(value) == "number")
            count = count + 1
            min = math.min(min, value)
            max = math.max(max, value)
        end
        local meta = Enum.{enum_name}Meta
        assert(meta.NumValues == count,
            "incoherent count: metadata=" .. tostring(meta.NumValues) ..
            " published=" .. tostring(count))
        assert(meta.MinValue == min, "incoherent minimum")
        assert(meta.MaxValue == max, "incoherent maximum")
        assert(count == {count}, "unexpected member count")
        assert(min == {min} and max == {max}, "unexpected bounds")
        "#
    ))
    .unwrap();
}

#[test]
fn currency_flags_publish_no_bonus_xp_and_retain_old_members() {
    assert_publication(
        "CurrencyFlagsB",
        "{ CurrencyBUseTotalEarnedForEarned=1, CurrencyBShowQuestXPGainInTooltip=2,
        CurrencyBNoNotificationMailOnOfflineProgress=4, CurrencyBBattlenetVirtualCurrency=8,
        FutureCurrencyFlag=16, CurrencyBDontDisplayIfZero=32,
        CurrencyBScaleMaxQuantityBySeasonWeeks=64, CurrencyBScaleMaxQuantityByWeeksSinceStart=128,
        CurrencyBForceMaxQuantityOnConversion=256, CurrencyBUnearnableBeforeMaxQuantityStart=512,
        CurrencyBAllowReductionByResourcefulness=1024, CurrencyBNoBonusXP=2048 }",
        12,
        1,
        2048,
    );
}

#[test]
fn illusion_flags_publish_ranged_permission_and_retain_old_members() {
    let ptr = cfg!(feature = "client-ptr");
    let expected = if ptr {
        "{ HideUntilCollected=1, PlayerConditionGrantsOnLogin=2,
        AllowedRangedShieldsHoldables=4, HiddenIllusion=8 }"
    } else {
        "{ HideUntilCollected=1, PlayerConditionGrantsOnLogin=2,
        AllowedRangedShieldsHoldables=4 }"
    };
    assert_publication(
        "TransmogIllusionFlags",
        expected,
        if ptr { 4 } else { 3 },
        1,
        if ptr { 8 } else { 4 },
    );
}

#[test]
#[cfg(feature = "profile-retail")]
fn house_finder_publishes_home_owner_without_adding_later_reasons() {
    let env = WowLuaEnv::new().unwrap();
    let expected = "{ None=0, Owner=1, CharterInvite=2, Guild=4, BNetFriends=8,
        PartySync=16, Random=32, HomeOwner=64 }";
    // Shared 12.1 compatibility already publishes Relinquished=65.
    // Preserve it; correcting that later member is outside this patch.
    let later_epoch = cfg!(feature = "retail-12-1-0");
    let count = if later_epoch { 9 } else { 8 };
    let max = if later_epoch { 65 } else { 64 };
    assert_publication_in_env(&env, "HouseFinderSuggestionReason", expected, count, 0, max);
    #[cfg(feature = "retail-12-1-0")]
    {
        env.exec("assert(Enum.HouseFinderSuggestionReason.Relinquished == 65)")
            .unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
        assert_publication_in_env(&env, "HouseFinderSuggestionReason", expected, count, 0, max);
        env.exec("assert(Enum.HouseFinderSuggestionReason.Relinquished == 65)")
            .unwrap();
    }
}

// Current retail documentation can include later members and shifted numbers.
// Assert the patch deltas, not a historical closed member set. Cache provenance
// and every retained-source row are recorded in the enum additions spec.
#[cfg(feature = "client-retail")]
fn assert_current_publication(enum_name: &str, expected: &str, removed: &str) {
    let env = WowLuaEnv::new().unwrap();
    assert_current_publication_in_env(&env, enum_name, expected, removed);
    wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    assert_current_publication_in_env(&env, enum_name, expected, removed);
}

#[cfg(feature = "client-retail")]
fn assert_current_publication_in_env(
    env: &WowLuaEnv,
    enum_name: &str,
    expected: &str,
    removed: &str,
) {
    env.exec(&format!(
        r#"
        local values = Enum.{enum_name}
        assert(type(values) == "table", "Enum.{enum_name} missing")
        for name, value in pairs({expected}) do
            assert(values[name] == value,
                "Enum.{enum_name}." .. name .. " expected " .. value ..
                " got " .. tostring(values[name]))
        end
        for _, name in ipairs({removed}) do
            assert(values[name] == nil, "Enum.{enum_name}." .. name .. " obsolete alias")
        end
        local count, min, max = 0, math.huge, -math.huge
        for name, value in pairs(values) do
            assert(type(value) == "number", "Enum.{enum_name}." .. name .. " not numeric")
            count = count + 1
            min = math.min(min, value)
            max = math.max(max, value)
        end
        local meta = Enum.{enum_name}Meta
        assert(type(meta) == "table", "Enum.{enum_name}Meta missing")
        assert(meta.NumValues == count, "Enum.{enum_name}Meta incoherent count")
        assert(meta.MinValue == min, "Enum.{enum_name}Meta incoherent minimum")
        assert(meta.MaxValue == max, "Enum.{enum_name}Meta incoherent maximum")
        "#
    ))
    .unwrap();
}

#[test]
#[cfg(feature = "client-retail")]
fn abbreviation_errors_remove_invalid_abbreviation_and_publish_current_flags() {
    // LocalizationSharedDocumentation.lua:6-17; source delta:570.
    assert_current_publication(
        "AbbreviationDataError",
        "{ InvalidBreakpoint=1, InvalidSignificandDivisor=2, InvalidFractionDivisor=4,
        NotMultipleOfTen=8 }",
        "{ 'InvalidAbbreviation' }",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn addon_restrictions_publish_chat() {
    // RestrictedActionsConstantsDocumentation.lua:19-32; source delta:572.
    assert_current_publication("AddOnRestrictionType", "{ Combat=0, Chat=5 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn edit_mode_account_settings_publish_totem_action_bar() {
    // EditModeManagerConstantsDocumentation.lua:202-245; source delta:576.
    assert_current_publication("EditModeAccountSetting", "{ ShowTotemActionBar=34 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn edit_mode_status_tracking_settings_publish_size() {
    // EditModeManagerConstantsDocumentation.lua:610-621; source delta:578.
    assert_current_publication("EditModeStatusTrackingBarSetting", "{ Size=3 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn edit_mode_systems_publish_totem_action_bar() {
    // EditModeManagerConstantsDocumentation.lua:636-670; source delta:580.
    assert_current_publication("EditModeSystem", "{ TotemActionBar=25 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn edit_mode_unit_frame_settings_publish_big_defensive_icon_size() {
    // EditModeManagerConstantsDocumentation.lua:684-714; source delta:582.
    assert_current_publication(
        "EditModeUnitFrameSetting",
        "{ BigDefensiveIconSize=21 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn fragments_publish_dynamic_pathing_links_and_housing_decor_proxy_tag() {
    // WowCSConstantsDocumentation.lua:6-90; source deltas:584-585.
    assert_current_publication(
        "FragmentID",
        "{ FPathingDynamicLinks=40, TagHousingDecorProxyGameObject=226 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn account_tutorials_publish_housing_endeavors_tab_seen() {
    // TutorialDocumentation.lua:83-140; source delta:587.
    assert_current_publication(
        "FrameTutorialAccount",
        "{ HousingEndeavorsTabSeen=48 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn house_exterior_flags_publish_hidden_unless_owned() {
    // PlayerHousingConstantsDocumentation.lua:37-49; source delta:589.
    assert_current_publication("HouseExteriorWMODataFlags", "{ HiddenUnlessOwned=8 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn housing_decor_restrictions_publish_invalid_light_overlap() {
    // HousingDecorSharedDocumentation.lua:6-20; source delta:593.
    assert_current_publication(
        "HousingDecorPlacementRestriction",
        "{ InvalidLightOverlap=64 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn housing_item_toasts_rename_house_to_house_type_without_old_alias() {
    // HousingUIDocumentation.lua:945-957; source delta:595.
    assert_current_publication(
        "HousingItemToastType",
        "{ Room=0, Fixture=1, Customization=2, Decor=3, HouseType=4 }",
        "{ 'House' }",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn housing_results_publish_starting_area_binding_and_invalid_light_overlap() {
    // PlayerHousingConstantsDocumentation.lua:260-379; source deltas:597-598.
    // Current cumulative retail values, not inferred historical positions.
    assert_current_publication(
        "HousingResult",
        "{ BoundToStartingArea=19, InvalidLightOverlap=60 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn loot_method_styles_publish_mainline_without_personal_only_alias() {
    // LootConstantsDocumentation.lua:22-31; source deltas:600-601.
    assert_current_publication(
        "LootMethodStyles",
        "{ Mainline=0, Vanilla=1 }",
        "{ 'PersonalOnly' }",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn photo_sharing_upload_statuses_publish_disabled() {
    // ImageSharingConstantsDocumentation.lua:50-87; source delta:603.
    assert_current_publication("PhotoSharingUploadStatus", "{ Disabled=0 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn survey_delivery_moments_publish_encounter_end() {
    // WowSurveyConstantsDocumentation.lua:18-31; source delta:605.
    assert_current_publication("SurveyDeliveryMoment", "{ EncounterEnd=5 }", "{}");
}

#[test]
#[cfg(feature = "client-retail")]
fn transmog_situations_publish_weather_and_time_categories() {
    // TransmogOutfitConstantsDocumentation.lua:291-330; source deltas:609-618.
    assert_current_publication(
        "TransmogSituation",
        "{ AllWeather=22, WeatherClear=23, WeatherRain=24, WeatherSnow=25, WeatherSand=26,
        AllTime=27, TimeMorning=28, TimeDay=29, TimeEvening=30, TimeNight=31 }",
        "{}",
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn transmog_situation_triggers_publish_weather_and_time_of_day() {
    // TransmogOutfitConstantsDocumentation.lua:361-379; source deltas:620-621.
    assert_current_publication(
        "TransmogSituationTrigger",
        "{ Weather=9, TimeOfDay=10 }",
        "{}",
    );
}
