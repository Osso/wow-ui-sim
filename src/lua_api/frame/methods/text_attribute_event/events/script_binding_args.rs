use crate::lua_api::frame::methods::forbidden_aspects::hooked_script_component_allowed;
use crate::lua_api::script_helpers::{
    ScriptBinding, is_layout_script_handler, protected_lua_pcall_state, registry_value,
    set_registry_value,
};
use crate::lua_api::taint::{clear_active_stack_taint, restore_active_stack_taint};
use crate::lua_bridge::stack_val;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub(super) fn optional_script_binding_from_stack(
    state: &mut LuaState,
    index: i32,
) -> LuaResult<ScriptBinding> {
    match stack_val(state, index) {
        Val::Nil => Ok(ScriptBinding::Normal),
        Val::Num(raw) if raw.is_finite() => parse_script_binding(raw),
        other => Err(runtime_error(format!(
            "script binding type must be a number, got {}",
            other.type_name()
        ))),
    }
}

fn parse_script_binding(raw: f64) -> LuaResult<ScriptBinding> {
    let binding_index = raw as i32;
    if binding_index as f64 == raw
        && let Some(binding) = ScriptBinding::from_index(binding_index)
    {
        return Ok(binding);
    }
    Err(runtime_error(format!(
        "script binding type must be 0, 1, or 2, got {raw}"
    )))
}

const HOOK_GATE_KEY: &str = "__wow_hooked_script_component_gate";

/// Chains run each component through the forbidden-aspect gate, so an addon hook on a
/// secure handler (or the reverse) is suppressed without dropping the other component.
pub(super) fn build_hooked_script(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &str,
    old: Val,
    hook: Val,
) -> LuaResult<Val> {
    let func = state.load(
        r#"
        local old, hook, frameId, layout, allowed = ...
        if old == nil then
            return hook
        end
        return function(...)
            if allowed(frameId, layout, old) then old(...) end
            if allowed(frameId, layout, hook) then hook(...) end
        end
    "#,
    )?;
    let gate = hook_gate(state);
    let args = [
        old,
        hook,
        Val::Num(frame_id as f64),
        Val::Bool(is_layout_script_handler(handler_name)),
        gate,
    ];
    let saved_taints = clear_active_stack_taint(state);
    let result = protected_lua_pcall_state(state, Val::Function(func.gc_ref()), &args);
    restore_active_stack_taint(state, saved_taints);
    result
        .map_err(runtime_error)?
        .into_iter()
        .next()
        .ok_or_else(|| runtime_error("HookScript factory returned no handler"))
}

fn hook_gate(state: &mut LuaState) -> Val {
    let existing = registry_value(state, HOOK_GATE_KEY);
    if matches!(existing, Val::Function(_)) {
        return existing;
    }
    let gate = Closure::Rust(RustClosure::new(
        hooked_script_component_allowed,
        "HookScriptGate",
    ));
    let gate = Val::Function(state.gc.alloc_closure(gate));
    let stack_slot = state.top;
    state.ensure_stack(stack_slot + 1);
    state.stack_set(stack_slot, gate);
    state.top = stack_slot + 1;
    set_registry_value(state, HOOK_GATE_KEY, gate);
    state.top = stack_slot;
    gate
}

pub(super) fn reject_unsupported_hook_binding(
    state: &mut LuaState,
    binding: ScriptBinding,
) -> bool {
    if binding == ScriptBinding::Normal {
        return false;
    }
    state.push(Val::Bool(false));
    true
}
