//! Legacy archaeology probes consumed by `Blizzard_ArchaeologyUI`.
//!
//! These globals predate the `C_*` namespacing pass; they live at the
//! top-level Lua surface and are not exposed under `C_ResearchInfo`.
//!
//! Race-summary surface (gates the OnLoad path):
//! - `GetArchaeologyInfo() → professionName: string`
//! - `GetNumArchaeologyRaces() → number`
//! - `GetArchaeologyRaceInfo(raceIndex: number, getCurrentArtifact: bool?) →
//!   name: string, texture: number, raceItemID: number, currencyAmount: number,
//!   projectAmount: number` (returns nothing for out-of-range indices)
//! - `GetNumArtifactsByRace(raceIndex: number) → number` (0 for out-of-range)
//!
//! Active-artifact surface (drives the artifact page):
//! - `GetSelectedArtifactInfo() →
//!   name, description, rarity, icon, spellDescription, numSockets,
//!   bgTexture, spellID` (returns nothing when no artifact is selected)
//! - `SetSelectedArtifact(raceID: number, artifactID: number?)`
//! - `GetArtifactProgress() → base, adjust, totalCost`
//! - `CanSolveArtifact() → bool`
//! - `SolveArtifact()` — clears progress, fires
//!   `RESEARCH_ARTIFACT_COMPLETE` with the artifact name as payload.
//!
//! Keystone-socket surface (drives the keystone buttons):
//! - `ItemAddedToArtifact(socketIndex: number) → bool` — true when the
//!   1-based socket holds a keystone.
//! - `SocketItemToArtifact()` — slots a keystone into the leftmost empty
//!   socket and adds `keystone_value` to `adjust_progress`.
//! - `RemoveItemFromArtifact()` — empties the rightmost-set socket and
//!   subtracts `keystone_value` from `adjust_progress`.
//!
//! Completion-history surface (drives the completed-page paginator):
//! - `IsArtifactCompletionHistoryAvailable() → bool`
//! - `GetArtifactInfoByRace(raceIndex, projectIndex) →
//!   name, description, rarity, icon, spellDescription, _, _, spellID,
//!   firstCompletionTime, completionCount` (10 returns; positions 6 and 7
//!   are explicitly ignored by the addon — we push 0). Returns nothing
//!   for out-of-range indices so the paginator's `if not name` advances
//!   to the next race.
//! - `RequestArtifactCompletionHistory()` — server-driven companion;
//!   stubbed as flipping `history_available` to true.
//!
//! Close hint:
//! - `CloseResearch()` — fired by `ArchaeologyFrame_OnHide` (`:51`) and
//!   `ArchaeologyFrame_ShowFailed` (`:201`). Records the call timestamp
//!   into `state.archaeology.last_close_request` so tests can assert it
//!   ran; otherwise a no-op.
//!
//! Without `GetArchaeologyInfo` the addon errors out at
//! `Blizzard_ArchaeologyUI.lua:102` during `OnLoad` and `ArchaeologyFrame`
//! becomes a half-initialized table.

use crate::event::{Event, EventArg};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::SelectedArtifact;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::time::Instant;

fn get_archaeology_info(state: &mut LuaState) -> LuaResult<u32> {
    let profession = borrow_state(state)?.archaeology.profession_name.clone();
    let val = create_string(state, &profession);
    state.push(val);
    Ok(1)
}

