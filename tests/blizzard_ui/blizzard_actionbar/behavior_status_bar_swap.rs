//! Behavior pin: `StatusTrackingManagerMixin:UpdateBarsShown` walks every
//! entry in `StatusTrackingBarInfo.BarsEnum`, keeps the ones whose
//! `CanShowBar(barIndex)` returns true, sorts the survivors by
//! `BarPriorities[barIndex]` (descending), truncates the list to
//! `#self.barContainers` (= 2 in the simulator: Main + Secondary), and
//! writes the truncated list to `self.shownBarIndices`. Seeding
//! `state.housing.tracked_house_guid` flips the `HouseFavor` arm's
//! `C_Housing.GetTrackedHouseGuid()` from nil to non-nil, so calling
//! `manager:UpdateBarsShown()` swaps `HouseFavor` (priority 5) into the
//! Main container, demotes `Experience` (priority 4) to the Secondary
//! container, and truncates `Reputation` (priority 1) off the end.
//!
//! ## Source contract (`Shared/StatusTrackingManager.lua:59-110`)
//!
//! ```lua
//! function StatusTrackingManagerMixin:UpdateBarsShown()
//!     ...
//!     local newBarIndicesToShow = {};
//!     for _, barIndex in pairs(StatusTrackingBarInfo.BarsEnum) do
//!         if self:CanShowBar(barIndex) then
//!             table.insert(newBarIndicesToShow, barIndex);
//!         end
//!     end
//!     table.sort(newBarIndicesToShow,
//!         function(left, right)
//!             return self:GetBarPriority(left) > self:GetBarPriority(right)
//!         end);
//!
//!     while #newBarIndicesToShow > #self.barContainers do
//!         table.remove(newBarIndicesToShow, #newBarIndicesToShow);
//!     end
//!
//!     for i = 1, #self.barContainers do
//!         ...
//!         barContainer:SetShownBar(newBarIndex);
//!     end
//!
//!     self.shownBarIndices = newBarIndicesToShow;
//!     self:UpdateBarVisuals();
//! end
//! ```
//!
//! `BarsEnum` (`StatusTrackingManager.lua:5-13`):
//! `Reputation=1, Honor=2, Artifact=3, Experience=4, Azerite=5, HouseFavor=6`.
//! `BarPriorities` (lua:15-22):
//! `Azerite=0, Reputation=1, Honor=2, Artifact=3, Experience=4, HouseFavor=5`.
//! Index space (`BarsEnum`) and priority space (`BarPriorities`) are
//! distinct -- `Azerite` is enum 5 but priority 0; `HouseFavor` is enum 6
//! but priority 5.
//!
//! `CanShowBar` (`Mainline/StatusTrackingManagerOverrides.lua:22-40`):
//! - Reputation: `C_Reputation.GetWatchedFactionData()` non-nil and
//!   `name ~= ""`. Cold simulator returns the canonical War Within
//!   faction (id 2590, non-empty name) per
//!   `globals/reputation_data.rs:214-217`, so this arm is **cold true**.
//! - Honor: `IsWatchingHonorAsXP() or C_PvP.IsActiveBattlefield() or
//!   IsInActiveWorldPVP()`. All three default false -> **cold false**.
//! - Artifact: `HasArtifactEquipped() and not IsEquippedArtifactMaxed()
//!   and not IsEquippedArtifactDisabled()`. `HasArtifactEquipped()` is
//!   stubbed to a hard `false` in `runtime_surface_bootstrap.lua:117-121`,
//!   so this arm is **always false** in the simulator (seeding
//!   `state.equipped_artifact` does NOT flip it because the stub does not
//!   read state). Pinning that surface is the artifact_bar test's job;
//!   here Artifact stays out of the eligible set.
//! - Experience: `not IsPlayerAtEffectiveMaxLevel() and not IsXPUserDisabled()
//!   and not C_GameRules.IsGameRuleActive(Enum.GameRule.ExperienceBarDisabled)`.
//!   Defaults are `is_max_level=false`, `xp_disabled=false`, empty game
//!   rules -> **cold true**.
//! - Azerite: `not C_AzeriteItem.IsAzeriteItemAtMaxLevel() and azeriteItem
//!   and azeriteItem:IsEquipmentSlot() and IsAzeriteItemEnabled(...)`.
//!   `state.azerite_item = None` -> `FindActiveAzeriteItem` returns nil ->
//!   short-circuits to **cold false**.
//! - HouseFavor: `C_Housing.GetTrackedHouseGuid()`. Defaults nil ->
//!   **cold false**. Seeding `state.housing.tracked_house_guid =
//!   Some("...")` flips it to non-nil -> **post-seed true**.
//!
//! ## Cold eligible set
//!
//! Cold `CanShowBar` returns true for `{Reputation(p=1), Experience(p=4)}`.
//! Sorted desc: `[Experience, Reputation]`. With 2 containers, both fit.
//! `manager.shownBarIndices` should equal `[Experience, Reputation]` after
//! the harness's startup `PLAYER_ENTERING_WORLD` runs OnEvent ->
//! `UpdateBarsShown` (manager OnEvent runs that for every registered event
//! when `UnitExists("player")` is true; Mainline registers
//! `PLAYER_ENTERING_WORLD` and 16 other events at lua:2-20).
//!
//! ## Post-seed eligible set
//!
//! Seeding `state.housing.tracked_house_guid = Some(...)` adds
//! `HouseFavor(p=5)` to the eligible set. Eligible = `{Rep(1), Exp(4),
//! HouseFavor(5)}`. Sorted desc: `[HouseFavor, Experience, Reputation]`.
//! Truncated to 2 containers: `[HouseFavor, Experience]`. Reputation
//! falls off the end.
//!
//! Calling `manager:UpdateBarsShown()` directly (rather than firing
//! `TRACKED_HOUSE_CHANGED` and waiting for OnEvent dispatch) isolates the
//! pin to the priority-sort + truncation logic. The OnEvent path is
//! pinned indirectly: `behavior_house_favor_bar.rs` already pins
//! `OnShow -> RegisterFrameForEvents -> OnEvent -> Update`, so adding an
//! OnEvent fire here would just duplicate that coverage.
//!
//! ## Why `manager.shownBarIndices` is the truth, not `container:GetShownBar()`
//!
//! `UpdateBarsShown` writes `self.shownBarIndices = newBarIndicesToShow;`
//! at lua:107 unconditionally. The per-container `shownBarIndex` only
//! advances when `SetShownBar`'s alpha branch (lua:194-205) routes
//! through `ApplyPendingBarToShow` -- which only happens when the
//! container is hidden / alpha 0, OR after the FadeOut animation
//! finishes. For a container that is currently showing one bar and is
//! told to switch to another, the immediate post-call state is:
//! `pendingBarToShowIndex = newIndex`, `shownBarIndex = oldIndex`
//! (animation in flight). The deterministic, animation-independent pin
//! is therefore on `manager.shownBarIndices`, not on
//! `container:GetShownBar()`.
//!
//! ## Container count is 2
//!
//! `Mainline/StatusTrackingBar.xml:41,49` defines exactly two frames that
//! inherit `StatusTrackingBarContainerTemplate` with
//! `parentArray="barContainers"`: `MainStatusTrackingBarContainer` and
//! `SecondaryStatusTrackingBarContainer`. Both attach to
//! `StatusTrackingBarManager.barContainers` (1-indexed) at XML load.
//!
//! ## Observations
//!
//! 1. After `with_blizzard_addon_startup_shape(&[Blizzard_ActionBar])`,
//!    `StatusTrackingBarManager` exists with `#barContainers == 2` and
//!    cold `manager.shownBarIndices == [Experience(4), Reputation(1)]`.
//! 2. After seeding `state.housing.tracked_house_guid = Some(...)` and
//!    calling `manager:UpdateBarsShown()`,
//!    `manager.shownBarIndices == [HouseFavor(6), Experience(4)]`.
//! 3. The eligible set has 3 entries (`Rep(1), Exp(4), HouseFavor(5)`)
//!    but the result has only 2 -- the truncation loop at lua:83-85 ran.
//!
//! ## Regression candidates the assertions catch
//!
//! - Priority comparator inverts (ascending instead of descending) ->
//!   `shownBarIndices == [Reputation, Experience]` cold and
//!   `[Reputation, Azerite]` post-seed (Azerite priority 0). The cold
//!   ordering check fires first.
//! - `BarPriorities` table swaps two entries (e.g. HouseFavor=4,
//!   Experience=5) -> post-seed `shownBarIndices[1] == Experience`
//!   instead of HouseFavor.
//! - Truncation loop direction inverts (removes from front instead of
//!   tail) -> post-seed `shownBarIndices == [Experience, Reputation]`
//!   (HouseFavor dropped).
//! - HouseFavor `CanShowBar` arm regresses (e.g. checks the wrong
//!   field) -> HouseFavor stays out of the eligible set, post-seed
//!   `shownBarIndices == [Experience, Reputation]`.
//! - `barContainers` array loses one entry (only Main remains) ->
//!   `#shownBarIndices == 1` post-seed.
//! - `UpdateBarsShown` early-returns and skips the assignment at lua:107
//!   (e.g. spurious `IsAnimating` regression) -> post-seed
//!   `shownBarIndices` stays at the cold value `[Experience, Reputation]`.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBar";

