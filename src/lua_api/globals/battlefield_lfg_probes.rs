//! Battlefield / LFG probe globals.
//!
//! Covers battlefield queue probes plus the LFD state used by Group Finder and
//! QueueStatusFrame. Most functions return seeded, inert server-state shapes
//! until the simulator models real queue matching.
//!
//! - `GetBattlefieldStatus`, `GetNumBattlegroundEntries`, and
//!   `GetWorldPVPQueueStatus` reflect `SimState.battlefield_queue`.
//! - `GetBattlefieldInstanceRunTime()` → 0; entry timing is not modeled.
//! - `GetLFGDungeonInfo(dungeonID)`    → 21 values from `SimState.lfd_dungeons`.
//! - `GetLFGMode(category)`            → `("queued", nil)` after `JoinLFG`,
//!   `("proposal", nil)` after the queue-pop timer fires, otherwise `(nil, nil)`.
//! - `GetLFGDungeonNumEncounters(id)`  → `(numEncounters, numCompleted)`.
//! - `GetLFGDungeonEncounterInfo(id, idx)` → `(name, texture, isKilled)`.
//! - `GetLFDChoiceOrder()`             → array of dungeon IDs in seeded order.
//! - `GetNumRandomDungeons()`          → count of random-flagged LFD entries.
//! - `GetLFGRandomDungeonInfo(index)`  → `(id, name)` for 1-based random index.
//! - `GetRandomDungeonBestChoice()`    → first random dungeon id, or nil.
//! - `GetNumRFDungeons()`              → count of raid-finder entries.
//! - `GetRFDungeonInfo(index)`         → id plus dungeon info for a RF entry.
//! - `GetLFDLockPlayerCount()`         → 0. The sim has no LFD locks.
//! - `GetLFDLockInfo(dungeonID, idx)`  → all-nil. No lock data without queue.
//! - `GetLFDRoleLockInfo(id, roleID)`  → empty table. No role restrictions.
//! - `GetLFDChoiceCollapseState(t?)` and `GetLFDChoiceEnabledState(t?)` feed
//!   `LFGDungeonList_Setup`.
//! - `ClearAllLFGDungeons(category)`   → clears selected and active LFG mode.
//! - `SetLFGDungeon(category, id)`      → validates and records selected ids.
//! - `JoinLFG(category)`               → marks the category queued and schedules
//!   a proposal pop.
//! - `LeaveLFG(category)`              → clears queued/proposal state.
//! - `GetLFGProposal()`                → active proposal tuple or inert shape.
//! - `GetLFGProposalMember(index)`     → active proposal member tuple.
//! - `GetLFGProposalEncounter(index)`  → active proposal boss tuple.
//! - `GetLFGInfoServer(category, id?)` → queued server-info tuple consumed by
//!   Blizzard's Lua `GetLFGMode`.
//! - `GetLFGQueuedList(category, t?)`   → selected queue ids by category.
//! - `GetLFGQueueStats(category, id?)`  → queue-status display tuple.
//! - `GetLFGLockList()`                → empty table. No server locks.
//! - `GetBestRFChoice()`               → first seeded raid-finder dungeon, or nil.
//! - `GetRandomScenarioBestChoice()`   → nil. No scenario state. Same path.
//! - `GetLFGDungeonRewards(id)`        → 7 zeros + nil spellID. Called from
//!   `LFDQueueFrameRandom_UpdateFrame` when a random dungeon is selected.
//! - `GetLFGDungeonRewardCapInfo(id)`  → 11 nils. No currency cap data.
//! - `DungeonAppearsInRandomLFD(id)`   → LFD category for seeded dungeon ids.
//! - `IsLFGDungeonJoinable(dungeonID)` → `(isAvailableForAll, isAvailableForPlayer,
//!   hideIfNotJoinable, totalGroupSizeRequired)` from `lfd_dungeons` + `player.level`.

mod queue;

use crate::lua_api::methods::{borrow_state, create_string, create_table};
use crate::lua_api::state::BattlefieldStatus;
use crate::lua_api::state_types::LfdDungeonInfo;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_bool(state: &LuaState, index: i32) -> bool {
    matches!(stack_val(state, index), Val::Bool(true))
}

