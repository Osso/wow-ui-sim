//! Faction / reputation probe + selection globals.
//!
//! Migrates 5 entries off stubs:
//!
//! - `GetFactionInfoByID(factionID)` → 16-value retail shape from a
//!   matching `SimState.factions` entry, or nil when unknown.
//! - `GetGuildFactionInfo()`         → 6-value shape for the synthetic
//!   guild faction: name from `world.guild_name`, other fields fixed
//!   to Exalted / Friendly defaults until the sim models guild rep.
//! - `GetSelectedFaction()`          → `SimState.selected_faction_index`.
//! - `SetSelectedFaction(index)`     → write it back + fire
//!   `UPDATE_FACTION`.
//! - `SetWatchedFaction(index)`      → write
//!   `SimState.watched_faction_index` and flip the matching entry's
//!   `is_watched` flag; fires `UPDATE_FACTION`.
//! - `GetWatchedFactionInfo()`       → legacy status-bar tuple for the
//!   currently watched faction, or nil when no faction is watched.

use crate::event::Event;
use crate::lua_api::globals::reputation_data;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::stack_val;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table as RiluaTable;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_u32(state: &LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

fn push_update_faction_event(state: &mut LuaState) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: "UPDATE_FACTION".to_string(),
        args: Vec::new(),
    });
    Ok(())
}

/// `GetFactionInfoByID(factionID)` — retail: 16 values per row. When
/// the id isn't in `SimState.factions` we return nothing (matches
/// retail's "unknown faction" behaviour).
fn get_faction_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let Some(faction_id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let Some(entry) = borrow_state(state)?
        .factions
        .iter()
        .find(|e| e.faction_id == faction_id)
        .cloned()
    else {
        return Ok(0);
    };
    let name = create_string(state, &entry.name);
    let description = create_string(state, &entry.description);
    state.push(name); // 1: name
    state.push(description); // 2: description
    state.push(Val::Num(entry.standing as f64)); // 3: standingID
    state.push(Val::Num(entry.bottom as f64)); // 4: bottomValue
    state.push(Val::Num(entry.top as f64)); // 5: topValue
    state.push(Val::Num(entry.earned as f64)); // 6: earnedValue
    state.push(Val::Bool(entry.at_war)); // 7: atWarWith
    state.push(Val::Bool(entry.can_toggle_at_war)); // 8: canToggleAtWar
    state.push(Val::Bool(entry.is_header)); // 9: isHeader
    state.push(Val::Bool(entry.is_collapsed)); // 10: isCollapsed
    state.push(Val::Bool(entry.has_rep)); // 11: hasRep
    state.push(Val::Bool(entry.is_watched)); // 12: isWatched
    state.push(Val::Bool(entry.is_child)); // 13: isChild
    state.push(Val::Num(entry.faction_id as f64)); // 14: factionID
    state.push(Val::Bool(entry.has_bonus_rep_gain)); // 15: hasBonusRepGain
    state.push(Val::Bool(entry.can_be_lfg_bonus)); // 16: canBeLFGBonus
    Ok(16)
}

/// `GetGuildFactionInfo()` — retail: `(name, description, standingID,
/// bottomValue, topValue, earnedValue)`. We synthesise a guild faction
/// from `WorldState.guild_name` at Exalted (8) standing; the sim
/// doesn't model guild reputation progression.
fn get_guild_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let guild_name = borrow_state(state)?.world.guild_name.clone();
    let Some(guild_name) = guild_name else {
        return Ok(0);
    };
    let name = create_string(state, &guild_name);
    let description = create_string(state, "Guild");
    state.push(name);
    state.push(description);
    state.push(Val::Num(8.0)); // standingID: Exalted
    state.push(Val::Num(0.0)); // bottomValue
    state.push(Val::Num(1000.0)); // topValue
    state.push(Val::Num(1000.0)); // earnedValue (capped)
    Ok(6)
}

/// `GetSelectedFaction()` — 1-based reputation-window index.
fn get_selected_faction(state: &mut LuaState) -> LuaResult<u32> {
    let idx = borrow_state(state)?.selected_faction_index;
    state.push(Val::Num(idx as f64));
    Ok(1)
}

