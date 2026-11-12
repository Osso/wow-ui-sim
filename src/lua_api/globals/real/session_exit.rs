//! Session exit globals (`Quit`, `Logout`, `ForceQuit`, `ForceLogout`).
//!
//! The GUI owns the actual window close. These globals mark simulator state so
//! any Lua path, including Blizzard GameMenu button scripts, can request it.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn request_simulator_exit(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.simulator_exit_requested = true;
    Ok(0)
}

fn request_logout(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.is_logged_in = false;
    Ok(0)
}

pub(crate) fn is_simulator_exit_requested(state: &mut LuaState) -> LuaResult<u32> {
    let requested = borrow_state(state)?.simulator_exit_requested;
    state.push(Val::Bool(requested));
    Ok(1)
}

fn cancel_logout(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .character_services
        .automatic_boost_type = None;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "Quit", request_simulator_exit)?;
    LuaApiMut::register_function(lua, "QuitGame", request_simulator_exit)?;
    LuaApiMut::register_function(lua, "ForceQuit", request_simulator_exit)?;
    LuaApiMut::register_function(lua, "Logout", request_logout)?;
    LuaApiMut::register_function(lua, "ForceLogout", request_logout)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "CancelLogout", cancel_logout)?;
    }

    Ok(())
}
