//! `C_Roleset` active blocklist/allowlist filters backed by `SimState`.
//!
//! The filters are stored and reported exactly as applied, and every tagged
//! frame is reevaluated once per apply. A filtered frame keeps its shown
//! state but is never visible (`Frame::roleset_filtered`). OnShow/OnHide are
//! not dispatched for filter transitions (unverified against the client).

use super::helpers::ensure_namespace;
use crate::lua_api::SimState;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_num, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Roleset")?;
    table_set_rust_fn_static(state, namespace, "ApplyRolesetFilters", apply_filters)?;
    if !cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        return Ok(());
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "GetActiveAllowedRolesets",
        get_active_allowed,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetActiveBlockedRolesets",
        get_active_blocked,
    )
}

/// Replaces both filters atomically; an empty table clears that filter.
fn apply_filters(state: &mut LuaState) -> LuaResult<u32> {
    let blocked = roleset_names(state, 1)?;
    let allowed = roleset_names(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.active_blocked_rolesets = blocked;
    sim.active_allowed_rolesets = allowed;
    refresh_all_roleset_filters(&mut sim);
    Ok(0)
}

/// Roleset that hides its frames whatever filters are active.
const ALWAYS_BLOCKED: &str = "alwaysBlocked";

/// Whether the active filters hide a frame tagged with `rolesets`.
///
/// Untagged frames are never filtered (INFERRED: tagging is opt-in, and an
/// allowlist hiding untagged frames would hide UIParent).
fn is_filtered(rolesets: &[String], blocked: &[String], allowed: &[String]) -> bool {
    if rolesets.is_empty() {
        return false;
    }
    let blocked_hit = rolesets
        .iter()
        .any(|name| name == ALWAYS_BLOCKED || blocked.contains(name));
    let allowed_miss = !allowed.is_empty() && !rolesets.iter().any(|name| allowed.contains(name));
    blocked_hit || allowed_miss
}

/// Change a frame's tags and reevaluate it against the active filters.
pub(crate) fn update_frame_rolesets(
    sim: &mut SimState,
    id: u64,
    update: impl FnOnce(&mut Vec<String>),
) {
    let Some(frame) = sim.widgets.get_mut(id) else {
        return;
    };
    update(&mut frame.rolesets);
    refresh_frame_roleset_filter(sim, id);
}

/// Parse a comma-separated roleset list, dropping blanks and duplicates.
pub(crate) fn split_roleset_list(list: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for name in list
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        if !names.iter().any(|existing| existing == name) {
            names.push(name.to_string());
        }
    }
    names
}

fn refresh_frame_roleset_filter(sim: &mut SimState, id: u64) {
    let Some(frame) = sim.widgets.get(id) else {
        return;
    };
    let filtered = is_filtered(
        &frame.rolesets,
        &sim.active_blocked_rolesets,
        &sim.active_allowed_rolesets,
    );
    sim.set_frame_roleset_filtered(id, filtered);
}

fn refresh_all_roleset_filters(sim: &mut SimState) {
    let tagged: Vec<u64> = sim
        .widgets
        .iter_ids()
        .filter(|&id| {
            sim.widgets
                .get(id)
                .is_some_and(|frame| !frame.rolesets.is_empty())
        })
        .collect();
    for id in tagged {
        refresh_frame_roleset_filter(sim, id);
    }
}

fn get_active_allowed(state: &mut LuaState) -> LuaResult<u32> {
    let names = borrow_state(state)?.active_allowed_rolesets.clone();
    push_names(state, &names);
    Ok(1)
}

fn get_active_blocked(state: &mut LuaState) -> LuaResult<u32> {
    let names = borrow_state(state)?.active_blocked_rolesets.clone();
    push_names(state, &names);
    Ok(1)
}

fn roleset_names(state: &LuaState, index: i32) -> LuaResult<Vec<String>> {
    let Val::Table(table) = stack_val(state, index) else {
        return Err(rilua::runtime_error(
            "C_Roleset.ApplyRolesetFilters: blockedRolesets and allowedRolesets must be tables",
        ));
    };
    let values: Vec<Val> = state
        .gc
        .tables
        .get(table)
        .map(|table| {
            table
                .array_slice()
                .iter()
                .copied()
                .take_while(|value| !matches!(value, Val::Nil))
                .collect()
        })
        .unwrap_or_default();
    Ok(values
        .into_iter()
        .filter_map(|value| val_to_string(state, value))
        .collect())
}

pub(crate) fn push_names(state: &mut LuaState, names: &[String]) {
    let result = create_table(state);
    let Val::Table(array) = result else {
        unreachable!("create_table must return a table");
    };
    for (index, name) in names.iter().enumerate() {
        let name = create_string(state, name);
        table_set_num(state, array, (index + 1) as f64, name);
    }
    state.push(result);
}
