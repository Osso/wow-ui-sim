//! Behavior pin: active battlefields make Honor displace Reputation.
//!
//! The PLAN filename says "hides_xp_bar", but retail source does not make
//! `C_PvP.IsActiveBattlefield()` hide Experience. It makes Honor eligible;
//! then `StatusTrackingManagerMixin:UpdateBarsShown` priority-sorts and
//! truncates to the two available containers. The resulting pair is
//! Experience + Honor, so Reputation is the bar that drops.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBar";
const MANAGER_LUA: &str = "StatusTrackingBarManager";

const REPUTATION_BAR_INDEX: i64 = 1;
const HONOR_BAR_INDEX: i64 = 2;
const EXPERIENCE_BAR_INDEX: i64 = 4;
const EXPECTED_CONTAINER_COUNT: i64 = 2;

#[test]
fn active_battlefield_swaps_reputation_for_honor_status_bar() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_cold_status_bars_are_experience_and_reputation(env);

        seed_active_battlefield(env);
        quiesce_container_fade_animations(env);
        fire_battlefield_status_event(env);

        assert_honor_can_show_from_battlefield(env);
        assert_battlefield_status_bars_are_experience_and_honor(env);
    });
    }
}

fn seed_active_battlefield(env: &WowLuaEnv) {
    env.state().borrow_mut().is_active_battlefield = true;
}

fn fire_battlefield_status_event(env: &WowLuaEnv) {
    env.fire_event("ZONE_CHANGED_NEW_AREA")
        .expect("ZONE_CHANGED_NEW_AREA must dispatch cleanly");
}

fn assert_honor_can_show_from_battlefield(env: &WowLuaEnv) {
    let can_show_honor: bool = env
        .eval(&format!(
            "return C_PvP.IsActiveBattlefield() \
             and {MANAGER_LUA}:CanShowBar(StatusTrackingBarInfo.BarsEnum.Honor)"
        ))
        .expect("battlefield honor eligibility probe must run cleanly");
    assert!(
        can_show_honor,
        "active battlefield must make StatusTrackingManager eligible to show Honor"
    );
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_cold_status_bars_are_experience_and_reputation(env: &WowLuaEnv) {
    let (length, first, second) = read_shown_bar_indices(env);
    assert_eq!(length, EXPECTED_CONTAINER_COUNT);
    assert_eq!(first, EXPERIENCE_BAR_INDEX);
    assert_eq!(second, REPUTATION_BAR_INDEX);
}

// Forever Camelot/StatusTrackingBarConstants.lua:21-28 sets Exp=0,
// Rep=2 and Honor=3. Shared/StatusTrackingManager.lua:61 sorts descending.
#[cfg(feature = "client-wowforever")]
fn assert_cold_status_bars_are_experience_and_reputation(env: &WowLuaEnv) {
    assert_forever_bar_priorities(env);
    assert_eq!(
        read_shown_bar_indices(env),
        (
            EXPECTED_CONTAINER_COUNT,
            REPUTATION_BAR_INDEX,
            EXPERIENCE_BAR_INDEX
        )
    );
}

#[cfg(feature = "client-wowforever")]
fn assert_battlefield_status_bars_are_experience_and_honor(env: &WowLuaEnv) {
    assert_forever_bar_priorities(env);
    assert_eq!(
        read_shown_bar_indices(env),
        (
            EXPECTED_CONTAINER_COUNT,
            HONOR_BAR_INDEX,
            REPUTATION_BAR_INDEX
        )
    );
}

#[cfg(feature = "client-wowforever")]
fn assert_forever_bar_priorities(env: &WowLuaEnv) {
    let priorities: (i64, i64, i64) = env.eval(
        "local p = StatusTrackingBarInfo.BarPriorities; local b = StatusTrackingBarInfo.BarsEnum; \
         return p[b.Experience], p[b.Reputation], p[b.Honor]"
    ).unwrap();
    assert_eq!(priorities, (0, 2, 3));
    assert!(priorities.2 > priorities.1 && priorities.1 > priorities.0);
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_battlefield_status_bars_are_experience_and_honor(env: &WowLuaEnv) {
    let (length, first, second) = read_shown_bar_indices(env);
    assert_eq!(length, EXPECTED_CONTAINER_COUNT);
    assert_eq!(
        first, EXPERIENCE_BAR_INDEX,
        "Experience remains first because its priority is higher than Honor"
    );
    assert_eq!(
        second, HONOR_BAR_INDEX,
        "Honor must replace Reputation while active battlefield is true"
    );
}

fn quiesce_container_fade_animations(env: &WowLuaEnv) {
    let quiesced: bool = env
        .eval(&format!(
            r#"
            for _, c in ipairs({MANAGER_LUA}.barContainers) do
                if c.FadeInAnimation:IsPlaying() then c.FadeInAnimation:Stop() end
                if c.FadeOutAnimation:IsPlaying() then c.FadeOutAnimation:Stop() end
                if c.MaxLevelFadeOutAnimation:IsPlaying() then
                    c.MaxLevelFadeOutAnimation:Stop()
                end
            end
            for _, c in ipairs({MANAGER_LUA}.barContainers) do
                if c:IsAnimating() then return false end
            end
            return true
            "#,
        ))
        .expect("animation-quiesce probe must run cleanly");
    assert!(quiesced, "status tracking containers must be idle");
}

fn read_shown_bar_indices(env: &WowLuaEnv) -> (i64, i64, i64) {
    let length: i64 = env
        .eval(&format!("return #{MANAGER_LUA}.shownBarIndices"))
        .expect("shownBarIndices length probe must run cleanly");
    let first: i64 = env
        .eval(&format!("return {MANAGER_LUA}.shownBarIndices[1] or -1"))
        .expect("shownBarIndices[1] probe must run cleanly");
    let second: i64 = env
        .eval(&format!("return {MANAGER_LUA}.shownBarIndices[2] or -1"))
        .expect("shownBarIndices[2] probe must run cleanly");
    (length, first, second)
}