fn is_default_lfd_enabled(d: &LfdDungeonInfo, player_level: i32) -> bool {
    d.dungeon_id > 0
        && !d.is_random
        && !d.is_follower_dungeon
        && player_level >= d.min_level
        && player_level <= d.max_level
}

/// INFERRED R88ei (native evidence pending): the highest supplied active queue index.
fn get_max_battlefield_id(state: &mut LuaState) -> LuaResult<u32> {
    let max_id = {
        let sim = borrow_state(state)?;
        if sim.battlefield_queue.status == BattlefieldStatus::None {
            0
        } else {
            sim.battlefield_queue.index.max(0)
        }
    };
    state.push(Val::Num(max_id as f64));
    Ok(1)
}

/// `GetBattlefieldStatus(index)` — retail returns 9 values. The sim
/// only models a single active queue, so any index other than the
/// one stored in `battlefield_queue` reports `"none"` with default
/// zeros.
fn get_battlefield_status(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(1);
    let (status, map_name) = {
        let sim = borrow_state(state)?;
        if sim.battlefield_queue.status == BattlefieldStatus::None
            || sim.battlefield_queue.index != index
        {
            (BattlefieldStatus::None, String::new())
        } else {
            (
                sim.battlefield_queue.status,
                sim.battlefield_queue.name.clone(),
            )
        }
    };
    let status_val = create_string(state, status.as_wow_str());
    let map_val = create_string(state, &map_name);
    state.push(status_val); // 1: status
    state.push(map_val); // 2: mapName
    state.push(Val::Num(0.0)); // 3: instanceID
    state.push(Val::Num(0.0)); // 4: levelRangeMin
    state.push(Val::Num(0.0)); // 5: levelRangeMax
    state.push(Val::Num(0.0)); // 6: teamSize (0 for BGs)
    state.push(Val::Bool(false)); // 7: registeredMatch
    state.push(Val::Bool(true)); // 8: eligibleInQueue (matches retail default)
    state.push(Val::Bool(false)); // 9: waitingOnOtherActivity
    Ok(9)
}

/// `GetBattlefieldInstanceRunTime()` — retail returns ms since the
/// player entered a battleground instance. The sim doesn't track that,
/// so always 0.
fn get_battlefield_instance_run_time(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

/// `GetNumBattlegroundEntries()` — 1 while the queue holds any active
/// state, 0 otherwise.
fn get_num_battleground_entries(state: &mut LuaState) -> LuaResult<u32> {
    let count = if borrow_state(state)?.battlefield_queue.status == BattlefieldStatus::None {
        0.0
    } else {
        1.0
    };
    state.push(Val::Num(count));
    Ok(1)
}

/// `GetWorldPVPQueueStatus(index)` — retail returns 7 values. The sim
/// doesn't seed world PvP queues, so every index reports the inert queue
/// shape Blizzard UI uses for "not queued".
fn get_world_pvp_queue_status(state: &mut LuaState) -> LuaResult<u32> {
    let status = create_string(state, "none");
    let map_name = create_string(state, "");
    state.push(status); // 1: status
    state.push(map_name); // 2: mapName
    state.push(Val::Num(0.0)); // 3: queueID / battleID
    state.push(Val::Num(0.0)); // 4: expireTime
    state.push(Val::Num(0.0)); // 5: averageWaitTime
    state.push(Val::Num(0.0)); // 6: queuedTime
    state.push(Val::Bool(false)); // 7: suspended
    Ok(7)
}

fn push_dungeon_info(state: &mut LuaState, d: &LfdDungeonInfo) {
    let name = create_string(state, &d.name);
    let texture = create_string(state, &d.texture_filename);
    let description = create_string(state, &d.description);
    let map_name = create_string(state, &d.map_name);
    state.push(name); // 1: name
    state.push(Val::Num(d.type_id as f64)); // 2: typeID
    state.push(Val::Num(d.subtype_id as f64)); // 3: subtypeID
    state.push(Val::Num(d.min_level as f64)); // 4: minLevel
    state.push(Val::Num(d.max_level as f64)); // 5: maxLevel
    state.push(Val::Num(d.rec_level as f64)); // 6: recLevel
    state.push(Val::Num(d.min_rec_level as f64)); // 7: minRecLevel
    state.push(Val::Num(d.max_rec_level as f64)); // 8: maxRecLevel
    state.push(Val::Num(d.expansion_level as f64)); // 9: expansionLevel
    state.push(Val::Num(d.group_id as f64)); // 10: groupID
    state.push(texture); // 11: textureFilename
    state.push(Val::Num(d.difficulty as f64)); // 12: difficulty
    state.push(Val::Num(d.max_players as f64)); // 13: maxPlayers
    state.push(description); // 14: description
    state.push(Val::Bool(d.is_holiday)); // 15: isHoliday
    state.push(Val::Num(0.0)); // 16: bonusRepAmount
    if d.min_players > 1 {
        state.push(Val::Num(d.min_players as f64)); // 17: minPlayers
    } else {
        state.push(Val::Nil); // 17: minPlayers
    }
    state.push(Val::Bool(false)); // 18: isTimewalker
    state.push(map_name); // 19: mapName
    state.push(Val::Num(d.min_gear as f64)); // 20: minGear
    state.push(Val::Bool(d.is_scaling_dungeon)); // 21: isScalingDungeon
}

/// `GetLFGDungeonInfo(dungeonID)` → 21 values from `lfd_dungeons`, or nil.
fn get_lfg_dungeon_info(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => {
            state.push(Val::Nil);
            return Ok(1);
        }
    };
    let dungeon = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .find(|d| d.dungeon_id == dungeon_id)
        .cloned();
    let Some(d) = dungeon else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_dungeon_info(state, &d);
    Ok(21)
}

