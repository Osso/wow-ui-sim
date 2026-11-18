//! Mists cinematic completion state from explicit host inputs.

use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_cinematic_finished(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CinematicFinished", cinematic_finished)?;
    Ok(())
}

pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) -> crate::Result<()> {
    let mut lua = env.lua.borrow_mut();
    if is_glue {
        LuaApiMut::register_function(&mut *lua, "CinematicStarted", cinematic_started_unknown)?;
    } else {
        LuaApiMut::set_global_val(&mut *lua, "CinematicStarted", Val::Nil)?;
    }
    Ok(())
}

/// UNKNOWN (R88aq pending): Glue registration only; this consumes no args and returns no values.
fn cinematic_started_unknown(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// VERIFIED (R88ao, Mists 5.5.4.70631): finish records flags for an active type and clears it.
fn cinematic_finished(state: &mut LuaState) -> LuaResult<u32> {
    let movie_type = i32::from_stack(state, 1)?;
    let user_canceled = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    let did_error = Option::<bool>::from_stack(state, 3)?.unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if !sim.cinematic_active || sim.movie_playback.active_cinematic_type != Some(movie_type) {
        return Ok(0);
    }
    let elapsed = sim
        .movie_playback
        .cinematic_started_at
        .map(|start| start.elapsed());
    sim.movie_playback.last_cinematic_finish = Some(crate::lua_api::state::CinematicFinishRecord {
        movie_type,
        elapsed,
        user_canceled,
        did_error,
    });
    sim.movie_playback.active_cinematic_type = None;
    sim.movie_playback.cinematic_started_at = None;
    sim.cinematic_active = false;
    Ok(0)
}
