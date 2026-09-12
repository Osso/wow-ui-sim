//! Behavior pin: `ReputationStatusBarMixin:Update`
//! (`Blizzard_ActionBar/Shared/ReputationBar.lua:77-169`) consumes
//! `C_Reputation.GetWatchedFactionData()` and dispatches across three
//! visual variants:
//!
//! 1. **Standard rep** — when the watched faction is neither paragon
//!    nor a major faction, the bar shows
//!    `(currentStanding, currentReactionThreshold..nextReactionThreshold)`
//!    on the reaction-coloured atlas (`UI-HUD-...-Red/Orange/Yellow/Green`),
//!    `level = watchedFactionData.reaction`, `maxLevel = MAX_REPUTATION_REACTION = 8`.
//! 2. **Major-faction (Renown)** — when
//!    `C_Reputation.IsMajorFaction(factionID)` is true, the bar shows
//!    `(currentStanding, 0..renownLevelThreshold)` on the blue Renown
//!    atlas (`overrideUseBlueBar = true`), `level = renownLevel`,
//!    `maxLevel = renownLevels[#].level` from
//!    `C_MajorFactions.GetRenownLevels`. Note `value` is left as
//!    `currentStanding` (the major-faction branch at lua:112-116
//!    overwrites only `minBar/maxBar/level/overrideUseBlueBar`,
//!    not `value`); a side effect is that `bar.value` can exceed
//!    `bar.max` when `currentStanding > renownLevelThreshold`.
//! 3. **Paragon** — when
//!    `C_Reputation.IsFactionParagonForCurrentPlayer(factionID)` is
//!    true, the bar shows `(currentValue % threshold, 0..threshold)`
//!    where `(currentValue, threshold, _, hasRewardPending)` come from
//!    `C_Reputation.GetFactionParagonInfo`. `hasRewardPending` adds
//!    `threshold` to `value` (overflowing the bar visually as a
//!    "ready to claim" indicator). `level = maxLevel = nil`, so
//!    `isCapped` short-circuits false.
//!
//! ## Source contract
//!
//! `ReputationStatusBarMixin:Update` (lua:77-169) reads:
//! - `watchedFactionData = C_Reputation.GetWatchedFactionData()`
//!   (early-returns on nil or `factionID == 0`)
//! - `factionID`, `reaction`, `currentReactionThreshold`,
//!   `nextReactionThreshold`, `currentStanding`, `name` from that table
//! - `C_Reputation.IsFactionParagonForCurrentPlayer(factionID)` —
//!   gates the paragon arm at lua:99-111
//! - `C_Reputation.IsMajorFaction(factionID)` — gates the major-faction
//!   arm at lua:112-116, and acts as a `overrideUseBlueBar` override
//!   inside the paragon arm at lua:109-111
//! - `C_MajorFactions.GetMajorFactionData(factionID)` —
//!   `renownLevelThreshold`, `renownLevel` for the major-faction arm
//! - `C_MajorFactions.GetRenownLevels(factionID)` —
//!   `[#].level` for `GetMaxLevel` (lua:62-65)
//! - `C_GossipInfo.GetFriendshipReputation(factionID)` —
//!   `friendshipFactionID` first-time only (lua:91); current sim seeds
//!   `friendshipFactionID = 0` so the friendship arm at lua:117-127
//!   is unreachable from the test harness
//! - `C_Reputation.IsAccountWideReputation(factionID)` — appends a
//!   suffix label at lua:147-149
//! - `C_Reputation.GetFactionParagonInfo(factionID)` —
//!   `(currentValue, threshold, _, hasRewardPending)` for the paragon
//!   arm at lua:100
//!
//! After the branch dispatch, lua:135-141 normalises `minBar = 0`,
//! `maxBar -= minBar`, `value -= minBar`, then lua:143
//! `self:SetBarValues(value, minBar, maxBar, level, maxLevel)` routes
//! through `StatusTrackingBarMixin:SetBarValues`
//! (`Shared/StatusTrackingBar.lua:32-39`). The `supportsAnimation`
//! KeyValue is set on the inner `<StatusBar parentKey="StatusBar">`
//! (`StatusTrackingBarTemplate.xml:11`), NOT on the outer bar, so
//! `self.supportsAnimation` (outer bar) is nil and SetBarValues takes
//! the else branch at lua:36-37: a direct
//! `self.StatusBar:SetMinMaxValues(minBar, maxBar)` +
//! `self.StatusBar:SetValue(currentValue)`. The `level`/`maxLevel`
//! arguments are computed but discarded by this dispatch path.
//!
//! Finally lua:159 routes through
//! `ReputationStatusBarMixin:UpdateBarTextures(reactionLevel, overrideUseBlueBar)`
//! (`Mainline/ReputationBarOverrides.lua:41-47`) which picks
//! `barAtlases[reactionLevel]` (lua:3-13) for the standard arm and
//! `blueBarAtlas` for the Renown arm.
//!
//! ## Watched faction wiring
//!
//! `C_Reputation.GetWatchedFactionData` (`faction_probes.rs:314-321`)
//! reads `reputation_data::watched_faction()`
//! (`reputation_data.rs:213-217`), hard-coded to `Council of Dornogal`
//! (factionID 2590, reaction HONORED=6, currentStanding=8200,
//! top_value=12000). Faction 2590 is also seeded into
//! `state.major_factions` (`game_data.rs:1040-1088`, renown_level=1,
//! renown_level_threshold=2500) and
//! `state.major_faction_renown_levels` (1..=20). The test therefore
//! exercises the **major-faction branch** out of the box; to drive the
//! standard-rep branch the test mutates
//! `state.major_factions.remove(&2590)` and to drive the paragon
//! branch it inserts `state.faction_paragon[2590]`.
//!
//! ## Why call `bar:Update()` directly instead of firing
//! `UPDATE_FACTION` end-to-end
//!
//! `ReputationStatusBarMixin:OnLoad` (lua:171-173) only registers
//! `CVAR_UPDATE`, NOT `UPDATE_FACTION`. The reputation bar's actual
//! refresh path is via `StatusTrackingManagerMixin:OnEvent`
//! (`Shared/StatusTrackingManager.lua:36-48`) which routes
//! `UPDATE_FACTION` (and friends) through `barContainer:UpdateShownBarAll()`
//! (lua:344-349) → `shownBar:UpdateAll()` → `Update`. But that path
//! requires the bar to be the `shownBar`, which depends on the
//! container's animation state machine (`SetShownBar` →
//! `ApplyPendingBarToShow` → `newBar:UpdateAll()`). To keep the test
//! deterministic and isolate the Update body's branch dispatch from
//! the show/animation state machine, we call `bar:Update()` directly.
//! The registration → dispatch path is pinned by `surface_events.rs`
//! (StatusTrackingManager registers UPDATE_FACTION /
//! MAJOR_FACTION_RENOWN_LEVEL_CHANGED).
//!
//! ## Why the major-faction arm uses currentStanding for `value`
//!
//! Re-reading lua:98-116: the initial assignment
//! `value = watchedFactionData.currentStanding` happens BEFORE the
//! if-elseif chain, and the `elseif IsMajorFaction(factionID)` arm
//! reassigns only `minBar/maxBar/level/overrideUseBlueBar`. So in the
//! major-faction branch, `bar.value = watchedFactionData.currentStanding`
//! and `bar.max = renownLevelThreshold`. With WATCHED_CURRENT_STANDING
//! (8200) > MF_RENOWN_THRESHOLD (8000), `bar.value > bar.max` is the
//! correct contract — the test pins this overflow as evidence that
//! the major-faction arm did NOT incorrectly reset `value` to
//! `renownReputationEarned` or similar.
//!
//! ## Observations
//!
//! 1. After `with_blizzard_addon_startup_shape(&[Blizzard_ActionBar])`,
//!    the chain
//!    `StatusTrackingBarManager.MainStatusTrackingBarContainer.bars[BarsEnum.Reputation]`
//!    resolves with `.StatusBar` and the bar instance is a
//!    `ReputationStatusBarTemplate` frame
//!    (`Shared/ReputationBar.xml:3-11`).
//! 2. After mutating `state.major_factions.remove(&2590)` and calling
//!    `bar:Update()`: `bar.factionID == 2590`, `bar.value == 8200`,
//!    `bar.max == 12000`, `StatusBar:GetValue() == 8200`,
//!    `GetMinMaxValues() == (0, 12000)` (standard rep branch).
//! 3. After re-inserting `state.major_factions[2590]` with
//!    `renown_level=15`, `renown_level_threshold=8000` and calling
//!    `bar:Update()`: `bar.factionID == 2590`, `bar.value == 8200`
//!    (currentStanding, NOT reset by the major-faction arm),
//!    `bar.max == 8000` (renownLevelThreshold),
//!    `StatusBar:GetMinMaxValues() == (0, 8000)`,
//!    `StatusBar:GetValue() == 8000` (clamped to max — the unclamped
//!    8200 only lives on `bar.value`, the outer Lua property).
//! 4. After clearing major faction, seeding
//!    `state.faction_paragon[2590]` with `current_value=12345`,
//!    `threshold=10000`, `has_reward_pending=false`, and calling
//!    `bar:Update()`: `bar.value == 12345 % 10000 == 2345`,
//!    `bar.max == 10000`, `StatusBar:GetMinMaxValues() == (0, 10000)`.
//!
//! ## Regression candidates the assertions catch
//!
//! - `Update` short-circuits on `watchedFactionData == nil` → all
//!   post-Update probes fail because `bar.value`/`bar.max` are nil
//! - The `IsFactionParagonForCurrentPlayer → IsMajorFaction → else`
//!   dispatch order at lua:99-128 inverts → branch values cross-pollute
//! - `C_Reputation.IsMajorFaction` regresses to read the wrong state
//!   field → standard-branch test fails (still sees major-faction max)
//! - `C_MajorFactions.GetMajorFactionData(factionID).renownLevelThreshold`
//!   regresses → major-faction observation 3's `bar.max` mismatches
//! - `C_Reputation.GetFactionParagonInfo` regresses to drop
//!   `currentValue` / `threshold` → paragon observation 4's
//!   `bar.value % threshold` mismatches
//! - `SetBarValues` else-branch (`SetMinMaxValues`+`SetValue`) regresses
//!   → `StatusBar:GetValue()`/`GetMinMaxValues()` stay at zero while
//!   `bar.value`/`bar.max` (Lua state on the bar itself) still track

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::{FactionParagonInfo, MajorFactionData, RenownLevelInfo, WowLuaEnv};