/// `GetLFGMode(category[, queueID])` — retail returns `(mode, submode)`.
/// The sim tracks category-level queued state only.
fn get_lfg_mode(state: &mut LuaState) -> LuaResult<u32> {
    let category = stack_i32(state, 1).unwrap_or(0);
    let mode = {
        let sim = borrow_state(state)?;
        let has_proposal = sim
            .lfg_active_proposal
            .as_ref()
            .is_some_and(|proposal| proposal.category == category);
        if has_proposal {
            Some("proposal")
        } else if sim.lfg_active_categories.contains(&category) {
            Some("queued")
        } else {
            None
        }
    };
    match mode {
        Some(mode) => {
            let mode = create_string(state, mode);
            state.push(mode);
        }
        None => state.push(Val::Nil),
    }
    if mode == Some("proposal") {
        let submode = create_string(state, "unaccepted");
        state.push(submode);
    } else {
        state.push(Val::Nil);
    }
    Ok(2)
}

/// `GetLFGDungeonNumEncounters(dungeonID)` → `(numEncounters, numCompleted)`.
fn get_lfg_dungeon_num_encounters(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = stack_i32(state, 1).unwrap_or(0);
    let known = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .any(|d| d.dungeon_id == dungeon_id && dungeon_id > 0);
    if known {
        state.push(Val::Num(3.0)); // numEncounters
        state.push(Val::Num(0.0)); // numCompleted
    } else {
        state.push(Val::Num(0.0));
        state.push(Val::Num(0.0));
    }
    Ok(2)
}

