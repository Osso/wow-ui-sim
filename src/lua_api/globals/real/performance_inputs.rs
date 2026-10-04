//! Live explicit profiling snapshots; no automatic timing or native attribution.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetEventCPUUsage", event_cpu_usage)?;
    LuaApiMut::register_function(lua, "GetFunctionCPUUsage", function_cpu_usage)?;
    LuaApiMut::register_function(lua, "GetScriptCPUUsage", script_cpu_usage)
}

/// INFERRED NeverSecret: cache omits argument policy; all extras are checked.
fn reject_secret_arguments(state: &LuaState) -> LuaResult<()> {
    for value in state.stack.iter().take(state.top).skip(state.base) {
        if rilua::table_security::is_secret_value(state, *value) {
            return Err(rilua::runtime_error(
                "profiling queries do not accept secret arguments",
            ));
        }
    }
    Ok(())
}

fn event_cpu_usage(state: &mut LuaState) -> LuaResult<u32> {
    reject_secret_arguments(state)?;
    let (time, count) = {
        let sim = borrow_state(state)?;
        (
            sim.performance_inputs.event_time,
            sim.performance_inputs.event_count,
        )
    };
    state.push(Val::Num(time));
    state.push(Val::Num(count));
    Ok(2)
}

fn function_cpu_usage(state: &mut LuaState) -> LuaResult<u32> {
    reject_secret_arguments(state)?;
    let (time, count) = {
        let sim = borrow_state(state)?;
        (
            sim.performance_inputs.function_time,
            sim.performance_inputs.function_count,
        )
    };
    state.push(Val::Num(time));
    state.push(Val::Num(count));
    Ok(2)
}

fn script_cpu_usage(state: &mut LuaState) -> LuaResult<u32> {
    reject_secret_arguments(state)?;
    let result = borrow_state(state)?.performance_inputs.script_usage;
    state.push(Val::Num(result));
    Ok(1)
}
