//! `A_Admin.SetReputationFactions(rows[, guildRow])` supplies the host-fed
//! Retail/Forever reputation rows of [`super::reputation_model`].
//!
//! Field names follow `FactionData` in ReputationInfoDocumentation.lua on both
//! pins. The input replaces the whole row list and the guild row; it does not
//! invent faction acquisition or server state.

use super::reputation_model::{ReputationFaction, ReputationModel};
use crate::lua_api::methods::{borrow_state_mut, table_get, val_to_string};
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const FUNCTION: &str = "A_Admin.SetReputationFactions";

const FLAGS: &[&str] = &[
    "atWarWith",
    "canToggleAtWar",
    "isChild",
    "isHeader",
    "isHeaderWithRep",
    "isCollapsed",
    "hasBonusRepGain",
    "canSetInactive",
    "isAccountWide",
    "isInactive",
];

/// Registered only where the host-fed model backs `C_Reputation`.
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !super::reputation_model::active() {
        return Ok(builder);
    }
    builder.set_function("SetReputationFactions", set_reputation_factions)
}

struct SuppliedRows {
    rows: Vec<ReputationFaction>,
    watched: Option<i32>,
}

fn set_reputation_factions(state: &mut LuaState) -> LuaResult<u32> {
    let supplied = read_rows(state, stack_val(state, 1))?;
    let guild = match stack_val(state, 2) {
        Val::Nil => None,
        value => Some(read_row(state, value)?),
    };
    replace_model(&mut borrow_state_mut(state)?.reputation, supplied, guild);
    // INFERRED: supplied rows stand for a server reputation update, which the
    // original StatusTrackingBarManager and ReputationFrame observe as UPDATE_FACTION.
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "UPDATE_FACTION",
        &[],
    )?;
    Ok(0)
}

fn replace_model(
    model: &mut ReputationModel,
    supplied: SuppliedRows,
    guild: Option<ReputationFaction>,
) {
    let rows = &supplied.rows;
    let selected = model
        .selected_faction_id
        .filter(|id| rows.iter().any(|row| row.faction_id == *id));
    *model = ReputationModel {
        selected_faction_id: selected,
        watched_faction_id: supplied.watched,
        sort_type: model.sort_type,
        show_legacy: model.show_legacy,
        factions: supplied.rows,
        guild,
    };
}

fn read_rows(state: &mut LuaState, value: Val) -> LuaResult<SuppliedRows> {
    let Val::Table(array) = value else {
        return Err(runtime_error(format!(
            "{FUNCTION}: expected an array of FactionData rows"
        )));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .map_or(0, |table| table.len(&state.gc.string_arena));
    let mut supplied = SuppliedRows {
        rows: Vec::new(),
        watched: None,
    };
    for index in 1..=length {
        let value = state
            .gc
            .tables
            .get(array)
            .map_or(Val::Nil, |table| table.get_int(index as i64));
        let row = read_row(state, value)?;
        if read_watched(state, value)? {
            if supplied.watched.is_some() || row.faction_id <= 0 {
                return Err(invalid("isWatched"));
            }
            supplied.watched = Some(row.faction_id);
        }
        supplied.rows.push(row);
    }
    Ok(supplied)
}

/// FactionData `isWatched` selects the watched faction; at most one row may set it.
fn read_watched(state: &mut LuaState, row: Val) -> LuaResult<bool> {
    match table_get(state, row, "isWatched") {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(invalid("isWatched")),
    }
}

fn read_row(state: &mut LuaState, value: Val) -> LuaResult<ReputationFaction> {
    let Val::Table(table) = value else {
        return Err(runtime_error(format!(
            "{FUNCTION}: expected a FactionData table"
        )));
    };
    let flags = read_flags(state, table)?;
    Ok(ReputationFaction {
        faction_id: read_integer(state, table, "factionID")?,
        name: read_optional_string(state, table, "name")?,
        description: read_optional_string(state, table, "description")?,
        reaction: read_integer(state, table, "reaction")?,
        bottom: read_integer(state, table, "currentReactionThreshold")?,
        top: read_integer(state, table, "nextReactionThreshold")?,
        standing: read_integer(state, table, "currentStanding")?,
        reputation_index: read_reputation_index(state, table)?,
        ..flags
    })
}

fn read_flags(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<ReputationFaction> {
    let mut values = [false; 10];
    for (slot, key) in values.iter_mut().zip(FLAGS) {
        *slot = match table_get(state, Val::Table(table), key) {
            Val::Nil => false,
            Val::Bool(value) => value,
            _ => return Err(invalid(key)),
        };
    }
    let [
        at_war,
        toggle,
        child,
        header,
        header_rep,
        collapsed,
        bonus,
        can_inactive,
        account,
        inactive,
    ] = values;
    Ok(ReputationFaction {
        at_war,
        can_toggle_at_war: toggle,
        is_child: child,
        is_header: header,
        is_header_with_rep: header_rep,
        is_collapsed: collapsed,
        has_bonus_rep_gain: bonus,
        can_set_inactive: can_inactive,
        is_account_wide: account,
        is_inactive: inactive,
        ..ReputationFaction::default()
    })
}

fn read_integer(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<i32> {
    match table_get(state, Val::Table(table), key) {
        Val::Num(value)
            if value.fract() == 0.0
                && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) =>
        {
            Ok(value as i32)
        }
        _ => Err(invalid(key)),
    }
}

fn read_optional_string(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &str,
) -> LuaResult<Option<String>> {
    match table_get(state, Val::Table(table), key) {
        Val::Nil => Ok(None),
        value @ Val::Str(_) => Ok(val_to_string(state, value)),
        _ => Err(invalid(key)),
    }
}

fn read_reputation_index(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<Option<u16>> {
    match table_get(state, Val::Table(table), "reputationIndex") {
        Val::Nil => Ok(None),
        Val::Num(value) if value.fract() == 0.0 && (0.0..=f64::from(u16::MAX)).contains(&value) => {
            Ok(Some(value as u16))
        }
        _ => Err(invalid("reputationIndex")),
    }
}

fn invalid(key: &str) -> rilua::LuaError {
    runtime_error(format!("{FUNCTION}: invalid {key}"))
}
