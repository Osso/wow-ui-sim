//! Environment-local ordinary global event callbacks.

use crate::lua_api::methods::{
    create_table, registry_table_or_create, table_get, table_set, val_to_string,
};
use crate::lua_api::script_helpers::protected_call_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const REGISTRY_KEY: &str = "__global_event_callbacks";

fn arguments(state: &LuaState) -> LuaResult<(String, Val)> {
    let event = stack_val(state, 1);
    let callback = stack_val(state, 2);
    // Simulator policy: nonempty string events and ordinary functions only.
    if !matches!(event, Val::Str(_)) {
        return Err(runtime_error("event name must be a string"));
    }
    let event = val_to_string(state, event).unwrap_or_default();
    if event.is_empty() {
        return Err(runtime_error("event name must not be empty"));
    }
    if !matches!(callback, Val::Function(_)) {
        return Err(runtime_error("event callback must be a function"));
    }
    Ok((event, callback))
}

fn callback_at(state: &LuaState, list: Val, index: usize) -> Val {
    let Val::Table(table) = list else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table)
        .map_or(Val::Nil, |table| table.get_int(index as i64))
}

fn append(state: &mut LuaState, list: Val, index: usize, callback: Val) -> LuaResult<()> {
    let Val::Table(table) = list else {
        return Err(runtime_error("callback list is not a table"));
    };
    state
        .gc
        .tables
        .get_mut(table)
        .ok_or_else(|| runtime_error("callback list was collected"))?
        .raw_set(Val::Num(index as f64), callback, &state.gc.string_arena)?;
    state.gc.barrier_back(table);
    Ok(())
}

fn replace_callbacks(state: &mut LuaState, event: &str, callback: Val, add: bool) -> LuaResult<()> {
    let registry = registry_table_or_create(state, REGISTRY_KEY);
    let old = table_get(state, registry, event);
    let replacement = create_table(state);
    let saved_top = state.top;
    state.push(replacement);
    let result = (|| {
        let mut destination = 1;
        let mut found = false;
        let mut source = 1;
        loop {
            let existing = callback_at(state, old, source);
            if matches!(existing, Val::Nil) {
                break;
            }
            source += 1;
            let same = existing == callback;
            found |= same;
            if add || !same {
                append(state, replacement, destination, existing)?;
                destination += 1;
            }
        }
        // Simulator policy: duplicate registration is idempotent; insertion order wins.
        if add && !found {
            append(state, replacement, destination, callback)?;
        }
        table_set(state, registry, event, replacement);
        Ok(())
    })();
    state.top = saved_top;
    result
}

pub fn register_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let (event, callback) = arguments(state)?;
    replace_callbacks(state, &event, callback, true)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub fn unregister_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let (event, callback) = arguments(state)?;
    replace_callbacks(state, &event, callback, false)?;
    Ok(0)
}

/// Dispatch a rooted immutable snapshot; mutations affect subsequent dispatches.
/// Protected-call errors are returned to the caller for normal error reporting.
pub fn dispatch_event_callbacks(
    state: &mut LuaState,
    event: &str,
    payload: &[Val],
) -> Result<(), Val> {
    let saved_top = state.top;
    // Root payload before any callbacks can allocate or trigger collection.
    for value in payload {
        state.push(*value);
    }
    let registry = registry_table_or_create(state, REGISTRY_KEY);
    let snapshot = table_get(state, registry, event);
    state.push(snapshot);
    let mut args = Vec::with_capacity(payload.len() + 1);
    args.push(Val::Nil);
    args.extend_from_slice(payload);
    let result = (|| {
        let mut index = 1;
        loop {
            let callback = callback_at(state, snapshot, index);
            if matches!(callback, Val::Nil) {
                return Ok(());
            }
            // Simulator policy: stop this dispatch at the first callback error.
            protected_call_state(state, callback, &args)?;
            index += 1;
        }
    })();
    state.top = saved_top;
    result
}
