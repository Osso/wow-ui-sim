//! `C_MythicPlus` probe surface backed by `SimState.mythic_plus`.
//!
//! Migrates 11 entries off the namespace stub tables:
//!
//! - `GetCurrentAffixes()` — returns array of `{id, seasonID}` tables.
//! - `GetCurrentSeason()` — returns the current season id number.
//! - `GetRunHistory(includePrev, includeIncomplete, currentSeasonOnly)` —
//!   returns array of `MythicPlusRunInfo` tables.
//! - `GetSeasonBestAffixScoreInfoForMap(mapID)` — returns affixScores
//!   array + bestOverAllScore, or nothing if no data.
//! - `GetWeeklyChestRewardLevel()` — returns four zeros when no run done.
//! - `GetOwnedKeystoneLevel()` — returns owned keystone level or nothing.
//! - `GetWeeklyBestForMap(mapChallengeModeID)` — returns
//!   durationSec, level, completionDate=nil, fraction=0, affixIDs={},
//!   members={}, or nothing if no data.
//! - `IsMythicPlusActive()` — returns the `is_active` bool.
//! - `IsWeeklyRewardAvailable()` — returns the `is_weekly_reward_available` bool.
//! - `RequestCurrentAffixes()` / `RequestMapInfo()` / `RequestRewards()` — no-ops.
//! - `C_ChallengeMode.GetMapTable()` / `GetMapUIInfo()` — seeded Mists-era
//!   challenge dungeon rows for the legacy ChallengesFrame.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_api::state::MythicPlusWeeklyBest;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const AFFIX_NAMES: &[(i32, &str)] = &[
    (2, "Skittish"),
    (3, "Volcanic"),
    (4, "Necrotic"),
    (5, "Teeming"),
    (6, "Raging"),
    (7, "Bolstering"),
    (8, "Sanguine"),
    (9, "Tyrannical"),
    (10, "Fortified"),
    (11, "Bursting"),
    (12, "Grievous"),
    (13, "Explosive"),
    (14, "Quaking"),
    (120, "Awakened"),
    (121, "Prideful"),
    (122, "Inspiring"),
    (123, "Spiteful"),
    (124, "Storming"),
    (147, "Xal'atath's Guile"),
    (148, "Xal'atath's Bargain: Ascendant"),
    (152, "Challenger's Peril"),
    (158, "Xal'atath's Bargain: Voidbound"),
    (159, "Xal'atath's Bargain: Oblivion"),
    (160, "Xal'atath's Bargain: Devour"),
    (162, "Xal'atath's Bargain: Pulsar"),
];

pub(super) fn register_mythic_plus_surface(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetAffixInfo", get_affix_info)?;
    let ns = ensure_namespace(state, "C_MythicPlus")?;
    register_mythic_plus_probe_handlers(state, ns)?;
    register_mythic_plus_status_handlers(state, ns)?;
    register_mythic_plus_request_handlers(state, ns)?;
    register_challenge_mode_surface(state)?;
    Ok(())
}

fn register_mythic_plus_probe_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetCurrentAffixes", get_current_affixes)?;
    table_set_rust_fn_static(state, ns, "GetCurrentSeason", get_current_season)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    table_set_rust_fn_static(state, ns, "GetRunHistory", get_run_history)?;
    #[cfg(feature = "retail-12-0-7")]
    crate::c_api::c_mythic_plus_calendar::register(state, ns)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetSeasonBestAffixScoreInfoForMap",
        get_season_best_affix_score_info_for_map,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetWeeklyChestRewardLevel",
        get_weekly_chest_reward_level,
    )?;
    table_set_rust_fn_static(state, ns, "GetOwnedKeystoneLevel", get_owned_keystone_level)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    table_set_rust_fn_static(state, ns, "GetWeeklyBestForMap", get_weekly_best_for_map)?;
    Ok(())
}

fn register_mythic_plus_status_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "IsMythicPlusActive", is_mythic_plus_active)?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsWeeklyRewardAvailable",
        is_weekly_reward_available,
    )?;
    Ok(())
}

fn register_mythic_plus_request_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "RequestCurrentAffixes", noop)?;
    table_set_rust_fn_static(state, ns, "RequestMapInfo", noop)?;
    table_set_rust_fn_static(state, ns, "RequestRewards", noop)?;
    Ok(())
}

fn register_challenge_mode_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_ChallengeMode")?;
    register_challenge_map_handlers(state, ns)?;
    register_challenge_reward_handlers(state, ns)?;
    register_challenge_request_handlers(state, ns)?;
    Ok(())
}

fn register_challenge_map_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetMapTable", challenge_get_map_table)?;
    table_set_rust_fn_static(state, ns, "GetMapUIInfo", challenge_get_map_ui_info)?;
    table_set_rust_fn_static(state, ns, "GetAffixInfo", get_affix_info)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetChallengeBestTime",
        challenge_get_challenge_best_time,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetChallengeModeMapTimes",
        challenge_get_challenge_mode_map_times,
    )?;
    table_set_rust_fn_static(state, ns, "GetNumMedals", challenge_get_num_medals)?;
    Ok(())
}

fn register_challenge_reward_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        ns,
        "GetChallengeMapRewardInfo",
        challenge_get_challenge_map_reward_info,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetNumChallengeMapRewards",
        challenge_get_num_challenge_map_rewards,
    )?;
    Ok(())
}

fn register_challenge_request_handlers(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        ns,
        "GetLeaverPenaltyWarningTimeLeft",
        get_leaver_penalty_warning_time_left,
    )?;
    table_set_rust_fn_static(state, ns, "RequestLeaders", noop)?;
    Ok(())
}