const ROOT: &str = "Blizzard_ActionBar";

const REP_BAR_LUA: &str = "StatusTrackingBarManager.MainStatusTrackingBarContainer\
    .bars[StatusTrackingBarInfo.BarsEnum.Reputation]";

const WATCHED_FACTION_ID: i64 = 2590;
const WATCHED_CURRENT_STANDING: f64 = 8200.0;
const WATCHED_NEXT_THRESHOLD: f64 = 12000.0;

const MF_RENOWN_LEVEL: i32 = 15;
const MF_RENOWN_THRESHOLD: i32 = 8000;
const MF_RENOWN_LEVELS_MAX: i32 = 30;

const PARAGON_CURRENT: i32 = 12_345;
const PARAGON_THRESHOLD: i32 = 10_000;
const PARAGON_VALUE_MOD: f64 = 2_345.0;

#[test]
fn reputation_bar_renders_standard_paragon_and_renown_branches_from_watched_faction_data() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_cold_globals_resolve(env);
        drive_standard_branch(env);
        drive_major_faction_branch(env);
        drive_paragon_branch(env);
    });
    }
}

fn assert_cold_globals_resolve(env: &WowLuaEnv) {
    let cold_globals_exist: bool = env
        .eval(
            r#"
            local manager = StatusTrackingBarManager
            local container = manager and manager.MainStatusTrackingBarContainer
            local idx = StatusTrackingBarInfo and StatusTrackingBarInfo.BarsEnum
                and StatusTrackingBarInfo.BarsEnum.Reputation
            local bar = container and idx and container.bars and container.bars[idx]
            return manager ~= nil and container ~= nil and idx == 1
                and bar ~= nil and bar.StatusBar ~= nil
            "#,
        )
        .expect("reputation bar global existence probe must run cleanly");
    assert!(
        cold_globals_exist,
        "Chain StatusTrackingBarManager.MainStatusTrackingBarContainer.\
         bars[BarsEnum.Reputation].StatusBar must resolve after `{ROOT}` \
         load; nil reading means TOC/InitializeBars/parentKey regression — \
         see header observation 1."
    );

    let watched_id_present: bool = env
        .eval(
            "local d = C_Reputation.GetWatchedFactionData() \
             return d ~= nil and d.factionID == 2590",
        )
        .expect("watched faction probe must run cleanly");
    assert!(
        watched_id_present,
        "Cold `C_Reputation.GetWatchedFactionData().factionID` must be \
         {WATCHED_FACTION_ID} (Council of Dornogal, hardcoded by \
         `reputation_data.rs::watched_faction`); other reading means \
         the faction-list seed regressed."
    );
}