fn get_num_archaeology_races(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.archaeology.races.len() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

/// Race-row tuple pushed onto the Lua stack by `GetArchaeologyRaceInfo`.
struct RaceRow {
    name: String,
    texture: u32,
    race_item_id: u32,
    currency_amount: i32,
    project_amount: i32,
}

fn race_row_for(state: &mut LuaState, race_index: i32) -> LuaResult<Option<RaceRow>> {
    let sim = borrow_state(state)?;
    let Some(race) = sim.archaeology.race_at(race_index) else {
        return Ok(None);
    };
    Ok(Some(RaceRow {
        name: race.name.clone(),
        texture: race.texture,
        race_item_id: race.race_item_id,
        currency_amount: race.currency_amount,
        project_amount: race.project_amount,
    }))
}

fn push_race_row(state: &mut LuaState, row: RaceRow) -> LuaResult<u32> {
    let name_val = create_string(state, &row.name);
    state.push(name_val);
    state.push(Val::Num(row.texture as f64));
    state.push(Val::Num(row.race_item_id as f64));
    state.push(Val::Num(row.currency_amount as f64));
    state.push(Val::Num(row.project_amount as f64));
    Ok(5)
}

fn get_archaeology_race_info(state: &mut LuaState) -> LuaResult<u32> {
    let race_index = i32::from_stack(state, 1)?;
    // Arg 2 (`getCurrentArtifact: bool?`) selects an alternate set of returns
    // for the artifact page; the race-summary surface ignores it. The
    // active-artifact branch is filed as a separate priority entry.
    match race_row_for(state, race_index)? {
        Some(row) => push_race_row(state, row),
        None => Ok(0),
    }
}

fn get_num_artifacts_by_race(state: &mut LuaState) -> LuaResult<u32> {
    let race_index = i32::from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.archaeology
            .race_at(race_index)
            .map(|r| r.artifacts.len() as f64)
            .unwrap_or(0.0)
    };
    state.push(Val::Num(count));
    Ok(1)
}

/// Active-artifact info pushed by `GetSelectedArtifactInfo`. Held in
/// `SelectedArtifact` and copied out so the borrow on `SimState` releases
/// before the values land on the Lua stack.
struct ArtifactInfo {
    name: String,
    description: String,
    rarity: i32,
    icon: u32,
    spell_description: String,
    num_sockets: i32,
    bg_texture: String,
    spell_id: u32,
}

impl ArtifactInfo {
    fn from_selected(selected: &SelectedArtifact) -> Self {
        Self {
            name: selected.name.clone(),
            description: selected.description.clone(),
            rarity: selected.rarity,
            icon: selected.icon,
            spell_description: selected.spell_description.clone(),
            num_sockets: selected.num_sockets,
            bg_texture: selected.bg_texture.clone(),
            spell_id: selected.spell_id,
        }
    }
}

fn artifact_info_for_selected(state: &mut LuaState) -> LuaResult<Option<ArtifactInfo>> {
    let sim = borrow_state(state)?;
    Ok(sim
        .archaeology
        .selected
        .as_ref()
        .map(ArtifactInfo::from_selected))
}

fn push_artifact_info(state: &mut LuaState, info: ArtifactInfo) -> LuaResult<u32> {
    let name_val = create_string(state, &info.name);
    let description_val = create_string(state, &info.description);
    let spell_description_val = create_string(state, &info.spell_description);
    let bg_texture_val = create_string(state, &info.bg_texture);
    state.push(name_val);
    state.push(description_val);
    state.push(Val::Num(info.rarity as f64));
    state.push(Val::Num(info.icon as f64));
    state.push(spell_description_val);
    state.push(Val::Num(info.num_sockets as f64));
    state.push(bg_texture_val);
    state.push(Val::Num(info.spell_id as f64));
    Ok(8)
}

fn get_selected_artifact_info(state: &mut LuaState) -> LuaResult<u32> {
    match artifact_info_for_selected(state)? {
        Some(info) => push_artifact_info(state, info),
        None => Ok(0),
    }
}

fn set_selected_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let race_id = i32::from_stack(state, 1)?;
    let artifact_id = Option::<i32>::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    let next = match sim.archaeology.selected.take() {
        Some(prev) => SelectedArtifact {
            race_id,
            artifact_id,
            ..prev
        },
        None => SelectedArtifact {
            race_id,
            artifact_id,
            ..SelectedArtifact::default()
        },
    };
    sim.archaeology.selected = Some(next);
    Ok(0)
}

