//! Session exit globals (`Quit`, `Logout`, `ForceQuit`, `ForceLogout`).
//!
//! The GUI owns the actual window close. These globals mark simulator state so
//! any Lua path, including Blizzard GameMenu button scripts, can request it.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaError, LuaResult, RuntimeError, Val};

fn request_simulator_exit(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.simulator_exit_requested = true;
    Ok(0)
}

fn request_logout(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.is_logged_in = false;
    Ok(0)
}

fn require_secure_session_call(state: &LuaState, name: &str) -> LuaResult<()> {
    let insecure = state.call_stack[..=state.ci]
        .iter()
        .any(|frame| frame.taint.is_some());
    if cfg!(feature = "client-retail") && insecure {
        return Err(LuaError::Runtime(RuntimeError {
            message: format!("{name} is protected from insecure code"),
            level: 1,
            traceback: vec![],
        }));
    }
    Ok(())
}

fn quit(state: &mut LuaState) -> LuaResult<u32> {
    require_secure_session_call(state, "Quit")?;
    request_simulator_exit(state)
}

fn logout(state: &mut LuaState) -> LuaResult<u32> {
    require_secure_session_call(state, "Logout")?;
    request_logout(state)
}

pub(crate) fn is_simulator_exit_requested(state: &mut LuaState) -> LuaResult<u32> {
    let requested = borrow_state(state)?.simulator_exit_requested;
    state.push(Val::Bool(requested));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "Quit", quit)?;
    LuaApiMut::register_function(lua, "QuitGame", request_simulator_exit)?;
    LuaApiMut::register_function(lua, "ForceQuit", request_simulator_exit)?;
    LuaApiMut::register_function(lua, "Logout", logout)?;
    LuaApiMut::register_function(lua, "ForceLogout", request_logout)?;

    Ok(())
}