/// `SetSelectedFaction(index)` — select a reputation-window row.
/// Retail clamps to the valid range silently; we do the same.
fn set_selected_faction(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    {
        let mut sim = borrow_state_mut(state)?;
        let len = sim.factions.len() as i32;
        sim.selected_faction_index = index.clamp(0, len);
    }
    push_update_faction_event(state)?;
    Ok(0)
}

/// `SetWatchedFaction(index)` — show the faction on the XP bar and
/// clear the previous watch flag. `index == 0` stops watching.
fn set_watched_faction(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    {
        let mut sim = borrow_state_mut(state)?;
        let len = sim.factions.len() as i32;
        let clamped = index.clamp(0, len);
        for entry in sim.factions.iter_mut() {
            entry.is_watched = false;
        }
        if clamped > 0 {
            if let Some(entry) = sim.factions.get_mut((clamped - 1) as usize) {
                entry.is_watched = true;
            }
        }
        sim.watched_faction_index = clamped;
    }
    push_update_faction_event(state)?;
    Ok(0)
}

fn get_watched_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(entry) = watched_faction_entry(state)? else {
        return Ok(0);
    };

    let name = create_string(state, &entry.name);
    state.push(name);
    state.push(Val::Num(entry.standing as f64));
    state.push(Val::Num(entry.bottom as f64));
    state.push(Val::Num(entry.top as f64));
    state.push(Val::Num(entry.earned as f64));
    state.push(Val::Num(entry.faction_id as f64));
    Ok(6)
}

fn watched_faction_entry(
    state: &LuaState,
) -> LuaResult<Option<crate::lua_api::state::FactionEntry>> {
    let sim = borrow_state(state)?;
    let index = sim.watched_faction_index;
    if index <= 0 {
        return Ok(None);
    }

    let entry = sim.factions.get((index - 1) as usize).cloned();
    Ok(entry)
}

fn reputation_entry_table(
    state: &mut LuaState,
    entry: &reputation_data::FactionEntry,
    _faction_id: i32,
) -> Val {
    let table = create_table(state);
    set_reputation_entry_text_fields(state, table, entry);
    set_reputation_entry_numeric_fields(state, table, entry);
    set_reputation_entry_flags(state, table, entry);
    table
}

fn set_reputation_entry_text_fields(
    state: &mut LuaState,
    table: Val,
    entry: &reputation_data::FactionEntry,
) {
    let name = create_string(state, entry.name);
    let description = create_string(state, entry.description);
    table_set(state, table, "name", name);
    table_set(state, table, "description", description);
}

fn set_reputation_entry_numeric_fields(
    state: &mut LuaState,
    table: Val,
    entry: &reputation_data::FactionEntry,
) {
    table_set(state, table, "factionID", Val::Num(entry.faction_id as f64));
    table_set(state, table, "reaction", Val::Num(entry.reaction as f64));
    table_set(state, table, "standing", Val::Num(entry.standing as f64));
    table_set(state, table, "currentReactionThreshold", Val::Num(0.0));
    table_set(
        state,
        table,
        "nextReactionThreshold",
        Val::Num(entry.top_value as f64),
    );
    table_set(
        state,
        table,
        "currentStanding",
        Val::Num(entry.standing as f64),
    );
    table_set(state, table, "topValue", Val::Num(entry.top_value as f64));
}

fn set_reputation_entry_flags(
    state: &mut LuaState,
    table: Val,
    entry: &reputation_data::FactionEntry,
) {
    table_set(state, table, "isHeader", Val::Bool(entry.is_header));
    table_set(state, table, "isCollapsed", Val::Bool(entry.is_collapsed));
    table_set(state, table, "isChild", Val::Bool(entry.is_child));
    table_set(
        state,
        table,
        "isAccountWide",
        Val::Bool(entry.is_account_wide),
    );
}

const REPUTATION_METHODS: &[(&str, RustFn)] = &[
    ("GetFactionDataByID", reputation_get_faction_data_by_id),
    ("IsFactionParagon", reputation_is_faction_paragon),
    (
        "IsFactionParagonForCurrentPlayer",
        reputation_is_faction_paragon_for_current_player,
    ),
    (
        "GetFactionParagonInfo",
        crate::c_api::c_reputation::get_faction_paragon_info,
    ),
    ("GetNumFactions", reputation_get_num_factions),
    ("GetFactionInfo", reputation_get_faction_info),
    ("GetWatchedFactionData", reputation_get_watched_faction_data),
    (
        "SetWatchedFactionByID",
        reputation_set_watched_faction_by_id,
    ),
    ("IsMajorFaction", reputation_is_major_faction),
    (
        "IsAccountWideReputation",
        reputation_is_account_wide_reputation,
    ),
];