fn get_lfg_random_cooldown_expiration(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_lfg_dungeon_encounter_info(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = stack_i32(state, 1).unwrap_or(0);
    let encounter_index = stack_i32(state, 2).unwrap_or(0);
    let known = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .any(|d| d.dungeon_id == dungeon_id && dungeon_id > 0);
    let encounter_in_range = (1..=3).any(|valid_index| valid_index == encounter_index);
    if !known || !encounter_in_range {
        state.push(Val::Nil);
        return Ok(1);
    }

    let name = format!("Encounter {encounter_index}");
    let name = create_string(state, &name);
    state.push(name);
    state.push(Val::Nil);
    state.push(Val::Bool(false));
    Ok(3)
}

/// `GetLFDChoiceOrder()` → array of dungeon IDs in seeded order.
fn get_lfd_choice_order(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .map(|d| d.dungeon_id)
        .collect::<Vec<_>>();
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, id) in ids.iter().enumerate() {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*id as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetNumRandomDungeons()` → count of lfd_dungeons where is_random=true.
fn get_num_random_dungeons(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .filter(|d| d.is_random)
        .count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

/// `GetLFGRandomDungeonInfo(index)` → `(id, name)` for 1-based random subset index.
fn get_lfg_random_dungeon_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(1) as usize;
    let found = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .filter(|d| d.is_random)
        .nth(index.saturating_sub(1))
        .map(|d| (d.dungeon_id, d.name.clone()));
    let Some((id, name)) = found else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(id as f64));
    let name_val = create_string(state, &name);
    state.push(name_val);
    Ok(2)
}

fn raid_finder_dungeons(
    sim: &crate::lua_api::state::SimState,
) -> impl Iterator<Item = &LfdDungeonInfo> {
    sim.lfd_dungeons.iter().filter(|d| d.max_players > 5)
}

/// `GetNumRFDungeons()` → count of raid-finder entries.
fn get_num_rf_dungeons(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        raid_finder_dungeons(&sim).count()
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

/// `GetRFDungeonInfo(index)` → `(id, name, typeID, ...)` for a 1-based RF
/// dropdown entry. It mirrors `GetLFGDungeonInfo` with the dungeon id prepended
/// and a nil mapID in slot 23, matching the fields RaidFinder.lua reads.
fn get_rf_dungeon_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(1) as usize;
    let dungeon = {
        let sim = borrow_state(state)?;
        raid_finder_dungeons(&sim)
            .nth(index.saturating_sub(1))
            .cloned()
    };
    let Some(dungeon) = dungeon else {
        state.push(Val::Nil);
        return Ok(1);
    };

    state.push(Val::Num(dungeon.dungeon_id as f64));
    push_dungeon_info(state, &dungeon);
    state.push(Val::Nil);
    Ok(23)
}

/// `GetRandomDungeonBestChoice()` → first random dungeon id, or nil.
fn get_random_dungeon_best_choice(state: &mut LuaState) -> LuaResult<u32> {
    let found = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .find(|d| d.is_random)
        .map(|d| d.dungeon_id);
    match found {
        Some(id) => {
            state.push(Val::Num(id as f64));
            Ok(1)
        }
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

/// `GetLFDLockPlayerCount()` → number of party members for which lock info
/// would be reported. The sim has no LFD locks, so 0.
fn get_lfd_lock_player_count(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

/// `GetLFDChoiceCollapseState(t?)` → table mapping header dungeonID → bool
/// for whether that header is collapsed in the LFD list. The sim has no
/// persisted UI state, so a fresh empty table is always returned (the
/// caller's optional argument is ignored — Lua-side `t = f(t)` simply
/// rebinds to the new table). Empty means all headers default to expanded.
fn get_lfd_choice_collapse_state(state: &mut LuaState) -> LuaResult<u32> {
    let result = create_table(state);
    state.push(result);
    Ok(1)
}

/// `GetLFDChoiceEnabledState(t?)` → table mapping dungeonID → checkbox
/// state. Fresh characters start with joinable non-follower specific
/// dungeons checked, matching the LFD panel's queueable default state.
fn get_lfd_choice_enabled_state(state: &mut LuaState) -> LuaResult<u32> {
    let entries = {
        let sim = borrow_state(state)?;
        let player_level = sim.player.level;
        sim.lfd_dungeons
            .iter()
            .filter_map(|d| {
                let enabled = sim
                    .lfd_enabled_dungeons
                    .get(&d.dungeon_id)
                    .copied()
                    .unwrap_or_else(|| is_default_lfd_enabled(d, player_level));
                enabled.then_some(d.dungeon_id)
            })
            .collect::<Vec<_>>()
    };
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for id in entries {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(Val::Num(id as f64), Val::Bool(true), &state.gc.string_arena);
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetLFGLockList()` → table mapping dungeonID → `{lfgID, reason}` for
/// locked dungeons. The sim has no server-side lock data, so this returns
/// an empty table. `LFGList_DefaultFilterFunction` returns `false` when
/// this list is nil — that's why an empty list (rather than nil) is
/// required for the dungeon list to populate.
fn get_lfg_lock_list(state: &mut LuaState) -> LuaResult<u32> {
    let result = create_table(state);
    state.push(result);
    Ok(1)
}

/// `GetBestRFChoice()` → first seeded raid-finder dungeon id, or nil.
/// Called from `RaidFinderFrame_OnEvent` when `LFG_LOCK_INFO_RECEIVED` fires.
fn get_best_rf_choice(state: &mut LuaState) -> LuaResult<u32> {
    let found = borrow_state(state).map(|sim| {
        raid_finder_dungeons(&sim)
            .next()
            .map(|dungeon| dungeon.dungeon_id)
    })?;
    match found {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

/// `GetRandomScenarioBestChoice()` → scenarioID or nil. The sim has no
/// scenario data. Called transitively from `RaidFinderFrame_OnEvent`
/// via `ScenarioFinder_Shared`.
fn get_random_scenario_best_choice(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

/// Keep catalog order and the supplied specific/random distinction. Other
/// profiles retain their existing empty scenario surface.
fn scenario_dungeons(
    sim: &crate::lua_api::state::SimState,
    random: bool,
) -> impl Iterator<Item = &LfdDungeonInfo> {
    const SCENARIO_SUBTYPE: i32 = 4; // LFG_SUBTYPEID_SCENARIO
    let modeled_profile = matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    );
    sim.lfd_dungeons.iter().filter(move |dungeon| {
        modeled_profile && dungeon.subtype_id == SCENARIO_SUBTYPE && dungeon.is_random == random
    })
}

fn get_scenarios_choice_order(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<_> = {
        let sim = borrow_state(state)?;
        scenario_dungeons(&sim, false)
            .map(|dungeon| dungeon.dungeon_id)
            .collect()
    };
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, id) in ids.into_iter().enumerate() {
            crate::lua_api::methods::table_set_num(
                state,
                table_ref,
                (index + 1) as f64,
                Val::Num(id as f64),
            );
        }
    }
    state.push(result);
    Ok(1)
}

fn get_num_random_scenarios(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        scenario_dungeons(&sim, true).count()
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_random_scenario_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1).and_then(|index| index.checked_sub(1)) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let dungeon = {
        let sim = borrow_state(state)?;
        scenario_dungeons(&sim, true).nth(index as usize).cloned()
    };
    let Some(dungeon) = dungeon else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let name = create_string(state, &dungeon.name);
    state.push(Val::Num(dungeon.dungeon_id as f64));
    state.push(name);
    state.push(Val::Num(dungeon.type_id as f64));
    state.push(Val::Num(dungeon.subtype_id as f64));
    state.push(Val::Num(dungeon.min_level as f64));
    state.push(Val::Num(dungeon.max_level as f64));
    Ok(6)
}

/// `GetLFGDungeonRewards(dungeonID)` → 7 numbers describing rewards. The
/// sim has no reward data, so all zeros (and nil spellID). Called from
/// the random-dungeon panel when a positive-id random dungeon is shown.
fn get_lfg_dungeon_rewards(state: &mut LuaState) -> LuaResult<u32> {
    for _ in 0..6 {
        state.push(Val::Num(0.0));
    }
    state.push(Val::Nil);
    Ok(7)
}

/// `GetLFGDungeonRewardCapInfo(dungeonID)` → 11 values describing the
/// currency cap that limits repeated rewards. The sim has no cap data, so
/// return the full nil shape; Blizzard exits early when currencyID is nil.
fn get_lfg_dungeon_reward_cap_info(state: &mut LuaState) -> LuaResult<u32> {
    for _ in 0..11 {
        state.push(Val::Nil);
    }
    Ok(11)
}

/// `DungeonAppearsInRandomLFD(dungeonID)` returns the LFG category id when
/// the dungeon is represented in the random/specific LFD pool. Blizzard's
/// Adventure Journal path uses this as the gate for selecting a dungeon by id.
fn dungeon_appears_in_random_lfd(state: &mut LuaState) -> LuaResult<u32> {
    let Some(dungeon_id) = stack_i32(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let appears = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .any(|d| d.dungeon_id == dungeon_id && d.dungeon_id > 0);
    if appears {
        state.push(Val::Num(1.0)); // LE_LFG_CATEGORY_LFD
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

/// `GetLFDLockInfo(dungeonID, partyIndex)` — retail returns 6 values
/// describing a player's lock on a dungeon. The sim has no locks, so
/// every call reports all-nil.
fn get_lfd_lock_info(state: &mut LuaState) -> LuaResult<u32> {
    for _ in 0..6 {
        state.push(Val::Nil);
    }
    Ok(6)
}

/// `GetLFDRoleLockInfo(dungeonID, roleID)` → array of `{reason_id,
/// sub_reason, reason_string}` rows. No role restrictions in the sim,
/// so an empty table is the inert shape Blizzard UI expects.
fn get_lfd_role_lock_info(state: &mut LuaState) -> LuaResult<u32> {
    let result = create_table(state);
    state.push(result);
    Ok(1)
}

/// `IsLFGDungeonJoinable(dungeonID)` →
/// `(isAvailableForAll, isAvailableForPlayer, hideIfNotJoinable, totalGroupSizeRequired)`.
///
/// Computed from `lfd_dungeons` + `player.level`. Unknown ids return
/// `(false, false, true, 0)` so callers that gate on `hideIfNotJoinable`
/// drop them. Headers (negative ids) participate the same way as dungeons —
/// `GetLFDChoiceOrder` includes them and the UI iterates the same level check.
fn is_lfg_dungeon_joinable(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => {
            state.push(Val::Bool(false));
            state.push(Val::Bool(false));
            state.push(Val::Bool(true));
            state.push(Val::Num(0.0));
            return Ok(4);
        }
    };
    let (available_for_all, available_for_player, total_group_size) = {
        let sim = borrow_state(state)?;
        let player_level = sim.player.level;
        match sim.lfd_dungeons.iter().find(|d| d.dungeon_id == dungeon_id) {
            Some(d) => {
                let in_range = player_level >= d.min_level && player_level <= d.max_level;
                (true, in_range, d.max_players)
            }
            None => (false, false, 0),
        }
    };
    state.push(Val::Bool(available_for_all));
    state.push(Val::Bool(available_for_player));
    state.push(Val::Bool(!available_for_all)); // hideIfNotJoinable: only true for unknown ids
    state.push(Val::Num(total_group_size as f64));
    Ok(4)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_battlefield_queue_globals(lua)?;
    register_lfg_globals(lua)?;
    Ok(())
}

fn register_lfg_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_lfd_info_globals(lua)?;
    register_lfd_state_globals(lua)?;
    Ok(())
}

fn register_battlefield_queue_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "GetMaxBattlefieldID", get_max_battlefield_id)?;
    }
    LuaApiMut::register_function(lua, "GetBattlefieldStatus", get_battlefield_status)?;
    LuaApiMut::register_function(
        lua,
        "GetBattlefieldInstanceRunTime",
        get_battlefield_instance_run_time,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetNumBattlegroundEntries",
        get_num_battleground_entries,
    )?;
    LuaApiMut::register_function(lua, "GetWorldPVPQueueStatus", get_world_pvp_queue_status)?;
    Ok(())
}

