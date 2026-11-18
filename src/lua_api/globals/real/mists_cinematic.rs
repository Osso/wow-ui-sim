//! Mists cinematic completion state from explicit host inputs.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

pub(crate) fn register_cinematic_finished(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CinematicFinished", cinematic_finished)?;
    Ok(())
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
