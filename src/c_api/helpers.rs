use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(crate) fn set_global_val(state: &mut LuaState, name: &str, value: Val) {
    let key = state.gc.intern_string(name.as_bytes());
    let global = state.global;
    if let Some(table) = state.gc.tables.get_mut(global) {
        let _ = table.raw_set(Val::Str(key), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
}

pub(crate) fn global_val(state: &mut LuaState, name: &str) -> Val {
    let key = state.gc.intern_string(name.as_bytes());
    state
        .gc
        .tables
        .get(state.global)
        .map(|table| table.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

pub(crate) fn ensure_global_table(state: &mut LuaState, name: &str) -> Val {
    match global_val(state, name) {
        table @ Val::Table(_) => table,
        _ => {
            let table = crate::lua_api::methods::create_table(state);
            set_global_val(state, name, table);
            table
        }
    }
}

pub(crate) fn ensure_namespace(
    state: &mut LuaState,
    name: &'static str,
) -> LuaResult<GcRef<Table>> {
    let key_ref = state.gc.intern_string_static(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let table_ref = match current {
        Val::Table(table_ref) => table_ref,
        _ => {
            let table_ref = state.gc.alloc_table(Table::new());
            let global = state.global;
            if let Some(globals) = state.gc.tables.get_mut(global) {
                let _ = globals.raw_set(
                    Val::Str(key_ref),
                    Val::Table(table_ref),
                    &state.gc.string_arena,
                );
            }
            state.gc.barrier_back(global);
            table_ref
        }
    };
    Ok(table_ref)
}

/// Keep ordinary namespace lookup from fabricating retired members.
pub(crate) fn mark_namespace_keys_removed(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    names: &[&str],
) {
    use crate::lua_api::methods::{create_table, table_get, table_set};
    let table = Val::Table(namespace);
    let existing = table_get(state, table, "__wow_removed_keys");
    let removed = if matches!(existing, Val::Table(_)) {
        existing
    } else {
        create_table(state)
    };
    for name in names {
        table_set(state, removed, name, Val::Bool(true));
    }
    table_set(state, table, "__wow_removed_keys", removed);
}

pub(crate) fn set_table_array(state: &mut LuaState, table: Val, index: i64, value: Val) {
    let Val::Table(table_ref) = table else { return };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}
