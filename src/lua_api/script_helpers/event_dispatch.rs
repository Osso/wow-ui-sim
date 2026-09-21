use super::{
    call_error_handler, call_error_handler_state, get_script_handlers_for_dispatch,
    get_scripts_for_dispatch, protected_lua_pcall_state, registry_table, table_get_str,
};
use crate::c_api::on_update_modes::OnUpdateMode;
use crate::lua_api::handler_timing;
use crate::lua_api::methods::{borrow_state, create_string, frame_ref, table_get, val_to_string};
use rilua::vm::closure::Closure;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApi, LuaApiMut, Val};
use std::borrow::Cow;
use std::collections::HashSet;
use std::time::Instant;

const BUILTIN_ADDON_NAME: &str = "__BuiltIn";

/// Get event listeners in registration order from rilua's registry.
pub fn get_event_listeners(state: &mut LuaState, event: &str) -> Vec<u64> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    collect_individual_listeners(state, event, &mut result, &mut seen);
    collect_all_event_listeners(state, &mut result, &mut seen);
    collect_widget_registry_listeners(state, event, &mut result, &mut seen);
    result
}

pub(crate) fn event_matches_unit_filter(
    state: &LuaState,
    widget_id: u64,
    event_name: &str,
    args: &[Val],
) -> rilua::LuaResult<bool> {
    let sim = borrow_state(state)?;
    let filter = sim
        .widgets
        .get(widget_id)
        .and_then(|frame| frame.registered_unit_events.get(event_name));
    Ok(filter.is_none_or(|unit| {
        args.first()
            .and_then(|value| val_to_string(state, *value))
            .as_deref()
            == Some(unit.as_str())
    }))
}

pub fn fire_named_event_state(state: &mut LuaState, event_name: &str, args: &[Val]) {
    for widget_id in get_event_listeners(state, event_name) {
        if let Err(error) = dispatch_named_event_handlers(state, widget_id, event_name, args) {
            call_error_handler_state(state, &error.to_string());
            return;
        }
    }
}

/// Dispatch one frame's filtered precall, normal and postcall event scripts.
/// Per-frame callbacks have their own unit filters and are dispatched separately.
pub(crate) fn dispatch_named_event_handlers(
    state: &mut LuaState,
    widget_id: u64,
    event_name: &str,
    args: &[Val],
) -> rilua::LuaResult<()> {
    if !event_matches_unit_filter(state, widget_id, event_name, args)? {
        return Ok(());
    }
    let handlers = get_scripts_for_dispatch(state, widget_id, "OnEvent");
    if handlers.is_empty() {
        return Ok(());
    }
    let Ok(frame) = frame_ref(state, widget_id) else {
        return Ok(());
    };
    let event_name_val = create_string(state, event_name);
    let mut call_args = Vec::with_capacity(args.len() + 2);
    call_args.push(frame);
    call_args.push(event_name_val);
    call_args.extend_from_slice(args);
    for handler in handlers {
        if !matches!(handler, Val::Function(_)) {
            continue;
        }
        if let Err(error) = protected_lua_pcall_state(state, handler, &call_args) {
            call_error_handler_state(state, &error);
        }
    }
    Ok(())
}

fn collect_individual_listeners(
    state: &mut LuaState,
    event: &str,
    result: &mut Vec<u64>,
    seen: &mut HashSet<u64>,
) {
    let event_tbl = resolve_event_subtable(state, "__event_individual", event);
    let Some(tbl) = event_tbl.and_then(|r| state.gc.tables.get(r)) else {
        return;
    };
    let slice = tbl.array_slice();
    for val in slice {
        if let Val::Num(id) = val {
            let id = *id as u64;
            result.push(id);
            seen.insert(id);
        }
    }
}

fn collect_all_event_listeners(
    state: &mut LuaState,
    result: &mut Vec<u64>,
    seen: &mut HashSet<u64>,
) {
    let Some(all_ref) = registry_table(state, "__event_all") else {
        return;
    };
    let Some(all) = state.gc.tables.get(all_ref) else {
        return;
    };
    let slice = all.array_slice();
    for val in slice {
        if let Val::Num(id) = val {
            let id = *id as u64;
            if seen.insert(id) {
                result.push(id);
            }
        }
    }
}

fn collect_widget_registry_listeners(
    state: &LuaState,
    event: &str,
    result: &mut Vec<u64>,
    seen: &mut HashSet<u64>,
) {
    use crate::lua_api::env::WowLuaAppData;

    let Some(app) = state.app_data::<WowLuaAppData>() else {
        return;
    };
    let Ok(sim) = app.sim_state.try_borrow() else {
        return;
    };
    for id in sim.widgets.get_event_listeners(event) {
        if seen.insert(id) {
            result.push(id);
        }
    }
}

