//! Secret-value tracking and scrub pass-throughs.
//!
//! Tracks which Lua values are "secret" via a registry table keyed by string
//! identity. Used by `issecretvalue` / `canaccess*` fallbacks and by C-side
//! Rust code that wants to mark unit-name strings as tainted.

#[cfg(feature = "retail-12-1-0")]
use rilua::LuaApiMut;
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

use crate::lua_api::methods::{registry_get, registry_set};

use super::register_if_missing;

const SECRET_VALUE_REGISTRY_KEY: &str = "__sim_secret_values";
const SECRET_TAINT_MARKER: &str = "*** SimSecretValue ***";
const LOADSTRING_SECRET_TAINT_MARKER: &str = "*** ForceTaint_Strong ***";

pub(super) fn register_scrub_fallbacks(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_if_missing(lua, "scrub", scrub_passthrough)?;
    register_if_missing(lua, "scrubsecretvalues", scrub_passthrough)?;
    register_if_missing(lua, "secretunwrap", secretunwrap_passthrough)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn register_retail_secret_values(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "secretwrap", secretwrap_native)?;
    LuaApiMut::register_function(lua, "secretunwrap", secretunwrap_native)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn secretwrap_native(state: &mut LuaState) -> LuaResult<u32> {
    let count = state.top - state.base;
    for index in 0..count {
        let value = rilua::table_security::wrap_secret(state, state.stack_get(state.base + index))?;
        state.push(value);
    }
    Ok(count as u32)
}

#[cfg(feature = "retail-12-1-0")]
fn secretunwrap_native(state: &mut LuaState) -> LuaResult<u32> {
    let count = state.top - state.base;
    for index in 0..count {
        let value =
            rilua::table_security::unwrap_secret(state, state.stack_get(state.base + index))?;
        state.push(value);
    }
    Ok(count as u32)
}

/// `scrub(...)` / `scrubsecretvalues(...)` — return args unchanged.
fn scrub_passthrough(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = (state.top as i32 - state.base as i32).max(0) as u32;
    Ok(nargs)
}

/// `secretunwrap(value)` — return the same value without changing its metadata.
fn secretunwrap_passthrough(state: &mut LuaState) -> LuaResult<u32> {
    if state.top <= state.base {
        state.push(Val::Nil);
    }
    Ok(1)
}

pub(crate) fn mark_secret_value(state: &mut LuaState, value: Val) {
    let Some(key) = secret_registry_key(state, value) else {
        return;
    };
    let marker = Val::Str(state.gc.intern_string(SECRET_TAINT_MARKER.as_bytes()));
    let table_ref = get_or_create_secret_value_table(state);
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(key, marker, &state.gc.string_arena);
    }
}

/// Recursive deep secret check: walks nested tables looking for any
/// secret value. Used by `canaccesstable` (the deep
/// accessibility check) and internally by `table_is_secret`.
pub(super) fn value_is_secret(
    state: &mut LuaState,
    value: Val,
    visited: &mut HashSet<rilua::vm::gc::arena::GcRef<Table>>,
) -> bool {
    if is_secret_value(state, value) || value_has_secret_marker(state, value) {
        return true;
    }

    match value {
        Val::Function(func_ref) => function_is_secret(state, func_ref),
        Val::Table(table_ref) => table_is_secret(state, table_ref, visited),
        _ => false,
    }
}

/// Shallow secret check: only inspects the value itself.
///
/// For tables this does NOT recurse into entry values or mistake ordinary
/// slot taint for a secret value. Used by
/// `issecretvalue`, `canaccessvalue`, and `canaccessallvalues`. The deep walk
/// in `value_is_secret`/`table_is_secret` traverses an entire frame's table
/// tree which is prohibitively expensive in pool Release hot paths
/// (`SecureObjectPoolMixin:CheckAllowReleaseObject` calls `issecretvalue` on
/// every released frame, plus SecureMap/SecureStack assertions on every
/// reclaim).
pub(super) fn value_is_secret_shallow(state: &mut LuaState, value: Val) -> bool {
    if is_secret_value(state, value) || value_has_secret_marker(state, value) {
        return true;
    }

    match value {
        Val::Function(func_ref) => function_is_secret(state, func_ref),
        _ => false,
    }
}

fn value_has_secret_marker(state: &mut LuaState, value: Val) -> bool {
    let Some(key) = secret_registry_key(state, value) else {
        return false;
    };
    let Val::Table(table_ref) = registry_get(state, SECRET_VALUE_REGISTRY_KEY) else {
        return false;
    };
    state.gc.tables.get(table_ref).is_some_and(|table| {
        matches!(
            table.get(key, &state.gc.string_arena),
            Val::Str(marker_ref)
                if state.gc.string_arena.get(marker_ref).is_some_and(|marker| {
                    marker.data() == SECRET_TAINT_MARKER.as_bytes()
                })
        )
    })
}

fn secret_registry_key(state: &mut LuaState, value: Val) -> Option<Val> {
    // Only strings carry persistent value-level taint in WoW's model
    // (unit identity names are the canonical case). Tables/functions/userdata
    // pass through pools and method dispatch and must not be permanently
    // marked — frames in this simulator are Val::Table, so marking them
    // poisons pool reuse via SecureMap's not-secret assertions.
    let Val::Str(value_ref) = value else {
        return None;
    };
    let key = format!("str:{}", value_ref.index());
    Some(Val::Str(state.gc.intern_string(key.as_bytes())))
}

fn get_or_create_secret_value_table(state: &mut LuaState) -> rilua::vm::gc::arena::GcRef<Table> {
    if let Val::Table(table_ref) = registry_get(state, SECRET_VALUE_REGISTRY_KEY) {
        return table_ref;
    }
    let table_ref = state.gc.alloc_table(Table::new());
    registry_set(state, SECRET_VALUE_REGISTRY_KEY, Val::Table(table_ref));
    table_ref
}

fn function_is_secret(
    state: &mut LuaState,
    func_ref: rilua::vm::gc::arena::GcRef<rilua::vm::closure::Closure>,
) -> bool {
    let Val::Table(taint_table_ref) = registry_get(state, "__closure_taint") else {
        return false;
    };
    let Some(table) = state.gc.tables.get(taint_table_ref) else {
        return false;
    };
    let taint = table.get(Val::Num(func_ref.index() as f64), &state.gc.string_arena);
    matches!(taint, Val::Str(taint_ref) if string_matches(state, taint_ref, LOADSTRING_SECRET_TAINT_MARKER))
}

fn string_matches(
    state: &LuaState,
    string_ref: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
    expected: &str,
) -> bool {
    state
        .gc
        .string_arena
        .get(string_ref)
        .is_some_and(|value| value.data() == expected.as_bytes())
}

pub(super) fn table_is_secret(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    visited: &mut HashSet<rilua::vm::gc::arena::GcRef<Table>>,
) -> bool {
    if !visited.insert(table_ref) {
        return false;
    }

    let Some(table) = state.gc.tables.get(table_ref) else {
        return false;
    };

    let mut entries = Vec::new();
    for (index, value) in table.array_slice().iter().copied().enumerate() {
        if !value.is_nil() {
            entries.push((Val::Num((index + 1) as f64), value));
        }
    }
    entries.extend(table.hash_entries());
    entries.into_iter().any(|(key, value)| {
        value_is_secret(state, key, visited) || value_is_secret(state, value, visited)
    })
}