fn register_lfd_info_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_lfd_dungeon_info_globals(lua)?;
    register_lfd_choice_globals(lua)?;
    register_lfg_reward_globals(lua)?;
    LuaApiMut::register_function(lua, "IsLFGDungeonJoinable", is_lfg_dungeon_joinable)?;
    Ok(())
}

fn register_lfd_dungeon_info_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLFGDungeonInfo", get_lfg_dungeon_info)?;
    LuaApiMut::register_function(lua, "GetLFGMode", get_lfg_mode)?;
    LuaApiMut::register_function(
        lua,
        "GetLFGDungeonNumEncounters",
        get_lfg_dungeon_num_encounters,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetLFGRandomCooldownExpiration",
        get_lfg_random_cooldown_expiration,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetLFGDungeonEncounterInfo",
        get_lfg_dungeon_encounter_info,
    )?;
    Ok(())
}

fn register_lfd_choice_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLFDChoiceOrder", get_lfd_choice_order)?;
    LuaApiMut::register_function(lua, "GetNumRandomDungeons", get_num_random_dungeons)?;
    LuaApiMut::register_function(lua, "GetLFGRandomDungeonInfo", get_lfg_random_dungeon_info)?;
    LuaApiMut::register_function(lua, "GetNumRFDungeons", get_num_rf_dungeons)?;
    LuaApiMut::register_function(lua, "GetRFDungeonInfo", get_rf_dungeon_info)?;
    LuaApiMut::register_function(
        lua,
        "GetRandomDungeonBestChoice",
        get_random_dungeon_best_choice,
    )?;
    Ok(())
}

