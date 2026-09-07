//! Attribute, frame-flag, and script-flag RustFn methods.

use super::helpers::{attribute_to_val, store_simple_attribute, val_to_f32};
use crate::lua_api::frame::methods::methods_helpers::can_change_protected_state_for;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, frame_id_from_stack,
    frame_ref, val_to_string,
};
use crate::lua_api::script_helpers::{
    call_error_handler_state, get_script as get_rilua_script, protected_lua_pcall_state,
};
use crate::lua_api::taint::{clear_active_stack_taint, restore_active_stack_taint};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub(super) fn get_attribute(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let arg_count = state.top.saturating_sub(state.base);
    let first = val_to_string(state, stack_val(state, 2));
    let second = val_to_string(state, stack_val(state, 3));
    let third = val_to_string(state, stack_val(state, 4));
    let keys = build_attribute_keys(arg_count, first, second, third)?;
    let attr = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|frame| {
            keys.iter()
                .find_map(|key| frame.attributes.get(key.as_str()).cloned())
        })
    };
    let val = attribute_to_val(state, attr.as_ref());
    state.push(val);
    Ok(1)
}

fn build_attribute_keys(
    arg_count: usize,
    first: Option<String>,
    second: Option<String>,
    third: Option<String>,
) -> LuaResult<Vec<String>> {
    match (first, second, third) {
        (Some(name), None, None) if arg_count == 2 => Ok(vec![name]),
        (Some(prefix), Some(name), suffix) => Ok(attribute_lookup_keys(
            &prefix,
            &name,
            suffix.as_deref().unwrap_or_default(),
        )),
        (None, Some(name), Some(suffix)) => Ok(attribute_lookup_keys("", &name, &suffix)),
        _ => Err(runtime_error("Arguments: (\"name\")")),
    }
}

fn attribute_lookup_keys(prefix: &str, name: &str, suffix: &str) -> Vec<String> {
    let mut keys = Vec::with_capacity(5);
    keys.push(format!("{prefix}{name}{suffix}"));
    keys.push(format!("*{name}{suffix}"));
    keys.push(format!("{prefix}{name}*"));
    keys.push(format!("*{name}*"));
    keys.push(name.to_string());
    keys
}

pub(super) fn set_attribute(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let name_val = stack_val(state, 2);
    let value = stack_val(state, 3);
    let Some(name) = val_to_string(state, name_val) else {
        return Ok(0);
    };
    if protected_write_blocked(state, id) {
        return Ok(0);
    }
    let name_arg = create_string(state, &name);
    let secure_delegate_dispatch = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).is_some_and(|frame| frame.forbidden)
    };
    let dispatch = |state: &mut LuaState| -> LuaResult<()> {
        let frame = frame_ref(state, id)?;
        if let Some(handler) = get_rilua_script(state, id, "OnAttributeChanged") {
            dispatch_attribute_changed(
                state,
                handler,
                frame,
                name_arg,
                value,
                secure_delegate_dispatch,
            );
        } else {
            dispatch_direct_attribute_changed(state, frame, name_arg, value);
        }
        Ok(())
    };
    if value == Val::Bool(false) {
        let _ = store_simple_attribute(state, id, &name, Val::Nil)?;
        dispatch(state)?;
    }
    let changed = store_simple_attribute(state, id, &name, value)?;
    if value != Val::Bool(false) {
        dispatch(state)?;
    }
    if changed {
        run_state_attribute_snippet(state, id, &name, value)?;
    }
    Ok(0)
}

fn run_state_attribute_snippet(
    state: &mut LuaState,
    id: u64,
    attribute: &str,
    value: Val,
) -> LuaResult<()> {
    let Some(state_name) = attribute.strip_prefix("state-") else {
        return Ok(());
    };
    let snippet_attribute = format!("_onstate-{state_name}");
    let snippet_body = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| frame.attributes.get(&snippet_attribute))
            .and_then(attribute_string)
            .map(str::to_string)
    };
    let Some(snippet_body) = snippet_body else {
        return Ok(());
    };

    let snippet = compile_state_attribute_snippet(state, &snippet_body)?;
    let frame = frame_ref(state, id)?;
    if let Err(error) =
        protected_lua_pcall_state(state, Val::Function(snippet.gc_ref()), &[frame, value])
    {
        call_error_handler_state(state, &error);
    }
    Ok(())
}