fn drive_standard_branch(env: &WowLuaEnv) {
    remove_major_faction_seed(env);
    update_bar(env);

    assert_bar_faction_id(env, "standard", WATCHED_FACTION_ID as f64);
    assert_bar_value_and_max(
        env,
        "standard",
        WATCHED_CURRENT_STANDING,
        WATCHED_NEXT_THRESHOLD,
    );
    assert_status_bar_min_max(env, "standard", 0.0, WATCHED_NEXT_THRESHOLD);
    assert_status_bar_value(env, "standard", WATCHED_CURRENT_STANDING);
}

fn drive_major_faction_branch(env: &WowLuaEnv) {
    seed_major_faction(env);
    update_bar(env);

    assert_bar_value_and_max(
        env,
        "major-faction",
        WATCHED_CURRENT_STANDING,
        MF_RENOWN_THRESHOLD as f64,
    );
    assert_status_bar_min_max(env, "major-faction", 0.0, MF_RENOWN_THRESHOLD as f64);
    assert_status_bar_value(env, "major-faction", MF_RENOWN_THRESHOLD as f64);
}

fn drive_paragon_branch(env: &WowLuaEnv) {
    remove_major_faction_seed(env);
    seed_paragon(env);
    update_bar(env);

    assert_bar_value_and_max(env, "paragon", PARAGON_VALUE_MOD, PARAGON_THRESHOLD as f64);
    assert_status_bar_min_max(env, "paragon", 0.0, PARAGON_THRESHOLD as f64);
    assert_status_bar_value(env, "paragon", PARAGON_VALUE_MOD);
}

