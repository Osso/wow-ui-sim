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