fn register_lfg_reward_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLFGDungeonRewards", get_lfg_dungeon_rewards)?;
    LuaApiMut::register_function(
        lua,
        "GetLFGDungeonRewardCapInfo",
        get_lfg_dungeon_reward_cap_info,
    )?;
    LuaApiMut::register_function(
        lua,
        "DungeonAppearsInRandomLFD",
        dungeon_appears_in_random_lfd,
    )?;
    Ok(())
}

fn register_lfd_state_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLFDLockPlayerCount", get_lfd_lock_player_count)?;
    LuaApiMut::register_function(lua, "GetLFDLockInfo", get_lfd_lock_info)?;
    LuaApiMut::register_function(lua, "GetLFDRoleLockInfo", get_lfd_role_lock_info)?;
    LuaApiMut::register_function(
        lua,
        "GetLFDChoiceCollapseState",
        get_lfd_choice_collapse_state,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetLFDChoiceEnabledState",
        get_lfd_choice_enabled_state,
    )?;
    register_lfg_queue_state_globals(lua)?;
    LuaApiMut::register_function(lua, "GetLFGRoles", queue::get_lfg_roles)?;
    LuaApiMut::register_function(lua, "SetLFGRoles", queue::set_lfg_roles)?;
    LuaApiMut::register_function(lua, "GetLFGLockList", get_lfg_lock_list)?;
    LuaApiMut::register_function(lua, "GetBestRFChoice", get_best_rf_choice)?;
    LuaApiMut::register_function(
        lua,
        "GetRandomScenarioBestChoice",
        get_random_scenario_best_choice,
    )?;
    LuaApiMut::register_function(lua, "GetScenariosChoiceOrder", get_scenarios_choice_order)?;
    LuaApiMut::register_function(lua, "GetNumRandomScenarios", get_num_random_scenarios)?;
    LuaApiMut::register_function(lua, "GetRandomScenarioInfo", get_random_scenario_info)?;
    Ok(())
}

