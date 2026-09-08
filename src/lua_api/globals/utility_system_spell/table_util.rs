//! C_TableUtil and tInvert implementations.

use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaError, LuaResult, RuntimeError, Val};

/// tInvert(t) — return a new table with keys/values swapped.
///
/// `tInvert(tbl)` — return `{[v] = k for k, v in pairs(tbl)}`.
///
/// Mirrors Blizzard_SharedXMLBase/TableUtil.lua. Implemented in Rust
/// because the stub version (which pushed `nil`) shadows the Lua
/// definition — `EnumUtil.MakeEnum(...)` goes through this and every
/// downstream enum (ObjectiveTrackerModuleState, etc.) ends up nil
/// when the stub wins.
pub fn t_invert(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    let Val::Table(tbl_ref) = input else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let (array_values, hash_entries) = collect_table_entries(state, tbl_ref);
    let inverted_ref = state.gc.alloc_table(Table::new());
    insert_inverted_array(state, inverted_ref, array_values);
    insert_inverted_hash(state, inverted_ref, hash_entries);
    state.push(Val::Table(inverted_ref));
    Ok(1)
}

fn collect_table_entries(
    state: &LuaState,
    tbl_ref: rilua::vm::gc::arena::GcRef<Table>,
) -> (Vec<Val>, Vec<(Val, Val)>) {
    let array_values = state
        .gc
        .tables
        .get(tbl_ref)
        .map(|t| t.array_slice().to_vec())
        .unwrap_or_default();
    let hash_entries = state
        .gc
        .tables
        .get(tbl_ref)
        .map(|t| t.hash_entries())
        .unwrap_or_default();
    (array_values, hash_entries)
}

fn insert_inverted_array(
    state: &mut LuaState,
    inverted_ref: rilua::vm::gc::arena::GcRef<Table>,
    array_values: Vec<Val>,
) {
    for (index, value) in array_values.into_iter().enumerate() {
        if matches!(value, Val::Nil) {
            continue;
        }
        let key = Val::Num((index + 1) as f64);
        if let Some(t) = state.gc.tables.get_mut(inverted_ref) {
            let _ = t.raw_set(value, key, &state.gc.string_arena);
        }
        state.gc.barrier_back(inverted_ref);
    }
}

fn insert_inverted_hash(
    state: &mut LuaState,
    inverted_ref: rilua::vm::gc::arena::GcRef<Table>,
    hash_entries: Vec<(Val, Val)>,
) {
    for (key, value) in hash_entries {
        if matches!(value, Val::Nil) {
            continue;
        }
        if let Some(t) = state.gc.tables.get_mut(inverted_ref) {
            let _ = t.raw_set(value, key, &state.gc.string_arena);
        }
        state.gc.barrier_back(inverted_ref);
    }
}

/// TableUtil.FindIndexedMismatch(left, right [, comparator]) — first index where arrays differ.
pub fn table_util_find_indexed_mismatch(state: &mut LuaState) -> LuaResult<u32> {
    let left = stack_val(state, 1);
    let right = stack_val(state, 2);
    let comparator = stack_val(state, 3);
    let (Val::Table(left_ref), Val::Table(right_ref)) = (left, right) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let left_values = copy_array_slice(state, left_ref);
    let right_values = copy_array_slice(state, right_ref);
    let mismatch = find_first_mismatch(state, &left_values, &right_values, comparator)?;
    match mismatch {
        Some(index) => state.push(Val::Num(index as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

/// table.create(arrayCapacity, hashCapacity?) — return an empty preallocated table.
pub fn table_create(state: &mut LuaState) -> LuaResult<u32> {
    let array_hint = stack_val(state, 1);
    if matches!(array_hint, Val::Nil) {
        return Err(rilua::runtime_error(
            "bad argument #1 to 'create' (number expected, got nil)",
        ));
    }
    let array_capacity = table_capacity(array_hint);
    let hash_capacity = table_capacity(stack_val(state, 2));
    let table_ref = state
        .gc
        .alloc_table(Table::with_sizes(array_capacity, hash_capacity));
    state.push(Val::Table(table_ref));
    Ok(1)
}

fn table_capacity(value: Val) -> usize {
    let Val::Num(capacity) = value else {
        return 0;
    };
    if !capacity.is_finite() || capacity <= 0.0 {
        return 0;
    }
    capacity.floor() as usize
}

/// table.count(tbl) — count total entries and array-index entries.
pub fn table_count(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    if matches!(input, Val::Nil) {
        return Err(table_count_nil_error());
    }

    let Val::Table(table_ref) = input else {
        return Ok(0);
    };

    let counts = count_table_entries(state, table_ref)?;
    state.push(Val::Num(counts.total_nodes as f64));
    state.push(Val::Num(counts.array_nodes as f64));
    state.push(Val::Num(counts.max_array_index as f64));
    Ok(3)
}

#[cfg(feature = "retail-12-1-5")]
fn table_getcountinfo(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    if !matches!(input, Val::Table(_)) {
        return Err(rilua::runtime_error(format!(
            "bad argument #1 to 'getcountinfo' (table expected, got {})",
            input.type_name()
        )));
    }
    table_count(state)
}

fn table_count_nil_error() -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: "bad argument #1 to 'count' (table expected, got nil)".into(),
        level: 0,
        traceback: vec![],
    })
}

#[derive(Default)]
struct TableCounts {
    total_nodes: usize,
    array_nodes: usize,
    max_array_index: usize,
}

fn count_table_entries(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
) -> LuaResult<TableCounts> {
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Ok(TableCounts::default());
    };

    let mut counts = TableCounts::default();
    let mut key = Val::Nil;
    while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
        counts.total_nodes += 1;
        count_array_key(next_key, &mut counts);
        key = next_key;
    }
    Ok(counts)
}