const MANAGER_LUA: &str = "StatusTrackingBarManager";

const REPUTATION_BAR_INDEX: i64 = 1;
const EXPERIENCE_BAR_INDEX: i64 = 4;
const HOUSE_FAVOR_BAR_INDEX: i64 = 6;
const EXPECTED_CONTAINER_COUNT: i64 = 2;
const SEEDED_HOUSE_GUID: &str = "test-house-guid";

post_wrath_ui_fixture! {
#[test]
fn manager_updates_bars_shown_to_priority_sorted_truncated_indices() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_cold_manager_globals_resolve(env);
        assert_cold_shown_bar_indices_match_priority(env);
        seed_tracked_house_guid(env);
        invoke_update_bars_shown(env);
        assert_post_update_shown_bar_indices(env);
    });
    }
}
}

fn assert_cold_manager_globals_resolve(env: &WowLuaEnv) {
    let cold_globals_exist: bool = env
        .eval(&format!(
            r#"
            local manager = {MANAGER_LUA}
            return manager ~= nil
                and type(manager.barContainers) == "table"
                and #manager.barContainers == {EXPECTED_CONTAINER_COUNT}
                and type(manager.shownBarIndices) == "table"
            "#,
        ))
        .expect("manager global existence probe must run cleanly");
    assert!(
        cold_globals_exist,
        "{MANAGER_LUA} must expose `barContainers` (length \
         {EXPECTED_CONTAINER_COUNT} -- Main + Secondary, see header) and a \
         `shownBarIndices` table after `{ROOT}` load; nil/wrong-length \
         means StatusTrackingBar.xml dropped one of the two containers \
         or `parentArray=\"barContainers\"` regressed -- see header \
         observation 1."
    );
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_cold_shown_bar_indices_match_priority(env: &WowLuaEnv) {
    let (length, first, second) = read_shown_bar_indices(env);
    assert_eq!(
        length, EXPECTED_CONTAINER_COUNT,
        "Cold `manager.shownBarIndices` must have length \
         {EXPECTED_CONTAINER_COUNT} (Reputation + Experience, both cold \
         eligible); got {length} -- startup OnEvent did not run \
         UpdateBarsShown, or CanShowBar arms regressed -- see header \
         observation 1."
    );
    assert_eq!(
        first, EXPERIENCE_BAR_INDEX,
        "Cold `manager.shownBarIndices[1]` must equal \
         {EXPERIENCE_BAR_INDEX} (Experience, priority 4 -- highest among \
         cold-eligible); got {first} -- priority comparator regressed to \
         ascending order -- see header observation 1."
    );
    assert_eq!(
        second, REPUTATION_BAR_INDEX,
        "Cold `manager.shownBarIndices[2]` must equal \
         {REPUTATION_BAR_INDEX} (Reputation, priority 1); got {second} -- \
         the only other cold-eligible bar dropped out of the list, or a \
         non-eligible bar (Honor / Artifact / Azerite / HouseFavor) \
         leaked in -- see header observation 1."
    );
}

fn seed_tracked_house_guid(env: &WowLuaEnv) {
    env.state().borrow_mut().housing.tracked_house_guid = Some(SEEDED_HOUSE_GUID.to_string());
}

fn invoke_update_bars_shown(env: &WowLuaEnv) {
    assert_house_favor_can_show(env);
    quiesce_container_fade_animations(env);
    call_update_bars_shown(env);
}

fn assert_house_favor_can_show(env: &WowLuaEnv) {
    let house_favor_can_show: bool = env
        .eval(&format!(
            "return {MANAGER_LUA}:CanShowBar(\
             StatusTrackingBarInfo.BarsEnum.HouseFavor) and true or false"
        ))
        .expect("manager:CanShowBar(HouseFavor) probe must run cleanly");
    assert!(
        house_favor_can_show,
        "After seeding `state.housing.tracked_house_guid = Some(...)`, \
         `{MANAGER_LUA}:CanShowBar(BarsEnum.HouseFavor)` must return \
         true -- the arm at StatusTrackingManagerOverrides.lua:35-37 \
         reads `C_Housing.GetTrackedHouseGuid()`. If false, either \
         seeding does not propagate to the C_Housing namespace or the \
         arm regressed."
    );
}

fn call_update_bars_shown(env: &WowLuaEnv) {
    let invoked: bool = env
        .eval(&format!("{MANAGER_LUA}:UpdateBarsShown(); return true"))
        .expect("manager:UpdateBarsShown must run cleanly");
    assert!(
        invoked,
        "`{MANAGER_LUA}:UpdateBarsShown()` must complete without error; \
         a Lua-level error in the dispatch path would have been surfaced \
         by `eval`."
    );
}

// Forever Camelot/StatusTrackingBarConstants.lua:21-28 sets Exp=0,
// Rep=2 and HouseFavor=5; Shared/StatusTrackingManager.lua:61 sorts descending.
#[cfg(feature = "client-wowforever")]
fn assert_cold_shown_bar_indices_match_priority(env: &WowLuaEnv) {
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
fn assert_post_update_shown_bar_indices(env: &WowLuaEnv) {
    assert_forever_bar_priorities(env);
    assert_eq!(
        read_shown_bar_indices(env),
        (
            EXPECTED_CONTAINER_COUNT,
            HOUSE_FAVOR_BAR_INDEX,
            REPUTATION_BAR_INDEX
        )
    );
}

#[cfg(feature = "client-wowforever")]
fn assert_forever_bar_priorities(env: &WowLuaEnv) {
    let priorities: (i64, i64, i64) = env.eval(
        "local p = StatusTrackingBarInfo.BarPriorities; local b = StatusTrackingBarInfo.BarsEnum; \
         return p[b.Experience], p[b.Reputation], p[b.HouseFavor]"
    ).unwrap();
    assert_eq!(priorities, (0, 2, 5));
    assert!(priorities.2 > priorities.1 && priorities.1 > priorities.0);
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_post_update_shown_bar_indices(env: &WowLuaEnv) {
    let (length, first, second) = read_shown_bar_indices(env);
    assert_eq!(
        length, EXPECTED_CONTAINER_COUNT,
        "Post-update `manager.shownBarIndices` must have length \
         {EXPECTED_CONTAINER_COUNT} (truncation loop at \
         StatusTrackingManager.lua:83-85 caps at \
         #self.barContainers); got {length} -- truncation regressed or \
         barContainers count changed -- see header observation 3."
    );
    assert_eq!(
        first, HOUSE_FAVOR_BAR_INDEX,
        "Post-update `manager.shownBarIndices[1]` must equal \
         {HOUSE_FAVOR_BAR_INDEX} (HouseFavor -- enum index 6, priority 5, \
         highest among the seeded eligible set Rep+Exp+HouseFavor); got \
         {first} -- either HouseFavor's CanShowBar arm regressed (seeding \
         tracked_house_guid did not flip it), or the priority \
         comparator regressed, or BarPriorities[HouseFavor] regressed \
         below 4 -- see header observation 2."
    );
    assert_eq!(
        second, EXPERIENCE_BAR_INDEX,
        "Post-update `manager.shownBarIndices[2]` must equal \
         {EXPERIENCE_BAR_INDEX} (Experience, priority 4 -- second \
         highest); got {second} -- Reputation (priority 1) leaked into \
         slot 2 instead of being truncated, meaning the priority sort \
         regressed or the truncation loop removed from the front \
         instead of the tail -- see header observation 3."
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
    assert!(
        quiesced,
        "After stopping every container's FadeIn/FadeOut/MaxLevel \
         animations, `IsAnimating()` must report false on every \
         container -- otherwise UpdateBarsShown's lua:67-71 early-return \
         path swallows the call without writing `shownBarIndices` and \
         the test cannot pin the swap. A residual `IsShownBarAnimating` \
         (StatusBar:IsAnimating or :IsDirty) means cold dispatch left \
         the inner StatusBar mid-update."
    );
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
