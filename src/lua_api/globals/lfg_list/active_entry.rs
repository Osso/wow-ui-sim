//! The player's own listing: `CreateListing` / `UpdateListing` / `RemoveListing`
//! mutate `SimState::lfg_active_entry`; `GetActiveEntryInfo` publishes it as
//! `LfgEntryData`. `LFG_LIST_ACTIVE_ENTRY_UPDATE` is deferred to the next timer
//! tick because the real event arrives after the server round trip.

use super::catalog::u32_array_table;
use super::{defer_lfg_event, fire_event_with_args};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
};
use crate::lua_api::state_types::LfgActiveEntry;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::time::Instant;

fn number_field(state: &mut LuaState, data: Val, name: &str) -> LuaResult<Option<f64>> {
    match table_get(state, data, name) {
        Val::Nil => Ok(None),
        Val::Num(value) if value.is_finite() => Ok(Some(value)),
        _ => Err(runtime_error(format!(
            "LfgListingCreateData.{name} must be a number"
        ))),
    }
}

fn bool_field(state: &mut LuaState, data: Val, name: &str) -> LuaResult<bool> {
    match table_get(state, data, name) {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "LfgListingCreateData.{name} must be a boolean"
        ))),
    }
}

fn activity_ids(state: &mut LuaState, data: Val) -> LuaResult<Vec<u32>> {
    let ids = table_get(state, data, "activityIDs");
    let Val::Table(reference) = ids else {
        return Err(runtime_error(
            "LfgListingCreateData.activityIDs must be a table",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("LfgListingCreateData.activityIDs is not a live table"))?;
    let mut out = Vec::new();
    for index in 1.. {
        match table.get_int(index) {
            Val::Nil => break,
            Val::Num(id) => out.push(id as u32),
            _ => {
                return Err(runtime_error(
                    "LfgListingCreateData.activityIDs entries must be numbers",
                ));
            }
        }
    }
    Ok(out)
}

fn read_create_data(state: &mut LuaState) -> LuaResult<LfgActiveEntry> {
    let data = stack_val(state, 1);
    if !matches!(data, Val::Table(_)) {
        return Err(runtime_error(
            "listing requires an LfgListingCreateData table",
        ));
    }
    Ok(LfgActiveEntry {
        activity_ids: activity_ids(state, data)?,
        quest_id: number_field(state, data, "questID")?.map(|id| id as i32),
        auto_accept: bool_field(state, data, "isAutoAccept")?,
        cross_faction_listing: bool_field(state, data, "isCrossFactionListing")?,
        private_group: bool_field(state, data, "isPrivateGroup")?,
        new_player_friendly: bool_field(state, data, "newPlayerFriendly")?,
        playstyle: number_field(state, data, "playstyle")?.unwrap_or(0.0) as i32,
        general_playstyle: number_field(state, data, "generalPlaystyle")?.unwrap_or(0.0) as i32,
        required_dungeon_score: number_field(state, data, "requiredDungeonScore")?.unwrap_or(0.0),
        required_item_level: number_field(state, data, "requiredItemLevel")?.unwrap_or(0.0),
        required_pvp_rating: number_field(state, data, "requiredPvpRating")?.unwrap_or(0.0),
        censored: false,
        created_at: Instant::now(),
    })
}

/// `CreateListing(createData)` -> success. Fails while a listing is active.
pub(super) fn create_listing(state: &mut LuaState) -> LuaResult<u32> {
    let entry = read_create_data(state)?;
    let created = {
        let mut sim = borrow_state_mut(state)?;
        let free = sim.lfg_active_entry.is_none();
        if free {
            sim.lfg_active_entry = Some(entry);
        }
        free
    };
    if created {
        defer_lfg_event(state, dispatch_created, "C_LFGList.ActiveEntryCreated")?;
    }
    state.push(Val::Bool(created));
    Ok(1)
}

/// `UpdateListing(createData)` -> success. Keeps creation time and moderation
/// state; fails without an active listing.
pub(super) fn update_listing(state: &mut LuaState) -> LuaResult<u32> {
    let entry = read_create_data(state)?;
    let updated = {
        let mut sim = borrow_state_mut(state)?;
        match sim.lfg_active_entry.as_mut() {
            Some(active) => {
                *active = LfgActiveEntry {
                    censored: active.censored,
                    created_at: active.created_at,
                    ..entry
                };
                true
            }
            None => false,
        }
    };
    if updated {
        defer_lfg_event(state, dispatch_updated, "C_LFGList.ActiveEntryUpdated")?;
    }
    state.push(Val::Bool(updated));
    Ok(1)
}

pub(super) fn remove_listing(state: &mut LuaState) -> LuaResult<u32> {
    let removed = borrow_state_mut(state)?.lfg_active_entry.take().is_some();
    if removed {
        defer_lfg_event(state, dispatch_removed, "C_LFGList.ActiveEntryRemoved")?;
    }
    Ok(0)
}

fn dispatch_created(state: &mut LuaState) -> LuaResult<u32> {
    fire_event_with_args(state, "LFG_LIST_ACTIVE_ENTRY_UPDATE", vec![Val::Bool(true)])?;
    Ok(0)
}

// INFERRED: an in-place update reports created=false.
fn dispatch_updated(state: &mut LuaState) -> LuaResult<u32> {
    fire_event_with_args(
        state,
        "LFG_LIST_ACTIVE_ENTRY_UPDATE",
        vec![Val::Bool(false)],
    )?;
    Ok(0)
}

fn dispatch_removed(state: &mut LuaState) -> LuaResult<u32> {
    fire_event_with_args(state, "LFG_LIST_ACTIVE_ENTRY_UPDATE", Vec::new())?;
    Ok(0)
}

pub(super) fn has_active_entry_info(state: &mut LuaState) -> LuaResult<u32> {
    let has = borrow_state(state)?.lfg_active_entry.is_some();
    state.push(Val::Bool(has));
    Ok(1)
}

/// `GetActiveEntryInfo()` -> `LfgEntryData`, or nothing without a listing.
pub(super) fn get_active_entry_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(entry) = borrow_state(state)?.lfg_active_entry.clone() else {
        return Ok(0);
    };
    let info = create_table(state);
    state.push(info);
    let ids = u32_array_table(state, &entry.activity_ids);
    table_set(state, info, "activityIDs", ids);
    set_entry_fields(state, info, &entry);
    Ok(1)
}

fn set_entry_fields(state: &mut LuaState, info: Val, entry: &LfgActiveEntry) {
    // INFERRED: title/comment/voice come from the protected entry-creation
    // edit boxes, which are not modeled, so they publish as empty strings.
    for name in ["name", "comment", "voiceChat"] {
        let empty = create_string(state, "");
        table_set(state, info, name, empty);
    }
    for (name, value) in [
        ("requiredItemLevel", entry.required_item_level),
        // INFERRED: honor-level requirements are not part of LfgListingCreateData.
        ("requiredHonorLevel", 0.0),
        ("requiredDungeonScore", entry.required_dungeon_score),
        ("requiredPvpRating", entry.required_pvp_rating),
        ("playstyle", f64::from(entry.playstyle)),
        ("generalPlaystyle", f64::from(entry.general_playstyle)),
        // INFERRED: duration is whole seconds since the listing was created.
        ("duration", entry.created_at.elapsed().as_secs() as f64),
    ] {
        table_set(state, info, name, Val::Num(value));
    }
    if let Some(quest_id) = entry.quest_id {
        table_set(state, info, "questID", Val::Num(f64::from(quest_id)));
    }
    for (name, value) in [
        ("autoAccept", entry.auto_accept),
        ("privateGroup", entry.private_group),
        ("isCrossFactionListing", entry.cross_faction_listing),
        ("newPlayerFriendly", entry.new_player_friendly),
    ] {
        table_set(state, info, name, Val::Bool(value));
    }
    set_patch_12_1_entry_fields(state, info, entry);
}

#[cfg(feature = "retail-12-1-0")]
fn set_patch_12_1_entry_fields(state: &mut LuaState, info: Val, entry: &LfgActiveEntry) {
    table_set(state, info, "censored", Val::Bool(entry.censored));
}

#[cfg(not(feature = "retail-12-1-0"))]
fn set_patch_12_1_entry_fields(_state: &mut LuaState, _info: Val, _entry: &LfgActiveEntry) {}