fn attribute_string(attribute: &crate::widget::AttributeValue) -> Option<&str> {
    match attribute {
        crate::widget::AttributeValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn compile_state_attribute_snippet(state: &mut LuaState, body: &str) -> LuaResult<rilua::Function> {
    let loader = state.load(&format!(
        "return function(self, newstate) local strsub = string.sub; {body} end"
    ))?;
    let closure = call_function_state(state, Val::Function(loader.gc_ref()), &[])?;
    let Val::Function(func_ref) = closure else {
        return Err(runtime_error(
            "state attribute snippet loader did not return a function",
        ));
    };
    Ok(rilua::Function::from_gc_ref(func_ref))
}

/// True when the caller cannot mutate protected state for the current frame.
/// Attribute writes use the same lockdown gate as other protected frame
/// mutations.
pub(super) fn protected_write_blocked(state: &mut LuaState, id: u64) -> bool {
    !can_change_protected_state_for(state, id)
}

pub(super) fn dispatch_attribute_changed(
    state: &mut LuaState,
    handler: Val,
    frame: Val,
    name: Val,
    value: Val,
    secure_dispatch: bool,
) {
    let Ok(dispatcher) = state.load(
        r#"
        local handler, frame, name, value = ...
        handler(frame, name, value)
        "#,
    ) else {
        return;
    };
    let call_base = state.top;
    state.ensure_stack(call_base + 5);
    state.stack_set(call_base, Val::Function(dispatcher.gc_ref()));
    state.stack_set(call_base + 1, handler);
    state.stack_set(call_base + 2, frame);
    state.stack_set(call_base + 3, name);
    state.stack_set(call_base + 4, value);
    state.top = call_base + 5;

    let saved_taints = secure_dispatch.then(|| clear_active_stack_taint(state));
    let call_result = state.call_function(call_base, 0);
    if let Some(saved_taints) = saved_taints {
        restore_active_stack_taint(state, saved_taints);
    }

    if let Err(error) = call_result {
        call_error_handler_state(state, &error.to_string());
    }
    state.top = call_base;
}

fn dispatch_direct_attribute_changed(state: &mut LuaState, frame: Val, name: Val, value: Val) {
    let Ok(dispatcher) = state.load(
        r#"
        local frame, name, value = ...
        local env = debug.getfenv(frame)
        local fields = env and env[1]
        local handler = fields and rawget(fields, "OnAttributeChanged")
        if type(handler) ~= "function" then
            local ok, direct = pcall(rawget, frame, "OnAttributeChanged")
            if ok then
                handler = direct
            end
        end
        if type(handler) == "function" then
            handler(frame, name, value)
        end
        "#,
    ) else {
        return;
    };
    let call_base = state.top;
    state.ensure_stack(call_base + 4);
    state.stack_set(call_base, Val::Function(dispatcher.gc_ref()));
    state.stack_set(call_base + 1, frame);
    state.stack_set(call_base + 2, name);
    state.stack_set(call_base + 3, value);
    state.top = call_base + 4;
    if let Err(error) = state.call_function(call_base, 0) {
        call_error_handler_state(state, &error.to_string());
    }
    state.top = call_base;
}

pub(super) fn set_attribute_no_handler(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let name_val = stack_val(state, 2);
    let value = stack_val(state, 3);
    let Some(name) = val_to_string(state, name_val) else {
        return Ok(0);
    };
    if protected_write_blocked(state, id) {
        return Ok(0);
    }
    let _ = store_simple_attribute(state, id, &name, value)?;
    Ok(0)
}

pub(super) fn clear_attributes(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.attributes.clear();
    }
    Ok(0)
}

pub(super) fn child_update(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let script_id = val_to_string(state, stack_val(state, 2));
    let message = stack_val(state, 3);
    let child_updates = child_update_snippets(state, id, script_id.as_deref())?;
    let script_id_value = script_id
        .as_deref()
        .map(|value| create_string(state, value))
        .unwrap_or(Val::Nil);

    for (child_id, body) in child_updates {
        let snippet = compile_child_update_snippet(state, &body)?;
        let child = frame_ref(state, child_id)?;
        if let Err(error) = protected_lua_pcall_state(
            state,
            Val::Function(snippet.gc_ref()),
            &[child, script_id_value, message],
        ) {
            call_error_handler_state(state, &error);
        }
    }
    Ok(0)
}

fn child_update_snippets(
    state: &mut LuaState,
    id: u64,
    script_id: Option<&str>,
) -> LuaResult<Vec<(u64, String)>> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(Vec::new());
    };
    let specific_attribute = script_id.map(|id| format!("_childupdate-{id}"));
    let updates = frame
        .children
        .iter()
        .filter_map(|child_id| {
            let child = sim.widgets.get(*child_id)?;
            if !child.is_protected {
                return None;
            }
            let body = specific_attribute
                .as_deref()
                .and_then(|attribute| child.attributes.get(attribute))
                .or_else(|| child.attributes.get("_childupdate"))
                .and_then(attribute_string)?;
            Some((*child_id, body.to_string()))
        })
        .collect();
    Ok(updates)
}

