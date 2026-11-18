//! Temporary UNKNOWN Mists Glue placeholder for the original movie consumer.

use crate::lua_api::WowLuaEnv;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) -> crate::Result<()> {
    let mut lua = env.lua.borrow_mut();
    if is_glue {
        LuaApiMut::register_function(&mut *lua, "CinematicStarted", cinematic_started_unknown)?;
    } else {
        LuaApiMut::set_global_val(&mut *lua, "CinematicStarted", Val::Nil)?;
    }
    Ok(())
}

/// UNKNOWN (R88aq pending): consume no args and return no values; replace when evidence arrives.
fn cinematic_started_unknown(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