fn update_bar(env: &WowLuaEnv) {
    env.eval::<bool>(&format!("{REP_BAR_LUA}:Update(); return true"))
        .expect("bar:Update() must run cleanly");
}

fn remove_major_faction_seed(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.major_factions.remove(&WATCHED_FACTION_ID);
    state
        .major_faction_renown_levels
        .remove(&WATCHED_FACTION_ID);
}

fn seed_major_faction(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.major_factions.insert(
        WATCHED_FACTION_ID,
        MajorFactionData {
            faction_id: WATCHED_FACTION_ID,
            name: "Council of Dornogal".to_string(),
            expansion_filter: 11,
            max_level: MF_RENOWN_LEVELS_MAX,
            renown_level: MF_RENOWN_LEVEL,
            renown_reputation_earned: 0,
            renown_level_threshold: MF_RENOWN_THRESHOLD,
            ui_priority: 1,
            is_unlocked: true,
            unlock_description: None,
            celebration_sound_kit: 0,
            renown_fanfare_sound_kit_id: 0,
            texture_kit: "councilofdornogal".to_string(),
            faction_font_color: (0.96, 0.78, 0.40),
        },
    );
    state.major_faction_renown_levels.insert(
        WATCHED_FACTION_ID,
        (1..=MF_RENOWN_LEVELS_MAX)
            .map(|level| RenownLevelInfo {
                faction_id: WATCHED_FACTION_ID,
                level,
                locked: false,
                is_milestone: false,
                is_capstone: level == MF_RENOWN_LEVELS_MAX,
            })
            .collect(),
    );
}

