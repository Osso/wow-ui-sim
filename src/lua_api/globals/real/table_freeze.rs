//! Shallow table immutability, independent of rilua's recursive GC freezing.
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

use super::table_extensions::{argument_table, table_library};
use crate::lua_bridge::table_set_rust_fn_static;

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let table = table_library(state)?;
    table_set_rust_fn_static(state, table, "freeze", freeze)?;
    table_set_rust_fn_static(state, table, "isfrozen", is_frozen)?;
    table_set_rust_fn_static(state, table, "insert", insert)?;
    table_set_rust_fn_static(state, table, "remove", remove)?;
    table_set_rust_fn_static(state, table, "sort", sort)?;
    Ok(())
}

fn freeze(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    state
        .gc
        .tables
        .get_mut(table)
        .ok_or_else(|| runtime_error("table has been collected"))?
        .make_read_only();
    if cfg!(feature = "retail-12-1-5") {
        state.push(Val::Table(table));
        Ok(1)
    } else {
        Ok(0)
    }
}

fn is_frozen(state: &mut LuaState) -> LuaResult<u32> {
    let table = argument_table(state, 1)?;
    state.push(Val::Bool(is_table_frozen(state, table)));
    Ok(1)
}

fn is_table_frozen(state: &LuaState, table: GcRef<Table>) -> bool {
    state.gc.tables.is_frozen(table) || state.gc.tables.get(table).is_some_and(Table::is_read_only)
}

pub(crate) fn ensure_mutable(state: &LuaState, table: GcRef<Table>) -> LuaResult<()> {
    if is_table_frozen(state, table) {
        return Err(runtime_error("attempt to modify a frozen table"));
    }
    Ok(())
}

pub(crate) fn insert(state: &mut LuaState) -> LuaResult<u32> {
    ensure_mutable(state, argument_table(state, 1)?)?;
    rilua::stdlib::table::tab_insert(state)
}

pub(crate) fn remove(state: &mut LuaState) -> LuaResult<u32> {
    ensure_mutable(state, argument_table(state, 1)?)?;
    rilua::stdlib::table::tab_remove(state)
}

fn sort(state: &mut LuaState) -> LuaResult<u32> {
    ensure_mutable(state, argument_table(state, 1)?)?;
    rilua::stdlib::table::tab_sort(state)
}
