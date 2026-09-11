//! Resolve and compile virtual animation properties before allocating the instance.
use rilua::vm::state::LuaState;
use rilua::{Function, LuaResult, Val, runtime_error};

pub(super) fn prepare_template(
    state: &mut LuaState,
    name: Option<&str>,
) -> LuaResult<Option<Function>> {
    let Some(name) = name else { return Ok(None) };
    let code = crate::loader::helpers_anim::generate_animation_template_code(name)
        .map_err(runtime_error)?;
    let saved_slots = state.global_slots.take();
    let function = crate::loader::chunk_cache::load_chunk(state, &code, "animation-template")
        .map_err(|error| runtime_error(error.to_string()));
    state.global_slots = saved_slots;
    let function = function?;
    crate::lua_api::loader_env::apply_loading_scoped_fenv_state(state, &function)
        .map_err(|error| runtime_error(error.to_string()))?;
    Ok(Some(function))
}

pub(super) fn apply_template(
    state: &mut LuaState,
    function: Option<Function>,
    animation: Val,
) -> LuaResult<()> {
    if let Some(function) = function {
        crate::lua_api::script_helpers::call_void_function_state(
            state,
            Val::Function(function.gc_ref()),
            &[animation],
        )
        .map_err(runtime_error)?;
    }
    Ok(())
}
