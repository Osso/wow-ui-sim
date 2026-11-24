//! INFERRED (R88ft (native evidence pending)): session input for dangerous script permission.
//!
//! Native keeps this permission per account. The simulator models it in memory only.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "AreDangerousScriptsAllowed",
        are_dangerous_scripts_allowed,
    )?;
    LuaApiMut::register_function(lua, "SetAllowDangerousScripts", set_allow_dangerous_scripts)
}

fn are_dangerous_scripts_allowed(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = borrow_state(state)?.dangerous_scripts_allowed;
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn set_allow_dangerous_scripts(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = match stack_val(state, 1) {
        Val::Nil | Val::Bool(false) => false,
        Val::Bool(true) => true,
        _ => {
            return Err(runtime_error(
                "SetAllowDangerousScripts: expected a boolean",
            ));
        }
    };
    borrow_state_mut(state)?.dangerous_scripts_allowed = allowed;
    Ok(0)
}