fn resolve_event_subtable(
    state: &mut LuaState,
    registry_key: &'static str,
    event: &str,
) -> Option<GcRef<Table>> {
    let container_ref = registry_table(state, registry_key)?;
    match table_get_str(state, container_ref, event) {
        Val::Table(t) => Some(t),
        _ => None,
    }
}

/// Dispatch a script handler for a frame via rilua.
///
/// Looks up the script handler in the rilua registry and calls it with the
/// frame value as the first argument, followed by any additional args.
///
/// This is the rilua equivalent of calling `get_script` + `handler.call(frame_val)`.
pub fn dispatch_script(
    lua: &mut rilua::Lua,
    widget_id: u64,
    handler_name: &str,
    extra_args: &[Val],
) -> rilua::LuaResult<()> {
    let handlers = {
        let state = lua.state_mut();
        get_scripts_for_dispatch(state, widget_id, handler_name)
    };
    if handlers.is_empty() {
        return Ok(());
    }

    let frame_val = {
        let state = lua.state_mut();
        frame_ref(state, widget_id)?
    };
    let mut args = vec![frame_val];
    args.extend_from_slice(extra_args);

    for handler_val in handlers {
        let Val::Function(func_ref) = handler_val else {
            continue;
        };
        let (owner_addon, _, _) = handler_log_metadata(lua.state(), widget_id);
        let func = rilua::Function::from_gc_ref(func_ref);
        let previous_addon = replace_executing_addon(lua.state(), owner_addon);
        if let Err(e) = lua.call_function(&func, &args) {
            call_error_handler(lua, &e.to_string());
        }
        replace_executing_addon(lua.state(), previous_addon);
    }
    Ok(())
}

/// Dispatch OnUpdate handlers for visible frames via rilua.
///
/// This is the rilua equivalent of `on_update::fire`. It iterates the
/// `__on_update_scripts` registry table and calls each handler with
/// `(frame, elapsed)` arguments.
///
/// Callers should pause GC before calling this and step GC after.
pub fn dispatch_on_update(
    lua: &mut rilua::Lua,
    frame_ids: &[u64],
    elapsed: f64,
) -> rilua::LuaResult<()> {
    let elapsed_val = Val::Num(elapsed);
    for &frame_id in frame_ids {
        let mode = {
            let state = lua.state_mut();
            on_update_mode_for_frame(state, frame_id)?
        };
        let dispatch = {
            let state = lua.state_mut();
            should_dispatch_on_update(state, frame_id, mode)
        };
        if !dispatch {
            continue;
        }
        let handlers = {
            let state = lua.state_mut();
            get_script_handlers_for_dispatch(state, frame_id, "OnUpdate")
        };
        if handlers.is_empty() {
            continue;
        };
        let frame_val = {
            let state = lua.state_mut();
            frame_ref(state, frame_id)?
        };
        if mode.is_one_shot() {
            disable_on_update_mode(lua.state_mut(), frame_id)?;
        }
        for handler in handlers {
            let Val::Function(func_ref) = handler.handler else {
                continue;
            };
            let registered_source = {
                let state = lua.state_mut();
                super::get_script_source_binding(state, frame_id, "OnUpdate", handler.binding)
            };
            dispatch_on_update_handler(
                lua,
                frame_id,
                frame_val,
                elapsed_val,
                func_ref,
                registered_source,
            );
        }
    }
    Ok(())
}

fn should_dispatch_on_update(state: &mut LuaState, frame_id: u64, mode: OnUpdateMode) -> bool {
    if mode == OnUpdateMode::Disabled {
        return false;
    }
    if !mode.requires_visibility() {
        return true;
    }
    borrow_state(state)
        .map(|sim| sim.widgets.is_ancestor_visible(frame_id))
        .unwrap_or(false)
}

fn on_update_mode_for_frame(state: &mut LuaState, frame_id: u64) -> rilua::LuaResult<OnUpdateMode> {
    let frame = frame_ref(state, frame_id)?;
    OnUpdateMode::from_stored_value(table_get(state, frame, "__onUpdateMode"))
}

fn disable_on_update_mode(state: &mut LuaState, frame_id: u64) -> rilua::LuaResult<()> {
    let frame = frame_ref(state, frame_id)?;
    crate::lua_api::methods::table_set(
        state,
        frame,
        "__onUpdateMode",
        OnUpdateMode::Disabled.value(),
    );
    Ok(())
}

