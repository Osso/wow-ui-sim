//! `C_Timer.NewTimedSignalMap` backed by simulator timer state.

use crate::c_api::ensure_namespace;
use crate::lua_api::WowLuaAppData;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::vm::value::{Userdata, Val};
use rilua::{LuaResult, runtime_error};
use std::collections::BTreeMap;

const METATABLE: &str = "TimedSignalMap";

#[derive(Debug)]
struct TimedSignalMap {
    id: u64,
}

#[derive(Debug, Default)]
pub(crate) struct TimedSignalMapState {
    pub(crate) signals: BTreeMap<i64, f64>,
    pub(crate) owner_addon: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DueTimedSignal {
    pub(crate) id: u64,
    pub(crate) owner_addon: Option<u16>,
    pub(crate) key: i64,
}

pub(crate) fn register_c_timer_timed_signal_map(state: &mut LuaState) -> LuaResult<()> {
    let c_timer = ensure_namespace(state, "C_Timer")?;
    table_set_rust_fn_static(state, c_timer, "NewTimedSignalMap", new_timed_signal_map)
}

pub(crate) fn new_timed_signal_map(state: &mut LuaState) -> LuaResult<u32> {
    let callback = stack_val(state, 1);
    if !matches!(callback, Val::Function(_)) {
        return Err(runtime_error(
            "bad argument #1 to 'NewTimedSignalMap' (function expected)",
        ));
    }
    let id = crate::lua_api::next_timer_id();
    crate::lua_api::timer_layout::store_timer_callback(state, id, callback);
    let mut sim = app_state(state)?;
    let owner_addon = sim.loading_addon_index.or(sim.executing_addon_index);
    sim.timed_signal_maps.insert(
        id,
        TimedSignalMapState {
            signals: BTreeMap::new(),
            owner_addon,
        },
    );
    drop(sim);

    let metatable = metatable(state)?;
    let environment = state.gc.alloc_table(Table::new());
    let mut userdata = Userdata::with_metatable(Box::new(TimedSignalMap { id }), metatable);
    userdata.set_env(Some(environment));
    let reference = state.gc.alloc_userdata(userdata);
    state.push(Val::Userdata(reference));
    Ok(1)
}

fn metatable(state: &mut LuaState) -> LuaResult<rilua::vm::gc::arena::GcRef<Table>> {
    let metatable = rilua::stdlib::new_metatable(state, METATABLE)?;
    let signal_at_key = state.gc.intern_string_static(b"SignalAt");
    if state.gc.tables.get(metatable).is_some_and(|table| {
        !matches!(
            table.get_str(signal_at_key, &state.gc.string_arena),
            Val::Nil
        )
    }) {
        return Ok(metatable);
    }
    table_set_rust_fn_static(state, metatable, "__index", index)?;
    table_set_rust_fn_static(state, metatable, "__newindex", newindex)?;
    table_set_rust_fn_static(state, metatable, "__gc", finalize)?;
    table_set_rust_fn_static(state, metatable, "CancelAllSignals", cancel_all_signals)?;
    table_set_rust_fn_static(state, metatable, "CancelSignal", cancel_signal)?;
    table_set_rust_fn_static(state, metatable, "GetNextSignal", get_next_signal)?;
    table_set_rust_fn_static(state, metatable, "GetSignalCount", get_signal_count)?;
    table_set_rust_fn_static(state, metatable, "GetSignalTime", get_signal_time)?;
    table_set_rust_fn_static(state, metatable, "HasSignal", has_signal)?;
    table_set_rust_fn_static(state, metatable, "SignalAfter", signal_after)?;
    table_set_rust_fn_static(state, metatable, "SignalAt", signal_at)?;
    Ok(metatable)
}

fn map_id(state: &LuaState) -> LuaResult<u64> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error(
            "TimedSignalMap method expected userdata self",
        ));
    };
    state
        .gc
        .userdata
        .get(reference)
        .and_then(|userdata| userdata.downcast_ref::<TimedSignalMap>())
        .map(|map| map.id)
        .ok_or_else(|| runtime_error("TimedSignalMap method called on incompatible userdata"))
}

fn app_state(state: &LuaState) -> LuaResult<std::cell::RefMut<'_, crate::lua_api::SimState>> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    Ok(app.sim_state.borrow_mut())
}

fn signal_key(state: &LuaState, index: i32) -> LuaResult<i64> {
    let Val::Num(key) = stack_val(state, index) else {
        return Err(runtime_error(format!(
            "bad argument #{index} (number expected)"
        )));
    };
    if !key.is_finite() || key.fract() != 0.0 || key < i64::MIN as f64 || key > i64::MAX as f64 {
        return Err(runtime_error(format!(
            "bad argument #{index} (integer expected)"
        )));
    }
    Ok(key as i64)
}

fn signal_time(state: &LuaState, index: i32) -> LuaResult<f64> {
    let Val::Num(time) = stack_val(state, index) else {
        return Err(runtime_error(format!(
            "bad argument #{index} (number expected)"
        )));
    };
    if !time.is_finite() {
        return Err(runtime_error(format!(
            "bad argument #{index} (finite number expected)"
        )));
    }
    Ok(time)
}