fn register_reputation_namespace(lua: &mut rilua::Lua) -> crate::Result<()> {
    let reputation = reuse_or_create_namespace_table(lua, "C_Reputation");
    let Val::Table(reputation_ref) = reputation else {
        unreachable!("namespace table must be a table");
    };
    install_static_methods(lua.state_mut(), reputation_ref, REPUTATION_METHODS)?;
    LuaApiMut::set_global_val(lua, "C_Reputation", reputation)?;
    Ok(())
}

fn reuse_or_create_namespace_table(lua: &mut rilua::Lua, global_name: &str) -> Val {
    let existing = LuaApiMut::get_global_val(lua, global_name);
    match existing {
        Val::Table(table) => Val::Table(table),
        _ => create_table(lua.state_mut()),
    }
}

fn install_static_methods(
    state: &mut LuaState,
    table_ref: GcRef<RiluaTable>,
    methods: &[(&'static str, RustFn)],
) -> crate::Result<()> {
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn reputation_get_faction_data_by_id(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn reputation_is_faction_paragon(state: &mut LuaState) -> LuaResult<u32> {
    let is_paragon = match stack_i32(state, 1) {
        Some(id) => borrow_state(state)?
            .faction_paragon
            .contains_key(&(id as i64)),
        None => false,
    };
    state.push(Val::Bool(is_paragon));
    Ok(1)
}

fn reputation_is_faction_paragon_for_current_player(state: &mut LuaState) -> LuaResult<u32> {
    let is_active = match stack_i32(state, 1) {
        Some(id) => borrow_state(state)?
            .faction_paragon
            .get(&(id as i64))
            .is_some_and(|info| !info.too_low_level_for_paragon),
        None => false,
    };
    state.push(Val::Bool(is_active));
    Ok(1)
}

fn reputation_get_num_factions(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(reputation_data::num_factions() as f64));
    Ok(1)
}

fn reputation_get_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    let Some(entry) = reputation_data::get_faction_by_index(index) else {
        return Ok(0);
    };
    let table = reputation_entry_table(state, entry, index);
    state.push(table);
    Ok(1)
}

fn reputation_get_watched_faction_data(state: &mut LuaState) -> LuaResult<u32> {
    let Some(entry) = reputation_data::watched_faction() else {
        return Ok(0);
    };
    let table = reputation_entry_table(state, entry, entry.faction_id);
    state.push(table);
    Ok(1)
}

fn reputation_set_watched_faction_by_id(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn reputation_is_major_faction(state: &mut LuaState) -> LuaResult<u32> {
    let is_major = match stack_i32(state, 1) {
        Some(id) => borrow_state(state)?
            .major_factions
            .contains_key(&(id as i64)),
        None => false,
    };
    state.push(Val::Bool(is_major));
    Ok(1)
}

fn reputation_is_account_wide_reputation(state: &mut LuaState) -> LuaResult<u32> {
    let is_account_wide = match stack_i32(state, 1) {
        Some(id) => is_account_wide_reputation_id(state, id)?,
        None => false,
    };
    state.push(Val::Bool(is_account_wide));
    Ok(1)
}

fn is_account_wide_reputation_id(state: &mut LuaState, id: i32) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(sim.account_wide_reputation_factions.contains(&(id as i64)))
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetFactionInfoByID", get_faction_info_by_id)?;
    LuaApiMut::register_function(lua, "GetGuildFactionInfo", get_guild_faction_info)?;
    LuaApiMut::register_function(lua, "GetSelectedFaction", get_selected_faction)?;
    LuaApiMut::register_function(lua, "SetSelectedFaction", set_selected_faction)?;
    LuaApiMut::register_function(lua, "SetWatchedFaction", set_watched_faction)?;
    LuaApiMut::register_function(lua, "GetWatchedFactionInfo", get_watched_faction_info)?;
    register_reputation_namespace(lua)?;
    Ok(())
}
