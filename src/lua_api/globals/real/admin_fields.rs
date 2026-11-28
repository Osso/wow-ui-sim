//! Typed field readers for A_Admin info tables. Every reader fails with
//! "<api> <key> must be ..." so host fixtures surface invalid inputs at the call site.

use crate::lua_api::methods::{table_get, val_to_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn invalid(api: &str, key: &str, expected: &str) -> rilua::LuaError {
    runtime_error(format!("{api} {key} must be {expected}"))
}

pub(crate) fn finite(state: &mut LuaState, table: Val, api: &str, key: &str) -> LuaResult<f64> {
    match table_get(state, table, key) {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(invalid(api, key, "a finite number")),
    }
}

pub(crate) fn positive(state: &mut LuaState, table: Val, api: &str, key: &str) -> LuaResult<f64> {
    let number = finite(state, table, api, key)?;
    if number <= 0.0 {
        return Err(invalid(api, key, "positive"));
    }
    Ok(number)
}

pub(crate) fn unsigned(state: &mut LuaState, table: Val, api: &str, key: &str) -> LuaResult<u32> {
    let number = finite(state, table, api, key)?;
    if number.fract() != 0.0 || !(0.0..=f64::from(u32::MAX)).contains(&number) {
        return Err(invalid(api, key, "an unsigned integer"));
    }
    Ok(number as u32)
}

pub(crate) fn boolean(state: &mut LuaState, table: Val, api: &str, key: &str) -> LuaResult<bool> {
    match table_get(state, table, key) {
        Val::Bool(flag) => Ok(flag),
        _ => Err(invalid(api, key, "a boolean")),
    }
}

pub(crate) fn optional_string(
    state: &mut LuaState,
    table: Val,
    api: &str,
    key: &str,
) -> LuaResult<Option<String>> {
    match table_get(state, table, key) {
        Val::Nil => Ok(None),
        value @ Val::Str(_) => Ok(val_to_string(state, value)),
        _ => Err(invalid(api, key, "a string or nil")),
    }
}

pub(crate) fn string(state: &mut LuaState, table: Val, api: &str, key: &str) -> LuaResult<String> {
    optional_string(state, table, api, key)?.ok_or_else(|| invalid(api, key, "a string"))
}

/// The table values of an array field, in order; non-table entries are rejected.
pub(crate) fn table_array(
    state: &mut LuaState,
    table: Val,
    api: &str,
    key: &str,
) -> LuaResult<Vec<Val>> {
    let Val::Table(array) = table_get(state, table, key) else {
        return Err(invalid(api, key, "an array of tables"));
    };
    let mut rows = Vec::new();
    for position in 1.. {
        match state.gc.tables.get(array).map(|t| t.get_int(position)) {
            Some(row @ Val::Table(_)) => rows.push(row),
            Some(Val::Nil) | None => return Ok(rows),
            Some(_) => return Err(invalid(api, key, "an array of tables")),
        }
    }
    Ok(rows)
}
