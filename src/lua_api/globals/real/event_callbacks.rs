//! Environment-local ordinary global event callbacks.

use crate::lua_api::methods::{
    create_table, registry_get, registry_table_or_create, table_get, table_set, val_to_string,
};
use crate::lua_api::script_helpers::protected_call_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const REGISTRY_KEY: &str = "__global_event_callbacks";
const UNIT_REGISTRY_KEY: &str = "__global_unit_event_callbacks";

fn container_invoker(state: &mut LuaState, callback: Val) -> Option<Val> {
    if !matches!(callback, Val::Userdata(_)) {
        return None;
    }
    let bridge = registry_get(state, "__wow_function_containers");
    let Val::Table(objects) = table_get(state, bridge, "objects") else {
        return None;
    };
    let backing = state
        .gc
        .tables
        .get(objects)?
        .get(callback, &state.gc.string_arena);
    if !matches!(backing, Val::Table(_)) {
        return None;
    }
    let invoke = table_get(state, bridge, "invoke");
    matches!(invoke, Val::Function(_)).then_some(invoke)
}

fn arguments(state: &mut LuaState) -> LuaResult<(String, Val)> {
    let event = stack_val(state, 1);
    let callback = stack_val(state, 2);
    // Simulator policy: nonempty events; functions or modeled containers only.
    if !matches!(event, Val::Str(_)) {
        return Err(runtime_error("event name must be a string"));
    }
    let event = val_to_string(state, event).unwrap_or_default();
    if event.is_empty() {
        return Err(runtime_error("event name must not be empty"));
    }
    if !matches!(callback, Val::Function(_)) && container_invoker(state, callback).is_none() {
        return Err(runtime_error(
            "event callback must be a function or FunctionContainer",
        ));
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

fn replace_callbacks(
    state: &mut LuaState,
    registry: Val,
    key: &str,
    callback: Val,
    add: bool,
) -> LuaResult<()> {
    let old = table_get(state, registry, key);
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
        table_set(state, registry, key, replacement);
        Ok(())
    })();
    state.top = saved_top;
    result
}

pub fn register_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let (event, callback) = arguments(state)?;
    let registry = registry_table_or_create(state, REGISTRY_KEY);
    replace_callbacks(state, registry, &event, callback, true)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub fn unregister_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    let (event, callback) = arguments(state)?;
    let registry = registry_table_or_create(state, REGISTRY_KEY);
    replace_callbacks(state, registry, &event, callback, false)?;
    Ok(0)
}

fn update_unit_callback(state: &mut LuaState, add: bool) -> LuaResult<u32> {
    let (event, callback) = arguments(state)?;
    let unit = stack_val(state, 3);
    // Simulator policy: one nonempty unit token, without aliases or lists.
    if !matches!(unit, Val::Str(_)) {
        return Err(runtime_error("unit must be a nonempty string"));
    }
    let unit = val_to_string(state, unit).unwrap_or_default();
    if unit.is_empty() {
        return Err(runtime_error("unit must be a nonempty string"));
    }
    let registry = registry_table_or_create(state, UNIT_REGISTRY_KEY);
    let mut units = table_get(state, registry, &event);
    if !matches!(units, Val::Table(_)) {
        units = create_table(state);
        table_set(state, registry, &event, units);
    }
    replace_callbacks(state, units, &unit, callback, add)?;
    Ok(0)
}

pub fn register_unit_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    update_unit_callback(state, true)
}

pub fn unregister_unit_event_callback(state: &mut LuaState) -> LuaResult<u32> {
    update_unit_callback(state, false)
}

fn matching_unit_callbacks(state: &mut LuaState, event: &str, payload: &[Val]) -> Val {
    let Some(unit @ Val::Str(_)) = payload.first().copied() else {
        return Val::Nil;
    };
    let unit = val_to_string(state, unit).unwrap_or_default();
    let registry = registry_get(state, UNIT_REGISTRY_KEY);
    let units = table_get(state, registry, event);
    // Exact first-payload token matching is simulator policy.
    table_get(state, units, &unit)
}

/// Dispatch rooted immutable snapshots; mutations affect subsequent dispatches.
/// Protected-call errors are returned to the caller for normal error reporting.
pub fn dispatch_event_callbacks(
    state: &mut LuaState,
    event: &str,
    payload: &[Val],
) -> LuaResult<()> {
    let saved_top = state.top;
    // Root payload before any callbacks can allocate or trigger collection.
    for value in payload {
        state.push(*value);
    }
    let registry = registry_table_or_create(state, REGISTRY_KEY);
    let snapshot = table_get(state, registry, event);
    state.push(snapshot);
    let unit_snapshot = matching_unit_callbacks(state, event, payload);
    state.push(unit_snapshot);
    let mut args = Vec::with_capacity(payload.len() + 1);
    args.push(Val::Nil);
    args.extend_from_slice(payload);
    // Simulator policy: ordinary globals precede matching unit callbacks.
    let result = dispatch_snapshot(state, event, snapshot, &args)
        .and_then(|()| dispatch_snapshot(state, event, unit_snapshot, &args));
    state.top = saved_top;
    result
}

fn dispatch_snapshot(
    state: &mut LuaState,
    event: &str,
    snapshot: Val,
    args: &[Val],
) -> LuaResult<()> {
    let mut index = 1;
    loop {
        let callback = callback_at(state, snapshot, index);
        if matches!(callback, Val::Nil) {
            return Ok(());
        }
        let result = if let Some(invoke) = container_invoker(state, callback) {
            let mut container_args = Vec::with_capacity(args.len() + 1);
            container_args.push(callback);
            container_args.extend_from_slice(args);
            protected_call_state(state, invoke, &container_args)
        } else {
            protected_call_state(state, callback, args)
        };
        // Simulator policy: stop this dispatch at the first callback error.
        result.map_err(|error| {
            runtime_error(format!(
                "global callback for {event}: {}",
                val_to_string(state, error)
                    .unwrap_or_else(|| format!("{} error", error.type_name()))
            ))
        })?;
        index += 1;
    }
}
