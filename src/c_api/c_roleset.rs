//! `C_Roleset` active blocklist/allowlist filters backed by `SimState`.
//!
//! The filters are stored and reported exactly as applied. Frame visibility
//! reevaluation against frame roleset membership is not modeled yet; frames
//! keep their own shown state.

use super::helpers::ensure_namespace;
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
    Ok(0)
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

fn push_names(state: &mut LuaState, names: &[String]) {
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
