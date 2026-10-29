//! Ordinary quest-watch membership, queries and synchronous change notifications.

use super::helpers::{ensure_namespace, validate_supplied_relation_id};
use crate::lua_api::SimState;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::sim_substates::QuestLogEntry;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::{HashMap, hash_map::Entry};

use crate::client_profile::{ACTIVE, ClientProfile};

/// Retail, PTR and Forever QuestLogDocumentation.lua declare these values.
pub(crate) const ENABLED: bool = matches!(
    ACTIVE,
    ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
);

/// Values declared by QuestLogDocumentation.lua.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum QuestWatchType {
    Automatic = 0,
    Manual = 1,
}

const METHODS: &[(&str, RustFn)] = &[
    ("AddQuestWatch", add_quest_watch),
    ("RemoveQuestWatch", remove_quest_watch),
    ("GetQuestWatchType", get_quest_watch_type),
    ("GetNumQuestWatches", get_num_quest_watches),
    ("GetQuestIDForQuestWatchIndex", get_quest_id_for_watch_index),
];
const GLOBALS: &[(&str, RustFn)] = &[
    ("GetNumQuestWatches", get_num_quest_watches),
    ("IsQuestWatched", is_quest_watched),
    ("GetQuestIndexForWatch", get_log_index_for_watch),
    ("GetQuestWatchIndex", get_watch_index_for_log_index),
    ("GetQuestSortIndex", get_watch_index_for_log_index),
];

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_QuestLog")?;
    for &(name, function) in METHODS {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    if ACTIVE == ClientProfile::WowForever {
        let globals = state.global;
        for &(name, function) in GLOBALS {
            table_set_rust_fn_static(state, globals, name, function)?;
        }
    }
    Ok(())
}

/// Preserve the existing ordinary seed once; later queries read explicit membership.
/// Manual seeding is user-approved for Retail/PTR, simulator policy, not native evidence.
pub(crate) fn seed_quest_watch_types(entries: &[QuestLogEntry]) -> HashMap<i32, QuestWatchType> {
    entries
        .iter()
        .filter(|entry| !entry.is_world_quest)
        .map(|entry| (entry.quest_id, QuestWatchType::Manual))
        .collect()
}

fn watched_entries(sim: &SimState) -> impl Iterator<Item = (usize, &QuestLogEntry)> + '_ {
    sim.quest_log_entries
        .entries
        .iter()
        .enumerate()
        .filter(move |(_, entry)| {
            !entry.is_world_quest && sim.quest_watch_types.contains_key(&entry.quest_id)
        })
}

fn is_ordinary_quest(sim: &SimState, quest_id: i32) -> bool {
    sim.quest_log_entries
        .entries
        .iter()
        .any(|entry| entry.quest_id == quest_id && !entry.is_world_quest)
}

fn read_quest_id(state: &LuaState, api: &str) -> LuaResult<i32> {
    let id = validate_supplied_relation_id(stack_val(state, 1), "questID must be a number")
        .map_err(|error| rilua::runtime_error(format!("{api}: {error}")))?;
    id.ok_or_else(|| rilua::runtime_error(format!("{api}: questID must be a finite i32 integer")))
}

fn insert_manual_watch(sim: &mut SimState, quest_id: i32) -> bool {
    if !is_ordinary_quest(sim, quest_id) {
        return false;
    }
    let Entry::Vacant(entry) = sim.quest_watch_types.entry(quest_id) else {
        return false;
    };
    entry.insert(QuestWatchType::Manual);
    true
}

fn update_watch_membership(state: &mut LuaState, quest_id: i32, added: bool) -> LuaResult<bool> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        if added {
            insert_manual_watch(&mut sim, quest_id)
        } else {
            sim.quest_watch_types.remove(&quest_id).is_some()
        }
    };
    if changed {
        dispatch_event_now(
            state,
            "QUEST_WATCH_LIST_CHANGED",
            &[Val::Num(f64::from(quest_id)), Val::Bool(added)],
        )?;
    }
    Ok(changed)
}

fn add_quest_watch(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = read_quest_id(state, "C_QuestLog.AddQuestWatch")?;
    let changed = update_watch_membership(state, quest_id, true)?;
    state.push(Val::Bool(changed));
    Ok(1)
}

fn remove_quest_watch(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = read_quest_id(state, "C_QuestLog.RemoveQuestWatch")?;
    let changed = update_watch_membership(state, quest_id, false)?;
    state.push(Val::Bool(changed));
    Ok(1)
}

fn get_quest_watch_type(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = read_quest_id(state, "C_QuestLog.GetQuestWatchType")?;
    let watch_type = {
        let sim = borrow_state(state)?;
        if is_ordinary_quest(&sim, quest_id) {
            sim.quest_watch_types.get(&quest_id).copied()
        } else {
            None
        }
    };
    let value = watch_type
        .map(|kind| Val::Num(f64::from(kind as i32)))
        .unwrap_or(Val::Nil);
    state.push(value);
    Ok(1)
}

fn get_num_quest_watches(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        watched_entries(&sim).count()
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn read_legacy_index(state: &LuaState) -> LuaResult<i32> {
    // Preserve the existing optional numeric index conversion and truncation.
    Ok(Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32)
}

fn get_quest_id_for_watch_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_legacy_index(state)?;
    if index < 1 {
        return Ok(0);
    }
    let quest_id = {
        let sim = borrow_state(state)?;
        let mut entries = watched_entries(&sim);
        entries
            .nth((index - 1) as usize)
            .map(|(_, entry)| entry.quest_id)
    };
    let Some(quest_id) = quest_id else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(quest_id)));
    Ok(1)
}

fn ordinary_id_at_log_index(sim: &SimState, index: i32) -> Option<i32> {
    if index < 2 {
        return None;
    }
    let entry = sim.quest_log_entries.entries.get((index - 2) as usize)?;
    (!entry.is_world_quest).then_some(entry.quest_id)
}

fn is_quest_watched(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_legacy_index(state)?;
    let watched = {
        let sim = borrow_state(state)?;
        ordinary_id_at_log_index(&sim, index)
            .is_some_and(|id| sim.quest_watch_types.contains_key(&id))
    };
    state.push(Val::Bool(watched));
    Ok(1)
}

fn get_log_index_for_watch(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_legacy_index(state)?;
    let log_index = if index < 1 {
        0
    } else {
        let sim = borrow_state(state)?;
        let mut entries = watched_entries(&sim);
        entries
            .nth((index - 1) as usize)
            .map(|(row, _)| row + 2)
            .unwrap_or(0)
    };
    state.push(Val::Num(log_index as f64));
    Ok(1)
}

fn get_watch_index_for_log_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_legacy_index(state)?;
    let watch_index = if index < 2 {
        0
    } else {
        let sim = borrow_state(state)?;
        let mut entries = watched_entries(&sim);
        entries
            .position(|(row, _)| row == (index - 2) as usize)
            .map(|row| row + 1)
            .unwrap_or(0)
    };
    state.push(Val::Num(watch_index as f64));
    Ok(1)
}