fn compile_child_update_snippet(state: &mut LuaState, body: &str) -> LuaResult<rilua::Function> {
    let loader = state.load(&format!(
        "return function(self, scriptid, message) local strsub = string.sub; {body} end"
    ))?;
    let closure = call_function_state(state, Val::Function(loader.gc_ref()), &[])?;
    let Val::Function(func_ref) = closure else {
        return Err(runtime_error(
            "ChildUpdate: snippet loader did not return a function",
        ));
    };
    Ok(rilua::Function::from_gc_ref(func_ref))
}

pub(super) fn execute_attribute(state: &mut LuaState) -> LuaResult<u32> {
    let call = match execute_attribute_call(state)? {
        Ok(call) => call,
        Err(reason) => return push_execute_attribute_failure(state, reason),
    };

    match call.attr {
        Val::Function(_) => execute_function_attribute(state, call.attr, &call.extra_args),
        Val::Str(_) if call.frame_is_protected => {
            execute_snippet_attribute(state, call.frame_id, call.attr, call.extra_args)
        }
        Val::Str(_) => push_execute_attribute_failure(state, "unsupported-unprotected-snippet"),
        _ => push_execute_attribute_failure(state, "attribute-missing"),
    }
}

struct ExecuteAttributeCall {
    frame_id: u64,
    frame_is_protected: bool,
    attr: Val,
    extra_args: Vec<Val>,
}

fn execute_attribute_call(
    state: &mut LuaState,
) -> LuaResult<Result<ExecuteAttributeCall, &'static str>> {
    let id = frame_id_from_stack(state, 1)?;
    let name_val = stack_val(state, 2);
    let Some(name) = val_to_string(state, name_val) else {
        return Ok(Err("attribute-missing"));
    };
    let (attr, frame_is_protected) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let attr = frame.and_then(|frame| frame.attributes.get(name.as_str()).cloned());
        let frame_is_protected = frame.is_some_and(|frame| frame.is_protected);
        (attr, frame_is_protected)
    };
    let attr = attribute_to_val(state, attr.as_ref());
    let nargs = (state.top as i32 - state.base as i32) as usize;
    let extra_args = (3..=nargs)
        .map(|index| stack_val(state, index as i32))
        .collect::<Vec<_>>();
    Ok(Ok(ExecuteAttributeCall {
        frame_id: id,
        frame_is_protected,
        attr,
        extra_args,
    }))
}

fn execute_function_attribute(
    state: &mut LuaState,
    attr: Val,
    extra_args: &[Val],
) -> LuaResult<u32> {
    let results =
        protected_lua_pcall_state(state, attr, extra_args).map_err(|error| error.to_string());
    push_execute_attribute_result(state, results)
}

fn execute_snippet_attribute(
    state: &mut LuaState,
    frame_id: u64,
    attr: Val,
    extra_args: Vec<Val>,
) -> LuaResult<u32> {
    let Some(body) = val_to_string(state, attr) else {
        return push_execute_attribute_failure(state, "attribute-missing");
    };
    let Ok(snippet) = compile_execute_attribute_snippet(state, &body) else {
        return push_execute_attribute_failure(state, "snippet-compile-failed");
    };
    let mut args = Vec::with_capacity(extra_args.len() + 1);
    args.push(frame_ref(state, frame_id)?);
    args.extend(extra_args);
    let results = protected_lua_pcall_state(state, Val::Function(snippet.gc_ref()), &args)
        .map_err(|error| error.to_string());
    push_execute_attribute_result(state, results)
}

fn compile_execute_attribute_snippet(
    state: &mut LuaState,
    body: &str,
) -> LuaResult<rilua::Function> {
    let loader = state.load(&format!("return function(self, ...) {body} end"))?;
    let closure = call_function_state(state, Val::Function(loader.gc_ref()), &[])?;
    let Val::Function(func_ref) = closure else {
        return Err(rilua::runtime_error(
            "ExecuteAttribute: snippet loader did not return a function",
        ));
    };
    Ok(rilua::Function::from_gc_ref(func_ref))
}