fn challenge_get_map_table(state: &mut LuaState) -> LuaResult<u32> {
    let maps = create_table(state);
    set_table_array(state, maps, 1, Val::Num(2.0));
    state.push(maps);
    Ok(1)
}

fn challenge_get_map_ui_info(state: &mut LuaState) -> LuaResult<u32> {
    let challenge_id = i32::from_stack(state, 1)?;
    let (name, map_id) = challenge_map_ui_info(challenge_id);
    let name = create_string(state, name);
    state.push(name);
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Num(map_id as f64));
    Ok(6)
}

fn challenge_map_ui_info(challenge_id: i32) -> (&'static str, i32) {
    match challenge_id {
        2 => ("Temple of the Jade Serpent", 429),
        _ => ("Unknown Challenge", 0),
    }
}

fn challenge_get_challenge_best_time(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(2)
}

fn challenge_get_challenge_mode_map_times(state: &mut LuaState) -> LuaResult<u32> {
    let times = create_table(state);
    set_table_array(state, times, 1, Val::Num(45.0 * 60.0));
    set_table_array(state, times, 2, Val::Num(30.0 * 60.0));
    set_table_array(state, times, 3, Val::Num(15.0 * 60.0));
    state.push(times);
    Ok(1)
}

fn challenge_get_challenge_map_reward_info(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn challenge_get_num_challenge_map_rewards(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn challenge_get_num_medals(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(3.0));
    Ok(1)
}

fn noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_affix_info(state: &mut LuaState) -> LuaResult<u32> {
    let affix_id = i32::from_stack(state, 1)?;
    let Some(name) = affix_name(affix_id) else {
        return Ok(0);
    };

    let name = create_string(state, name);
    let description = create_string(state, "");
    state.push(name);
    state.push(description);
    state.push(Val::Num(affix_icon(affix_id) as f64));
    Ok(3)
}

fn affix_name(affix_id: i32) -> Option<&'static str> {
    AFFIX_NAMES
        .iter()
        .find_map(|(id, name)| (*id == affix_id).then_some(*name))
}

fn affix_icon(affix_id: i32) -> i32 {
    136243 + affix_id
}

fn get_current_affixes(state: &mut LuaState) -> LuaResult<u32> {
    let affixes = borrow_state(state)?.mythic_plus.current_affixes.clone();
    let array = create_table(state);
    for (i, affix) in affixes.into_iter().enumerate() {
        let entry = create_table(state);
        table_set(state, entry, "id", Val::Num(affix.id as f64));
        table_set(state, entry, "seasonID", Val::Num(affix.season_id as f64));
        set_table_array(state, array, i as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn get_current_season(state: &mut LuaState) -> LuaResult<u32> {
    let season = borrow_state(state)?.mythic_plus.current_season;
    state.push(Val::Num(season as f64));
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn get_run_history(state: &mut LuaState) -> LuaResult<u32> {
    // Args: includePreviousWeeks, includeIncompleteRuns, currentSeasonOnly
    // We ignore the filter args and return all seeded runs.
    let runs = borrow_state(state)?.mythic_plus.run_history.clone();
    let array = create_table(state);
    for (i, run) in runs.into_iter().enumerate() {
        let entry = create_table(state);
        table_set(
            state,
            entry,
            "mapChallengeModeID",
            Val::Num(run.map_challenge_mode_id as f64),
        );
        table_set(state, entry, "level", Val::Num(run.level as f64));
        table_set(state, entry, "completed", Val::Bool(run.completed));
        table_set(state, entry, "season", Val::Num(run.season as f64));
        table_set(state, entry, "runScore", Val::Num(run.run_score));
        table_set(state, entry, "thisWeek", Val::Bool(run.this_week));
        table_set(
            state,
            entry,
            "durationSec",
            Val::Num(run.duration_sec as f64),
        );
        // completionDate omitted (nil by default in Lua tables)
        set_table_array(state, array, i as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn get_season_best_affix_score_info_for_map(state: &mut LuaState) -> LuaResult<u32> {
    // No season-best data seeded by default — return nothing
    // (mayreturnnothing per API spec).
    let _map_id = i32::from_stack(state, 1)?;
    Ok(0)
}

fn get_weekly_chest_reward_level(state: &mut LuaState) -> LuaResult<u32> {
    // currentWeekBestLevel, weeklyRewardLevel, nextDifficultyWeeklyRewardLevel, nextBestLevel
    // Default all zeros — no run completed this week.
    let _ = state;
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(4)
}

fn get_owned_keystone_level(state: &mut LuaState) -> LuaResult<u32> {
    let level = borrow_state(state)?.mythic_plus.owned_keystone_level;
    if level == 0 {
        // mayreturnnothing: return nothing when player has no key.
        return Ok(0);
    }
    state.push(Val::Num(level as f64));
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn get_weekly_best_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let best: Option<MythicPlusWeeklyBest> = borrow_state(state)?
        .mythic_plus
        .weekly_best_per_map
        .get(&map_id)
        .cloned();
    let Some(b) = best else {
        // mayreturnnothing: no data for this map.
        return Ok(0);
    };
    state.push(Val::Num(b.duration_sec as f64));
    state.push(Val::Num(b.level as f64));
    state.push(Val::Nil); // completionDate nilable
    state.push(Val::Num(0.0)); // fraction
    let affix_ids = create_table(state);
    state.push(affix_ids);
    let members = create_table(state);
    state.push(members);
    Ok(6)
}

fn is_mythic_plus_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.mythic_plus.is_active;
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_weekly_reward_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.mythic_plus.is_weekly_reward_available;
    state.push(Val::Bool(available));
    Ok(1)
}

fn get_leaver_penalty_warning_time_left(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}