fn get_artifact_progress(state: &mut LuaState) -> LuaResult<u32> {
    let (base, adjust, total_cost) = {
        let sim = borrow_state(state)?;
        match sim.archaeology.selected.as_ref() {
            Some(s) => (s.base_progress, s.adjust_progress, s.total_cost),
            None => (0, 0, 0),
        }
    };
    state.push(Val::Num(base as f64));
    state.push(Val::Num(adjust as f64));
    state.push(Val::Num(total_cost as f64));
    Ok(3)
}

fn can_solve_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let result = borrow_state(state)?
        .archaeology
        .selected
        .as_ref()
        .is_some_and(|s| s.can_solve);
    state.push(Val::Bool(result));
    Ok(1)
}

fn solve_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let solved_name = {
        let mut sim = borrow_state_mut(state)?;
        let Some(selected) = sim.archaeology.selected.as_mut() else {
            return Ok(0);
        };
        selected.base_progress = 0;
        selected.adjust_progress = 0;
        selected.can_solve = false;
        selected.name.clone()
    };
    borrow_state_mut(state)?.events.push(Event {
        name: "RESEARCH_ARTIFACT_COMPLETE".to_string(),
        args: vec![EventArg::String(solved_name)],
    });
    Ok(0)
}

/// Resizes `selected.sockets` to match `selected.num_sockets` so the
/// keystone globals can index by `1..=num_sockets` regardless of how the
/// caller seeded the slot. Truncates excess entries (keystones beyond
/// the current capacity are dropped) and zero-fills missing entries.
fn normalize_sockets_to_capacity(selected: &mut SelectedArtifact) {
    let capacity = selected.num_sockets.max(0) as usize;
    selected.sockets.resize(capacity, false);
}

fn item_added_to_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let socket_index = i32::from_stack(state, 1)?;
    let occupied = if socket_index < 1 {
        false
    } else {
        let zero_based = (socket_index - 1) as usize;
        borrow_state(state)?
            .archaeology
            .selected
            .as_ref()
            .and_then(|s| s.sockets.get(zero_based).copied())
            .unwrap_or(false)
    };
    state.push(Val::Bool(occupied));
    Ok(1)
}

fn socket_item_to_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let keystone_value = sim.archaeology.keystone_value;
    let Some(selected) = sim.archaeology.selected.as_mut() else {
        return Ok(0);
    };
    normalize_sockets_to_capacity(selected);
    let Some(empty_index) = selected.sockets.iter().position(|filled| !filled) else {
        return Ok(0);
    };
    selected.sockets[empty_index] = true;
    selected.adjust_progress += keystone_value;
    Ok(0)
}

fn remove_item_from_artifact(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let keystone_value = sim.archaeology.keystone_value;
    let Some(selected) = sim.archaeology.selected.as_mut() else {
        return Ok(0);
    };
    normalize_sockets_to_capacity(selected);
    let Some(rightmost_set) = selected.sockets.iter().rposition(|filled| *filled) else {
        return Ok(0);
    };
    selected.sockets[rightmost_set] = false;
    selected.adjust_progress -= keystone_value;
    Ok(0)
}

fn is_artifact_completion_history_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.archaeology.history_available;
    state.push(Val::Bool(available));
    Ok(1)
}

fn request_artifact_completion_history(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.archaeology.history_available = true;
    Ok(0)
}

/// Snapshot of an `ArchaeologyArtifact` ready to be pushed as the 10
/// returns of `GetArtifactInfoByRace`. The addon explicitly ignores
/// positions 6 and 7 (`_, _,` in the destructuring at
/// `Blizzard_ArchaeologyUI.lua:450`); they are pushed as `0` so the
/// stack still has the right shape.
struct ArtifactRow {
    name: String,
    description: String,
    rarity: i32,
    icon: u32,
    spell_description: String,
    spell_id: u32,
    first_completion_time: i64,
    completion_count: i32,
}