fn push_execute_attribute_result(
    state: &mut LuaState,
    results: Result<Vec<Val>, String>,
) -> LuaResult<u32> {
    match results {
        Ok(values) => {
            state.push(Val::Bool(true));
            let return_count = values.len() as u32 + 1;
            for value in values {
                state.push(value);
            }
            Ok(return_count)
        }
        Err(error) => push_execute_attribute_failure(state, &error),
    }
}

fn push_execute_attribute_failure(state: &mut LuaState, reason: &str) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    let reason = create_string(state, reason);
    state.push(reason);
    Ok(2)
}

pub(super) fn set_frame_ref(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let label_val = stack_val(state, 2);
    let frame_val = stack_val(state, 3);
    let Some(label) = val_to_string(state, label_val) else {
        return Ok(0);
    };
    let _ = store_simple_attribute(state, id, &format!("_frame-{label}"), frame_val)?;
    Ok(0)
}

pub(super) fn get_frame_ref(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let label_val = stack_val(state, 2);
    let Some(label) = val_to_string(state, label_val) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let attr = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|frame| {
            frame
                .attributes
                .get(format!("_frame-{label}").as_str())
                .cloned()
        })
    };
    let attr = attribute_to_val(state, attr.as_ref());
    state.push(attr);
    Ok(1)
}

pub(super) fn set_forbidden(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    #[cfg(feature = "profile-retail")]
    {
        if !rilua::api::state_is_secure(state) {
            return Ok(0);
        }
    }

    {
        // TODO: combat lockdown check
        let forbidden = match stack_val(state, 2) {
            Val::Bool(b) => b,
            _ => true,
        };
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut(id) {
            frame.forbidden = forbidden;
        }
        Ok(0)
    }
}

pub(super) fn has_access_constraints(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let has_access_constraints = sim
        .widgets
        .get(id)
        .map(|frame| frame.forbidden || frame.access_restrictions != 0)
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(has_access_constraints));
    Ok(1)
}

pub(super) fn is_forbidden(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let val = sim.widgets.get(id).map(|f| f.forbidden).unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(val));
    Ok(1)
}

pub(super) fn can_change_protected_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let allowed = can_change_protected_state_for(state, id);
    state.push(Val::Bool(allowed));
    Ok(1)
}

pub(super) fn set_pass_through_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let nargs = (state.top as i32 - state.base as i32) as usize;
    let buttons = (2..=nargs)
        .filter_map(|index| val_to_string(state, stack_val(state, index as i32)))
        .map(|button| button.to_ascii_lowercase())
        .collect();
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.pass_through_buttons = buttons;
    }
    Ok(0)
}

pub(super) fn set_flattens_render_layers(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let flatten = matches!(stack_val(state, 2), Val::Bool(b) if b);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.flattens_render_layers = flatten;
    }
    Ok(0)
}

pub(super) fn set_motion_scripts_while_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = matches!(stack_val(state, 2), Val::Bool(b) if b);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.motion_scripts_while_disabled = enabled;
    }
    Ok(0)
}

pub(super) fn get_motion_scripts_while_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let val = sim
        .widgets
        .get(id)
        .map(|f| f.motion_scripts_while_disabled)
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(val));
    Ok(1)
}

pub(super) fn set_clips_children(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let clips = matches!(stack_val(state, 2), Val::Bool(b) if b);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.clips_children = clips;
    }
    Ok(0)
}

pub(super) fn does_clip_children(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let val = sim
        .widgets
        .get(id)
        .map(|f| f.clips_children)
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(val));
    Ok(1)
}

pub(super) fn set_hit_rect_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    // TODO: combat lockdown check
    let l = val_to_f32(stack_val(state, 2), 0.0);
    let r = val_to_f32(stack_val(state, 3), 0.0);
    let t = val_to_f32(stack_val(state, 4), 0.0);
    let b = val_to_f32(stack_val(state, 5), 0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.hit_rect_insets = (l, r, t, b);
    }
    Ok(0)
}

pub(super) fn get_hit_rect_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (l, r, t, b) = sim
        .widgets
        .get(id)
        .map(|f| f.hit_rect_insets)
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    drop(sim);
    state.push(Val::Num(l as f64));
    state.push(Val::Num(r as f64));
    state.push(Val::Num(t as f64));
    state.push(Val::Num(b as f64));
    Ok(4)
}