fn count_array_key(key: Val, counts: &mut TableCounts) {
    let Val::Num(number) = key else {
        return;
    };
    let array_index = number as usize;
    if array_index == 0 || array_index as f64 != number {
        return;
    }
    counts.array_nodes += 1;
    counts.max_array_index = counts.max_array_index.max(array_index);
}

fn copy_array_slice(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> Vec<Val> {
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default()
}

fn find_first_mismatch(
    state: &mut LuaState,
    left: &[Val],
    right: &[Val],
    comparator: Val,
) -> LuaResult<Option<usize>> {
    let count = left.len().max(right.len());
    for index in 0..count {
        let left_val = left.get(index).copied().unwrap_or(Val::Nil);
        let right_val = right.get(index).copied().unwrap_or(Val::Nil);
        let equal = if matches!(comparator, Val::Function(_)) {
            call_table_util_comparator(state, comparator, left_val, right_val, index + 1)?
        } else {
            left_val == right_val
        };
        if !equal {
            return Ok(Some(index + 1));
        }
    }
    Ok(None)
}

fn call_table_util_comparator(
    state: &mut LuaState,
    comparator: Val,
    left: Val,
    right: Val,
    index: usize,
) -> LuaResult<bool> {
    let call_base = state.top;
    state.ensure_stack(call_base + 5);
    state.stack_set(call_base, comparator);
    state.stack_set(call_base + 1, left);
    state.stack_set(call_base + 2, right);
    state.stack_set(call_base + 3, Val::Num(index as f64));
    state.top = call_base + 4;
    state.call_function(call_base, 1)?;
    let result = state.stack_get(call_base);
    state.top = call_base;
    Ok(!matches!(result, Val::Nil | Val::Bool(false)))
}

pub fn register_table_util(state: &mut LuaState) -> LuaResult<()> {
    register_table_library_extensions(state)?;

    let table_ref = state.gc.alloc_table(Table::new());
    #[cfg(not(feature = "retail-12-1-5"))]
    table_set_rust_fn_static(
        state,
        table_ref,
        "FindIndexedMismatch",
        table_util_find_indexed_mismatch,
    )?;
    #[cfg(feature = "retail-12-1-5")]
    {
        use crate::lua_api::methods::table_set_static;
        let removed = Val::Table(state.gc.alloc_table(Table::new()));
        table_set_static(state, removed, "FindIndexedMismatch", Val::Bool(true));
        table_set_static(state, Val::Table(table_ref), "__wow_removed_keys", removed);
    }
    let key_ref = state.gc.intern_string(b"C_TableUtil");
    let global_ref = state.global;
    if let Some(global) = state.gc.tables.get_mut(global_ref) {
        let _ = global.raw_set(
            Val::Str(key_ref),
            Val::Table(table_ref),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(global_ref);
    Ok(())
}

fn register_table_library_extensions(state: &mut LuaState) -> LuaResult<()> {
    let table_library = table_get_static(state, Val::Table(state.global), "table");
    let Val::Table(table_ref) = table_library else {
        return Ok(());
    };
    table_set_rust_fn_static(state, table_ref, "count", table_count)?;
    #[cfg(feature = "retail-12-1-5")]
    table_set_rust_fn_static(state, table_ref, "getcountinfo", table_getcountinfo)?;
    table_set_rust_fn_static(state, table_ref, "create", table_create)
}