fn artifact_row_for(
    state: &mut LuaState,
    race_index: i32,
    project_index: i32,
) -> LuaResult<Option<ArtifactRow>> {
    let sim = borrow_state(state)?;
    let Some(artifact) = sim.archaeology.artifact_at(race_index, project_index) else {
        return Ok(None);
    };
    Ok(Some(ArtifactRow {
        name: artifact.name.clone(),
        description: artifact.description.clone(),
        rarity: artifact.rarity,
        icon: artifact.icon,
        spell_description: artifact.spell_description.clone(),
        spell_id: artifact.spell_id,
        first_completion_time: artifact.first_completion_time,
        completion_count: artifact.completion_count,
    }))
}

fn push_artifact_row(state: &mut LuaState, row: ArtifactRow) -> LuaResult<u32> {
    let name_val = create_string(state, &row.name);
    let description_val = create_string(state, &row.description);
    let spell_description_val = create_string(state, &row.spell_description);
    state.push(name_val);
    state.push(description_val);
    state.push(Val::Num(row.rarity as f64));
    state.push(Val::Num(row.icon as f64));
    state.push(spell_description_val);
    // Returns 6 and 7 are documented as unused; the addon destructures
    // them as `_, _,` and never reads them. Push numeric zeros so the
    // tuple shape matches the documented signature.
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(row.spell_id as f64));
    state.push(Val::Num(row.first_completion_time as f64));
    state.push(Val::Num(row.completion_count as f64));
    Ok(10)
}

fn get_artifact_info_by_race(state: &mut LuaState) -> LuaResult<u32> {
    let race_index = i32::from_stack(state, 1)?;
    let project_index = i32::from_stack(state, 2)?;
    match artifact_row_for(state, race_index, project_index)? {
        Some(row) => push_artifact_row(state, row),
        None => Ok(0),
    }
}

fn close_research(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.archaeology.last_close_request = Some(Instant::now());
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetArchaeologyInfo", get_archaeology_info)?;
    LuaApiMut::register_function(lua, "GetNumArchaeologyRaces", get_num_archaeology_races)?;
    LuaApiMut::register_function(lua, "GetArchaeologyRaceInfo", get_archaeology_race_info)?;
    LuaApiMut::register_function(lua, "GetNumArtifactsByRace", get_num_artifacts_by_race)?;
    LuaApiMut::register_function(lua, "GetSelectedArtifactInfo", get_selected_artifact_info)?;
    LuaApiMut::register_function(lua, "SetSelectedArtifact", set_selected_artifact)?;
    LuaApiMut::register_function(lua, "GetArtifactProgress", get_artifact_progress)?;
    LuaApiMut::register_function(lua, "CanSolveArtifact", can_solve_artifact)?;
    LuaApiMut::register_function(lua, "SolveArtifact", solve_artifact)?;
    LuaApiMut::register_function(lua, "ItemAddedToArtifact", item_added_to_artifact)?;
    LuaApiMut::register_function(lua, "SocketItemToArtifact", socket_item_to_artifact)?;
    LuaApiMut::register_function(lua, "RemoveItemFromArtifact", remove_item_from_artifact)?;
    LuaApiMut::register_function(
        lua,
        "IsArtifactCompletionHistoryAvailable",
        is_artifact_completion_history_available,
    )?;
    LuaApiMut::register_function(lua, "GetArtifactInfoByRace", get_artifact_info_by_race)?;
    // Removed in 10.1.5; retain the modeled request for classic clients only.
    let retail = matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    );
    if !retail {
        LuaApiMut::register_function(
            lua,
            "RequestArtifactCompletionHistory",
            request_artifact_completion_history,
        )?;
    }
    LuaApiMut::register_function(lua, "CloseResearch", close_research)?;
    Ok(())
}
