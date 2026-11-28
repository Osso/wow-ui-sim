//! Supplied RAF cooldown snapshots; zero/false defaults are simulator policy.

#[cfg(feature = "client-retail")]
use sha2::{Digest, Sha256};

use wow_ui_sim::c_api::c_recruit_a_friend::SummonFriendCooldown;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn raf_cooldown_unsupplied_has_zero_timing_and_disabled_timer() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local start, duration, enabled = C_RecruitAFriend.GetSummonFriendCooldown()
        assert(start == 0 and duration == 0 and enabled == false)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_social_card_receives_supplied_cooldown_and_enable_flag() {
    let env = shared_xml_base_env();
    load_original_retail_social_card_sources(&env);
    assert_retail_social_card_starts_without_cooldown(&env);
    env.state().borrow_mut().summon_friend_cooldown = SummonFriendCooldown {
        start_time_seconds: 120.0,
        duration_seconds: 600.0,
        enable_cooldown_timer: true,
    };
    assert_retail_social_card_uses_supplied_cooldown(&env);
    env.state()
        .borrow_mut()
        .summon_friend_cooldown
        .enable_cooldown_timer = false;
    assert_retail_social_card_timer_can_be_disabled(&env);
    assert!(env.state().borrow().lua_errors.is_empty());
}

/// FriendsListTemplates.lua:361 reads EnumUtil, whose original owner is
/// Blizzard_SharedXMLBase since 096d8a1f8c retired the bootstrap copy.
///
/// This file builds both as its own `[[test]]` target and inside the generated
/// integration harness, so it loads the closure through the library API rather
/// than `crate::common`, which only the harness provides.
#[cfg(feature = "client-retail")]
fn shared_xml_base_env() -> WowLuaEnv {
    use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    let closure = discover_blizzard_addon_closure_for_screen(
        &ui,
        wow_ui_sim::screen::ScreenKind::Game,
        &["Blizzard_SharedXMLBase"],
    );
    for (name, toc) in closure {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    env
}

#[cfg(feature = "client-retail")]
fn load_original_retail_social_card_sources(env: &WowLuaEnv) {
    let cache = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    for (file, digest) in [
        (
            "Blizzard_FrameXMLUtil/Cooldown.lua",
            "775f21d2de4fb6f49df103cc4564dcea21b0d30a1d4f0ba0299292a878347e11",
        ),
        (
            "Blizzard_FriendsFrame/Mainline/FriendsListTemplates.lua",
            "3e36c8bfb43c342292d8a4542674a67f4930a573eb4f61702bf480f50e58372f",
        ),
    ] {
        let source = std::fs::read(cache.join(file)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&source)), digest);
        env.loader_env()
            .exec(std::str::from_utf8(&source).unwrap())
            .unwrap();
    }
}

#[cfg(feature = "client-retail")]
fn assert_retail_social_card_starts_without_cooldown(env: &WowLuaEnv) {
    env.exec(
        r#"
        RafCooldownCard = CreateFrame('Frame')
        RafCooldownCard.Cooldown = CreateFrame('Cooldown', nil, RafCooldownCard)
        RafCooldownCard.Cooldown:SetCooldown(12, 34)
        FriendsListSocialCardRAFSummonButtonMixin.RefreshCooldown(RafCooldownCard)
        assert(RafCooldownCard.cooldownStartTime == 0)
        assert(RafCooldownCard.cooldownDurationSeconds == 0)
        local start, duration = RafCooldownCard.Cooldown:GetCooldownTimes()
        assert(start == 0 and duration == 0)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
fn assert_retail_social_card_uses_supplied_cooldown(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListSocialCardRAFSummonButtonMixin.RefreshCooldown(RafCooldownCard)
        assert(RafCooldownCard.cooldownStartTime == 120)
        assert(RafCooldownCard.cooldownDurationSeconds == 600)
        local start, duration = RafCooldownCard.Cooldown:GetCooldownTimes()
        assert(start == 120000 and duration == 600000)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
fn assert_retail_social_card_timer_can_be_disabled(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListSocialCardRAFSummonButtonMixin.RefreshCooldown(RafCooldownCard)
        local start, duration = RafCooldownCard.Cooldown:GetCooldownTimes()
        assert(start == 0 and duration == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn raf_cooldown_reads_supplied_start_duration_and_timer_flag() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().summon_friend_cooldown = SummonFriendCooldown {
        start_time_seconds: 120.0,
        duration_seconds: 600.0,
        enable_cooldown_timer: true,
    };
    env.exec(
        r#"
        local start, duration, enabled = C_RecruitAFriend.GetSummonFriendCooldown()
        assert(start == 120 and duration == 600 and enabled == true)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .summon_friend_cooldown
        .enable_cooldown_timer = false;
    env.exec(
        r#"
        local start, duration, enabled = C_RecruitAFriend.GetSummonFriendCooldown()
        assert(start == 120 and duration == 600 and enabled == false)
    "#,
    )
    .unwrap();
}
