//! Shared timer callback dispatch for host ticks and nested admin clock steps.
use super::env::WowLuaAppData;
use super::methods::{borrow_state_mut, call_function_state, table_get};
use super::script_helpers::call_error_handler_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::time::Instant;

pub(crate) fn fire(
    state: &mut LuaState,
    owner: Option<u16>,
    callback: Val,
    id: u64,
    receives_timer: bool,
    arg: Option<Val>,
) {
    let started = Instant::now();
    let (source, addon, previous) = enter_callback_owner(state, owner, id);
    if let Some(source) = &source {
        source.log_entry(addon.as_deref(), id);
    }
    let args = callback_arguments(state, id, receives_timer, arg);
    let result = invoke_timer_callback(state, callback, &args);
    let failed = result.is_err();
    if let Err(error) = result {
        call_error_handler_state(state, &error.to_string());
    }
    borrow_state_mut(state)
        .expect("timer callback requires simulator state")
        .executing_addon_index = previous;
    if let Some(source) = source {
        source.log_return(addon.as_deref(), id, started.elapsed(), failed);
    }
    let app = state
        .app_data::<WowLuaAppData>()
        .expect("timer callback requires simulator app data");
    super::env_init::record_addon_time(&app.sim_state, owner, &started);
}

fn invoke_timer_callback(state: &mut LuaState, callback: Val, args: &[Val]) -> LuaResult<Val> {
    let (function, container) = resolve_timer_callback(state, callback);
    if let Some(container) = container {
        let Some((_, backing, _)) = super::timer_layout::function_container_data(state, container)
        else {
            return Err(runtime_error("timer callback container is unavailable"));
        };
        if table_get(state, backing, "_cancelled") == Val::Bool(true) {
            return Ok(Val::Nil);
        }
    }
    call_function_state(state, function, args)
}

fn resolve_timer_callback(state: &mut LuaState, callback: Val) -> (Val, Option<Val>) {
    if !matches!(callback, Val::Table(_)) {
        return (callback, None);
    }
    let function = table_get(
        state,
        callback,
        super::timer_layout::TIMER_CONTAINER_FUNCTION,
    );
    let container = table_get(state, callback, super::timer_layout::TIMER_CONTAINER_OBJECT);
    if matches!(function, Val::Function(_)) && matches!(container, Val::Userdata(_)) {
        return (function, Some(container));
    }
    (callback, None)
}

fn enter_callback_owner(
    state: &LuaState,
    owner: Option<u16>,
    id: u64,
) -> (
    Option<super::handler_timing::TimerCallbackSource>,
    Option<String>,
    Option<u16>,
) {
    let mut sim = borrow_state_mut(state).expect("timer callback requires simulator state");
    let source = super::handler_timing::is_enabled()
        .then(|| sim.timer_callback_sources.get(&id).cloned())
        .flatten();
    let addon = source
        .as_ref()
        .and_then(|_| owner)
        .and_then(|index| sim.addons.get(index as usize))
        .map(|addon| addon.folder_name.clone());
    let previous = sim.executing_addon_index;
    sim.executing_addon_index = owner;
    (source, addon, previous)
}

fn callback_arguments(
    state: &mut LuaState,
    id: u64,
    receives_timer: bool,
    arg: Option<Val>,
) -> Vec<Val> {
    if let Some(arg) = arg {
        return vec![arg];
    }
    if !receives_timer {
        return Vec::new();
    }
    match super::timer_layout::create_timer_handle_table(state, id) {
        Ok(handle) => vec![handle],
        Err(error) => {
            call_error_handler_state(state, &error.to_string());
            Vec::new()
        }
    }
}