fn register_lfg_queue_state_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "SetLFGDungeonEnabled", queue::set_lfg_dungeon_enabled)?;
    LuaApiMut::register_function(lua, "ClearAllLFGDungeons", queue::clear_all_lfg_dungeons)?;
    LuaApiMut::register_function(lua, "SetLFGDungeon", queue::set_lfg_dungeon)?;
    LuaApiMut::register_function(lua, "JoinLFG", queue::join_lfg)?;
    LuaApiMut::register_function(lua, "LeaveLFG", queue::leave_lfg)?;
    LuaApiMut::register_function(lua, "GetLFGInfoServer", queue::get_lfg_info_server)?;
    LuaApiMut::register_function(lua, "GetLFGQueuedList", queue::get_lfg_queued_list)?;
    LuaApiMut::register_function(lua, "GetLFGQueueStats", queue::get_lfg_queue_stats)?;
    register_lfg_proposal_globals(lua)?;
    Ok(())
}

fn register_lfg_proposal_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLFGProposal", queue::get_lfg_proposal)?;
    LuaApiMut::register_function(lua, "GetLFGProposalMember", queue::get_lfg_proposal_member)?;
    LuaApiMut::register_function(
        lua,
        "GetLFGProposalEncounter",
        queue::get_lfg_proposal_encounter,
    )?;
    LuaApiMut::register_function(lua, "AcceptProposal", queue::accept_proposal)?;
    LuaApiMut::register_function(lua, "RejectProposal", queue::reject_proposal)?;
    Ok(())
}
