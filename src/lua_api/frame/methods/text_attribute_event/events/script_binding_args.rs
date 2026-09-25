use crate::lua_api::script_helpers::{ScriptBinding, protected_lua_pcall_state};
use crate::lua_api::taint::{clear_active_stack_taint, restore_active_stack_taint};
use crate::lua_bridge::stack_val;
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

pub(super) fn build_hooked_script(state: &mut LuaState, old: Val, hook: Val) -> LuaResult<Val> {
    let func = state.load(
        r#"
        local old, hook = ...
        if old == nil then
            return hook
        end
        return function(...)
            old(...)
            hook(...)
        end
    "#,
    )?;
    let saved_taints = clear_active_stack_taint(state);
    let result = protected_lua_pcall_state(state, Val::Function(func.gc_ref()), &[old, hook]);
    restore_active_stack_taint(state, saved_taints);
    result
        .map_err(runtime_error)?
        .into_iter()
        .next()
        .ok_or_else(|| runtime_error("HookScript factory returned no handler"))
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