fn dispatch_on_update_handler(
    lua: &mut rilua::Lua,
    frame_id: u64,
    frame_val: Val,
    elapsed_val: Val,
    func_ref: GcRef<Closure>,
    registered_source: Option<String>,
) {
    let (owner_addon, addon_name, frame_name) = handler_log_metadata(lua.state(), frame_id);
    let func = rilua::Function::from_gc_ref(func_ref);
    let start = Instant::now();
    let previous_addon = replace_executing_addon(lua.state(), owner_addon);
    if let Err(e) = lua.call_function(&func, &[frame_val, elapsed_val]) {
        let source =
            registered_source.or_else(|| handler_error_source_label(lua.state(), func_ref));
        let error = handler_error_message(
            "OnUpdate",
            frame_id,
            frame_name.as_deref(),
            addon_name.as_deref(),
            source.as_deref(),
            &e.to_string(),
        );
        call_error_handler(lua, &error);
    }
    replace_executing_addon(lua.state(), previous_addon);
    let elapsed = start.elapsed();
    record_frame_timing(lua.state(), owner_addon, &start);
    log_dispatched_handler(
        lua.state(),
        func_ref,
        addon_name.as_deref(),
        "OnUpdate",
        frame_name.as_deref(),
        frame_id,
        elapsed,
    );
}

fn handler_error_message(
    handler_name: &str,
    frame_id: u64,
    frame_name: Option<&str>,
    addon_name: Option<&str>,
    source: Option<&str>,
    error: &str,
) -> String {
    let frame = handler_frame_label(frame_name, frame_id);
    let addon = addon_name.unwrap_or(BUILTIN_ADDON_NAME);
    let mut message = format!("[{handler_name}] frame={frame} addon={addon}");
    if let Some(source) = source.filter(|source| !source.is_empty()) {
        message.push_str(" source=");
        message.push_str(source);
    }
    message.push_str(": ");
    message.push_str(error);
    message
}

fn handler_frame_label(frame_name: Option<&str>, frame_id: u64) -> Cow<'_, str> {
    match frame_name {
        Some(name) if !name.is_empty() => Cow::Borrowed(name),
        _ => Cow::Owned(format!("#{frame_id}")),
    }
}

fn handler_log_metadata(
    state: &LuaState,
    frame_id: u64,
) -> (Option<u16>, Option<String>, Option<String>) {
    use crate::lua_api::env::WowLuaAppData;

    let Some(app) = state.app_data::<WowLuaAppData>() else {
        return (None, None, None);
    };
    let Ok(sim) = app.sim_state.try_borrow() else {
        return (None, None, None);
    };
    let owner_addon = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.owner_addon);
    let addon_name = owner_addon
        .and_then(|idx| sim.addons.get(idx as usize))
        .map(|addon| addon.folder_name.clone());
    let frame_name = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.name.clone());
    (owner_addon, addon_name, frame_name)
}

fn replace_executing_addon(state: &LuaState, owner_addon: Option<u16>) -> Option<u16> {
    use crate::lua_api::env::WowLuaAppData;

    let Some(app) = state.app_data::<WowLuaAppData>() else {
        return None;
    };
    let Ok(mut sim) = app.sim_state.try_borrow_mut() else {
        return None;
    };
    let previous_addon = sim.executing_addon_index;
    sim.executing_addon_index = owner_addon;
    previous_addon
}

fn log_dispatched_handler(
    state: &LuaState,
    func_ref: GcRef<Closure>,
    addon_name: Option<&str>,
    handler_name: &str,
    frame_name: Option<&str>,
    frame_id: u64,
    elapsed: std::time::Duration,
) {
    if !handler_timing::should_log(elapsed) {
        return;
    }

    let source = handler_source_label(state, func_ref);
    handler_timing::log_with_source(
        addon_name,
        handler_name,
        frame_name,
        frame_id,
        elapsed,
        source.as_deref(),
    );
}

fn handler_source_label(state: &LuaState, func_ref: GcRef<Closure>) -> Option<String> {
    handler_timing::lua_closure_source_label(state, func_ref)
}

fn handler_error_source_label(state: &LuaState, func_ref: GcRef<Closure>) -> Option<String> {
    let closure = state.gc.closures.get(func_ref)?;
    let lua_closure = closure.as_lua()?;
    let proto = lua_closure.proto.as_ref();
    if proto.short_source.is_empty() {
        return None;
    }

    Some(format!("{}:{}", proto.short_source, proto.line_defined))
}

fn record_frame_timing(state: &LuaState, owner_addon: Option<u16>, start: &Instant) {
    use crate::lua_api::env::WowLuaAppData;

    let Some(addon_idx) = owner_addon else {
        return;
    };
    let Some(app) = state.app_data::<WowLuaAppData>() else {
        return;
    };
    let Ok(mut sim) = app.sim_state.try_borrow_mut() else {
        return;
    };
    let Some(addon) = sim.addons.get_mut(addon_idx as usize) else {
        return;
    };
    addon.runtime.current_frame_ms += start.elapsed().as_secs_f64() * 1000.0;
}
