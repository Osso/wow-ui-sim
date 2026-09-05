//! Native `table` extensions published by retail 12.1.5.
//!
//! `Blizzard_SharedXMLBase/TableUtil.lua` aliases this startup-critical subset
//! before other Blizzard modules can use it.

use crate::lua_api::methods::create_table_with_capacity;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let table = table_library(state)?;
    table_set_rust_fn_static(state, table, "indexof", index_of)?;
    table_set_rust_fn_static(state, table, "contains", contains)?;
    table_set_rust_fn_static(state, table, "count", count)?;
    table_set_rust_fn_static(state, table, "isempty", is_empty)?;
    table_set_rust_fn_static(state, table, "removeunordered", remove_unordered)?;
    table_set_rust_fn_static(state, table, "removevalue", remove_value)?;
    table_set_rust_fn_static(state, table, "keys", keys)?;
    table_set_rust_fn_static(state, table, "values", values)?;
    Ok(())
}

fn table_library(state: &mut LuaState) -> LuaResult<GcRef<Table>> {
    let key = state.gc.intern_string_static(b"table");
    let Some(globals) = state.gc.tables.get(state.global) else {
        return Err(runtime_error("global table has been collected"));
    };
    let Val::Table(table) = globals.get_str(key, &state.gc.string_arena) else {
        return Err(runtime_error("table library is unavailable"));
    };
    Ok(table)
}

fn argument_table(state: &LuaState, index: i32) -> LuaResult<GcRef<Table>> {
    let value = stack_val(state, index);
    let Val::Table(table) = value else {
        return Err(runtime_error(format!(
            "bad argument #{index} (table expected, got {})",
            value.type_name()
        )));
    };
    Ok(table)
}

fn table_len(state: &LuaState, table: GcRef<Table>) -> LuaResult<usize> {
    state
        .gc
        .tables
        .get(table)
        .map(|table| table.len(&state.gc.string_arena))
        .ok_or_else(|| runtime_error("table has been collected"))
}

fn table_value(state: &LuaState, table: GcRef<Table>, index: usize) -> LuaResult<Val> {
    state
        .gc
        .tables
        .get(table)
        .map(|table| table.get_int(index as i64))
        .ok_or_else(|| runtime_error("table has been collected"))
}

fn set_table_value(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: Val,
    value: Val,
) -> LuaResult<()> {
    let table_ref = table;
    let Some(table) = state.gc.tables.get_mut(table_ref) else {
        return Err(runtime_error("table has been collected"));
    };
    table.raw_set(key, value, &state.gc.string_arena)?;
    state.gc.barrier_back(table_ref);
    Ok(())
}

fn optional_index(state: &LuaState, default: usize) -> LuaResult<usize> {
    match stack_val(state, 2) {
        Val::Nil => Ok(default),
        Val::Num(index) if index.is_finite() && index.fract() == 0.0 && index >= 1.0 => {
            Ok(index as usize)
        }
        Val::Num(_) => Ok(0),
        value => Err(runtime_error(format!(
            "bad argument #2 (number expected, got {})",
            value.type_name()
        ))),
    }
}

fn index_of(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    let target = stack_val(state, 2);
    for index in 1.. {
        let value = table_value(state, table, index)?;
        if value.is_nil() {
            state.push(Val::Nil);
            return Ok(1);
        }
        if value == target {
            state.push(Val::Num(index as f64));
            return Ok(1);
        }
    }
    unreachable!("unbounded loop returns at first nil")
}

fn contains(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    let target = stack_val(state, 2);
    let mut key = Val::Nil;
    loop {
        let next = state
            .gc
            .tables
            .get(table)
            .ok_or_else(|| runtime_error("table has been collected"))?
            .next(key, &state.gc.string_arena)?;
        let Some((next_key, value)) = next else {
            state.push(Val::Bool(false));
            return Ok(1);
        };
        if value == target {
            state.push(Val::Bool(true));
            return Ok(1);
        }
        key = next_key;
    }
}

fn count_entries(state: &LuaState, table: GcRef<Table>) -> LuaResult<usize> {
    let mut count = 0;
    let mut key = Val::Nil;
    loop {
        let next = state
            .gc
            .tables
            .get(table)
            .ok_or_else(|| runtime_error("table has been collected"))?
            .next(key, &state.gc.string_arena)?;
        let Some((next_key, _)) = next else {
            return Ok(count);
        };
        count += 1;
        key = next_key;
    }
}

fn count(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    state.push(Val::Num(count_entries(state, table)? as f64));
    Ok(1)
}

fn is_empty(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    let empty = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error("table has been collected"))?
        .next(Val::Nil, &state.gc.string_arena)?
        .is_none();
    state.push(Val::Bool(empty));
    Ok(1)
}

fn remove_unordered(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    let length = table_len(state, table)?;
    let index = optional_index(state, length)?;
    if index == 0 || index > length {
        state.push(Val::Nil);
        return Ok(1);
    }
    let removed = table_value(state, table, index)?;
    let last = table_value(state, table, length)?;
    if index != length {
        set_table_value(state, table, Val::Num(index as f64), last)?;
    }
    set_table_value(state, table, Val::Num(length as f64), Val::Nil)?;
    state.push(removed);
    Ok(1)
}

fn remove_value(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    let target = stack_val(state, 2);
    let length = table_len(state, table)?;
    let values = (1..=length)
        .map(|index| table_value(state, table, index))
        .collect::<LuaResult<Vec<_>>>()?;
    let retained: Vec<_> = values
        .into_iter()
        .filter(|value| *value != target)
        .collect();
    for (index, value) in retained.iter().copied().enumerate() {
        set_table_value(state, table, Val::Num((index + 1) as f64), value)?;
    }
    for index in retained.len() + 1..=length {
        set_table_value(state, table, Val::Num(index as f64), Val::Nil)?;
    }
    state.push(Val::Num((length - retained.len()) as f64));
    Ok(1)
}

fn keys_or_values(state: &mut LuaState, use_keys: bool) -> LuaResult<u32> {
    let source = argument_table(state, 1)?;
    let destination = match create_table_with_capacity(state, count_entries(state, source)?) {
        Val::Table(table) => table,
        _ => unreachable!("table factory returns a table"),
    };
    let mut source_key = Val::Nil;
    let mut destination_index = 1;
    loop {
        let next = state
            .gc
            .tables
            .get(source)
            .ok_or_else(|| runtime_error("table has been collected"))?
            .next(source_key, &state.gc.string_arena)?;
        let Some((next_key, value)) = next else {
            state.push(Val::Table(destination));
            return Ok(1);
        };
        let output = if use_keys { next_key } else { value };
        set_table_value(
            state,
            destination,
            Val::Num(destination_index as f64),
            output,
        )?;
        destination_index += 1;
        source_key = next_key;
    }
}

fn keys(state: &mut LuaState) -> LuaResult<u32> {
    keys_or_values(state, true)
}

fn values(state: &mut LuaState) -> LuaResult<u32> {
    keys_or_values(state, false)
}