fn map_state<'a>(
    state: &'a mut crate::lua_api::SimState,
    id: u64,
) -> LuaResult<&'a mut TimedSignalMapState> {
    state
        .timed_signal_maps
        .get_mut(&id)
        .ok_or_else(|| runtime_error("TimedSignalMap state is unavailable"))
}

fn index(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error(
            "TimedSignalMap __index expected userdata self",
        ));
    };
    let key = stack_val(state, 2);
    let userdata = state
        .gc
        .userdata
        .get(reference)
        .ok_or_else(|| runtime_error("collected TimedSignalMap"))?;
    let metatable = userdata
        .metatable()
        .ok_or_else(|| runtime_error("TimedSignalMap metatable is unavailable"))?;
    let method = state
        .gc
        .tables
        .get(metatable)
        .map(|table| table.get(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if !matches!(method, Val::Nil) {
        state.push(method);
        return Ok(1);
    }
    let value = userdata
        .env()
        .and_then(|environment| state.gc.tables.get(environment))
        .map(|table| table.get(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    state.push(value);
    Ok(1)
}

fn newindex(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error(
            "TimedSignalMap __newindex expected userdata self",
        ));
    };
    let key = stack_val(state, 2);
    let value = stack_val(state, 3);
    let userdata = state
        .gc
        .userdata
        .get_mut(reference)
        .ok_or_else(|| runtime_error("collected TimedSignalMap"))?;
    let environment = userdata
        .env()
        .ok_or_else(|| runtime_error("TimedSignalMap environment is unavailable"))?;
    state
        .gc
        .tables
        .get_mut(environment)
        .ok_or_else(|| runtime_error("collected TimedSignalMap environment"))?
        .raw_set(key, value, &state.gc.string_arena)?;
    state.gc.barrier_back(environment);
    Ok(0)
}

fn finalize(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    {
        let mut sim = app_state(state)?;
        sim.timed_signal_maps.remove(&id);
    }
    crate::lua_api::timer_layout::remove_timer_callback(state, id);
    Ok(0)
}

fn cancel_all_signals(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let mut sim = app_state(state)?;
    map_state(&mut sim, id)?.signals.clear();
    Ok(0)
}

fn cancel_signal(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let key = signal_key(state, 2)?;
    let mut sim = app_state(state)?;
    map_state(&mut sim, id)?.signals.remove(&key);
    Ok(0)
}

fn get_next_signal(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let next = {
        let mut sim = app_state(state)?;
        map_state(&mut sim, id)?
            .signals
            .iter()
            .min_by(|left, right| left.1.total_cmp(right.1).then_with(|| left.0.cmp(right.0)))
            .map(|(key, time)| (*key, *time))
    };
    if let Some((key, time)) = next {
        state.push(Val::Num(key as f64));
        state.push(Val::Num(time));
        Ok(2)
    } else {
        state.push(Val::Nil);
        Ok(1)
    }
}

fn get_signal_count(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let count = {
        let mut sim = app_state(state)?;
        map_state(&mut sim, id)?.signals.len()
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_signal_time(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let key = signal_key(state, 2)?;
    let time = {
        let mut sim = app_state(state)?;
        map_state(&mut sim, id)?.signals.get(&key).copied()
    };
    state.push(time.map_or(Val::Nil, Val::Num));
    Ok(1)
}

fn has_signal(state: &mut LuaState) -> LuaResult<u32> {
    let id = map_id(state)?;
    let key = signal_key(state, 2)?;
    let present = {
        let mut sim = app_state(state)?;
        map_state(&mut sim, id)?.signals.contains_key(&key)
    };
    state.push(Val::Bool(present));
    Ok(1)
}

fn signal_after(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = signal_time(state, 3)?;
    let now = app_state(state)?.start_time.elapsed().as_secs_f64();
    signal_at_values(state, signal_key(state, 2)?, now + seconds)
}

fn signal_at(state: &mut LuaState) -> LuaResult<u32> {
    signal_at_values(state, signal_key(state, 2)?, signal_time(state, 3)?)
}

fn signal_at_values(state: &mut LuaState, key: i64, time: f64) -> LuaResult<u32> {
    let id = map_id(state)?;
    let mut sim = app_state(state)?;
    map_state(&mut sim, id)?.signals.insert(key, time);
    Ok(0)
}

pub(crate) fn take_due_signals(state: &mut crate::lua_api::SimState) -> Vec<DueTimedSignal> {
    let now = state.start_time.elapsed().as_secs_f64();
    let mut due = Vec::new();
    for (id, map) in &mut state.timed_signal_maps {
        let keys: Vec<_> = map
            .signals
            .iter()
            .filter_map(|(key, time)| (*time <= now).then_some(*key))
            .collect();
        for key in keys {
            map.signals.remove(&key);
            due.push(DueTimedSignal {
                id: *id,
                owner_addon: map.owner_addon,
                key,
            });
        }
    }
    due
}

pub(crate) fn next_signal_delay(state: &crate::lua_api::SimState) -> Option<std::time::Duration> {
    let now = state.start_time.elapsed().as_secs_f64();
    state
        .timed_signal_maps
        .values()
        .flat_map(|map| map.signals.values())
        .map(|time| std::time::Duration::from_secs_f64((*time - now).max(0.0)))
        .min()
}