fn seed_paragon(env: &WowLuaEnv) {
    env.state().borrow_mut().faction_paragon.insert(
        WATCHED_FACTION_ID,
        FactionParagonInfo {
            current_value: PARAGON_CURRENT,
            threshold: PARAGON_THRESHOLD,
            reward_quest_id: 0,
            has_reward_pending: false,
            too_low_level_for_paragon: false,
            paragon_storage_level: 2,
        },
    );
}

fn assert_bar_value_and_max(env: &WowLuaEnv, branch: &str, expected_value: f64, expected_max: f64) {
    let bar_value: f64 = env
        .eval(&format!("return {REP_BAR_LUA}.value"))
        .unwrap_or_else(|_| panic!("{branch}-branch bar.value probe must run cleanly"));
    assert_eq!(
        bar_value, expected_value,
        "{branch}-branch `bar.value` must equal {expected_value} (set at \
         ReputationBar.lua:162). Got {bar_value}.",
    );

    let bar_max: f64 = env
        .eval(&format!("return {REP_BAR_LUA}.max"))
        .unwrap_or_else(|_| panic!("{branch}-branch bar.max probe must run cleanly"));
    assert_eq!(
        bar_max, expected_max,
        "{branch}-branch `bar.max` must equal {expected_max} (set at \
         ReputationBar.lua:163). Got {bar_max}.",
    );
}

fn assert_status_bar_min_max(env: &WowLuaEnv, branch: &str, expected_min: f64, expected_max: f64) {
    let min: f64 = env
        .eval(&format!(
            "local mn, _ = {REP_BAR_LUA}.StatusBar:GetMinMaxValues() return mn"
        ))
        .unwrap_or_else(|_| panic!("{branch}-branch StatusBar min probe must run cleanly"));
    let max: f64 = env
        .eval(&format!(
            "local _, mx = {REP_BAR_LUA}.StatusBar:GetMinMaxValues() return mx"
        ))
        .unwrap_or_else(|_| panic!("{branch}-branch StatusBar max probe must run cleanly"));
    assert_eq!(
        (min, max),
        (expected_min, expected_max),
        "{branch}-branch `StatusBar:GetMinMaxValues()` must equal \
         ({expected_min}, {expected_max}); the hidden-bar instant-flush \
         branch in SetAnimatedValues drives this. Got ({min}, {max}).",
    );
}

fn assert_status_bar_value(env: &WowLuaEnv, branch: &str, expected: f64) {
    let value: f64 = env
        .eval(&format!("return {REP_BAR_LUA}.StatusBar:GetValue()"))
        .unwrap_or_else(|_| panic!("{branch}-branch StatusBar:GetValue probe must run cleanly"));
    assert_eq!(
        value, expected,
        "{branch}-branch `StatusBar:GetValue()` must equal {expected} \
         (SetBarValues → SetAnimatedValues → ProcessChangesInstantly → \
         SetValue when the bar is hidden). Got {value}.",
    );
}

fn assert_bar_faction_id(env: &WowLuaEnv, branch: &str, expected: f64) {
    let bar_faction_id: f64 = env
        .eval(&format!("return {REP_BAR_LUA}.factionID"))
        .unwrap_or_else(|_| panic!("{branch}-branch bar.factionID probe must run cleanly"));
    assert_eq!(
        bar_faction_id, expected,
        "{branch}-branch `bar.factionID` must equal {expected} \
         (set at ReputationBar.lua:89 from watchedFactionData.factionID). \
         Got {bar_faction_id}.",
    );
}
