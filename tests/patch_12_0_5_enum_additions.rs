#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_publication(enum_name: &str, expected: &str, count: usize, min: i32, max: i32) {
    let env = WowLuaEnv::new().unwrap();
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
        assert(meta.NumValues == count, "incoherent count")
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
    let later_epoch = cfg!(feature = "retail-12-1-5");
    let expected = if later_epoch {
        "{ HideUntilCollected=1, PlayerConditionGrantsOnLogin=2,
        AllowedRangedShieldsHoldables=4, HiddenIllusion=8 }"
    } else {
        "{ HideUntilCollected=1, PlayerConditionGrantsOnLogin=2,
        AllowedRangedShieldsHoldables=4 }"
    };
    assert_publication(
        "TransmogIllusionFlags",
        expected,
        if later_epoch { 4 } else { 3 },
        1,
        if later_epoch { 8 } else { 4 },
    );
}

#[test]
#[cfg(feature = "profile-retail")]
fn house_finder_publishes_home_owner_without_adding_later_reasons() {
    assert_publication(
        "HouseFinderSuggestionReason",
        "{ None=0, Owner=1, CharterInvite=2, Guild=4, BNetFriends=8,
        PartySync=16, Random=32, HomeOwner=64 }",
        8,
        0,
        64,
    );
}
