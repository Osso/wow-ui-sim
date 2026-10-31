//! Value-access fallbacks: `issecurevariable`, `issecretvalue`,
//! `canaccessvalue`, `canaccessallvalues`, `canaccesstable`, plus the
//! per-slot taint-marking helpers.

use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

use crate::lua_api::methods::val_to_string;

use super::register_if_missing;
use super::secret_values::{mark_secret_value, table_is_secret, value_is_secret_shallow};

pub(super) fn register_value_access_fallbacks(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_if_missing(lua, "issecretvalue", issecretvalue_fallback)?;
    register_if_missing(lua, "canaccessvalue", canaccessvalue_fallback)?;
    register_if_missing(lua, "canaccessallvalues", canaccessallvalues_fallback)?;
    register_if_missing(lua, "canaccesstable", canaccesstable_fallback)?;
    Ok(())
}

pub(super) fn mark_secret_values(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = state.top.saturating_sub(state.base);
    for index in 0..nargs {
        mark_secret_value(state, state.stack_get(state.base + index));
    }
    Ok(0)
}

pub(super) fn mark_slot_taint(state: &mut LuaState) -> LuaResult<u32> {
    let table_val = state.stack_get(state.base);
    let key_val = state.stack_get(state.base + 1);
    let taint_val = state.stack_get(state.base + 2);
    let Some(taint) = val_to_string(state, taint_val) else {
        return Ok(0);
    };
    let Val::Table(table_ref) = table_val else {
        return Ok(0);
    };
    set_slot_taint(state, table_ref, key_val, &taint);
    Ok(0)
}

pub(super) fn issecurevariable_override(state: &mut LuaState) -> LuaResult<u32> {
    let first = state.stack_get(state.base);
    let second = state.stack_get(state.base + 1);
    let (table_ref, key_val) = match first {
        Val::Table(table_ref) if !matches!(second, Val::Nil) => (table_ref, second),
        Val::Str(_) | Val::Num(_) => (state.global, first),
        _ => {
            state.push(Val::Bool(true));
            return Ok(1);
        }
    };

    let taint = slot_taint_for_key(state, table_ref, key_val)
        .or_else(|| global_shadow_slot_taint_for_key(state, table_ref, key_val));

    match taint {
        None => {
            state.push(Val::Bool(true));
            Ok(1)
        }
        Some(taint) => {
            let taint = Val::Str(state.gc.intern_string(taint.as_bytes()));
            state.push(Val::Bool(false));
            state.push(taint);
            Ok(2)
        }
    }
}

fn slot_taint_for_key(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    key_val: Val,
) -> Option<String> {
    let table = state.gc.tables.get(table_ref)?;
    match key_val {
        Val::Str(s) => {
            let bytes = state.gc.string_arena.get(s)?.data();
            table.get_slot_taint_str(bytes).map(str::to_string)
        }
        Val::Num(n) if n.is_finite() && (n as i64) as f64 == n => {
            table.get_slot_taint_int(n as i64).map(str::to_string)
        }
        _ => None,
    }
}

fn global_shadow_slot_taint_for_key(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    key_val: Val,
) -> Option<String> {
    if table_ref != state.global {
        return None;
    }
    let runtime = state.global_slots.as_ref()?;
    let shadow_key = runtime.shadow_registry_key?;
    let registry = state.gc.tables.get(state.registry)?;
    let Val::Table(shadow_ref) = registry.get_str(shadow_key, &state.gc.string_arena) else {
        return None;
    };
    slot_taint_for_key(state, shadow_ref, key_val)
}

fn set_slot_taint(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    key_val: Val,
    taint: &str,
) {
    let Some(table) = state.gc.tables.get_mut(table_ref) else {
        return;
    };
    match key_val {
        Val::Str(s) => {
            let Some(bytes) = state.gc.string_arena.get(s).map(|ls| ls.data().to_vec()) else {
                return;
            };
            table.set_slot_taint_str(&bytes, taint);
        }
        Val::Num(n) if n.is_finite() && (n as i64) as f64 == n => {
            table.set_slot_taint_int(n as i64, taint);
        }
        _ => {}
    }
}

/// `issecretvalue(v)` — returns true for tainted values. Inspects the value
/// itself (string marker, function closure-taint, or direct slot-taint on a
/// table) without recursing into nested table contents; `canaccesstable` is
/// the deep check.
fn issecretvalue_fallback(state: &mut LuaState) -> LuaResult<u32> {
    let value = first_argument(state);
    let secret = value_is_secret_shallow(state, value);
    state.push(Val::Bool(secret));
    Ok(1)
}

/// The first argument, or nil when the call has none. Slots at or above
/// `top` may hold stale values from earlier calls.
fn first_argument(state: &LuaState) -> Val {
    if state.top > state.base {
        state.stack_get(state.base)
    } else {
        Val::Nil
    }
}

/// `canaccessvalue(v)` — returns true for non-secret values.
fn canaccessvalue_fallback(state: &mut LuaState) -> LuaResult<u32> {
    let value = first_argument(state);
    let secret = value_is_secret_shallow(state, value);
    state.push(Val::Bool(!secret));
    Ok(1)
}

/// `canaccessallvalues(...)` — returns true only if every argument is
/// non-secret.
fn canaccessallvalues_fallback(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = state.top.saturating_sub(state.base);
    let mut accessible = true;
    for index in 0..nargs {
        if value_is_secret_shallow(state, state.stack_get(state.base + index)) {
            accessible = false;
            break;
        }
    }
    state.push(Val::Bool(accessible));
    Ok(1)
}

/// `canaccesstable(t)` — returns true only for non-secret tables. Performs a
/// deep recursive walk so that tables containing tainted strings or
/// closures (e.g. unit identity names from `UnitName`) are reported as
/// inaccessible.
fn canaccesstable_fallback(state: &mut LuaState) -> LuaResult<u32> {
    let value = state.stack_get(state.base);
    let accessible = match value {
        Val::Table(table_ref) => !table_is_secret(state, table_ref, &mut HashSet::new()),
        _ => !value_is_secret_shallow(state, value),
    };
    state.push(Val::Bool(accessible));
    Ok(1)
}
