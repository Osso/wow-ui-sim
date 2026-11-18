//! MovieFrame subtitle preferences and Mists movie-handle lifecycle.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const MOVIE_SUCCESS_CODE: u32 = 0;
const MOVIE_MAX_RESULT_CODE: u32 = 4;

fn enable_subtitles(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = match stack_val(state, 2) {
        Val::Bool(value) => value,
        value => {
            return Err(runtime_error(format!(
                "EnableSubtitles expected boolean at argument 2, got {}",
                value.type_name()
            )));
        }
    };
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(id)
        .ok_or_else(|| runtime_error("EnableSubtitles requires a MovieFrame"))?;
    if frame.object_type_name.as_deref() != Some("MovieFrame") {
        return Err(runtime_error("EnableSubtitles requires a MovieFrame"));
    }
    frame.movie_subtitles_enabled = enabled;
    Ok(0)
}

#[cfg(feature = "client-mists")]
fn start_movie(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let movie_id = u32::from_stack(state, 2)?;
    require_movie_frame(state, frame_id, "StartMovie")?;
    let result_code = borrow_state(state)?
        .movie_playback
        .start_results
        .get(&movie_id)
        .copied();
    let Some(result_code) = result_code else {
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(2);
    };
    if result_code > MOVIE_MAX_RESULT_CODE {
        return Err(runtime_error(
            "StartMovie host result code must be between 0 and 4",
        ));
    }
    borrow_state_mut(state)?
        .movie_playback
        .active_handles
        .insert(frame_id);
    state.push(Val::Bool(result_code == MOVIE_SUCCESS_CODE));
    state.push(Val::Num(result_code as f64));
    Ok(2)
}

#[cfg(feature = "client-mists")]
fn stop_movie(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    require_movie_frame(state, frame_id, "StopMovie")?;
    if queue_movie_finish(state, frame_id)? {
        dispatch_pending_movie_finishes(state)?;
    }
    Ok(0)
}

#[cfg(feature = "client-mists")]
fn require_movie_frame(state: &LuaState, frame_id: u64, method: &str) -> LuaResult<()> {
    let sim = borrow_state(state)?;
    let is_movie_frame = sim
        .widgets
        .get(frame_id)
        .is_some_and(|frame| frame.object_type_name.as_deref() == Some("MovieFrame"));
    if is_movie_frame {
        Ok(())
    } else {
        Err(runtime_error(format!("{method} requires a MovieFrame")))
    }
}

#[cfg(feature = "client-mists")]
fn queue_movie_finish(state: &mut LuaState, frame_id: u64) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let playback = &mut sim.movie_playback;
    if !playback.active_handles.remove(&frame_id) {
        return Ok(false);
    }
    playback.pending_finishes.push_back(frame_id);
    if playback.dispatching_finish {
        return Ok(false);
    }
    playback.dispatching_finish = true;
    Ok(true)
}

#[cfg(feature = "client-mists")]
fn dispatch_pending_movie_finishes(state: &mut LuaState) -> LuaResult<()> {
    loop {
        let frame_id = {
            let mut sim = borrow_state_mut(state)?;
            match sim.movie_playback.pending_finishes.pop_front() {
                Some(frame_id) => Some(frame_id),
                None => {
                    sim.movie_playback.dispatching_finish = false;
                    None
                }
            }
        };
        let Some(frame_id) = frame_id else {
            return Ok(());
        };
        if let Err(error) = crate::lua_api::script_helpers::dispatch_script_state(
            state,
            frame_id,
            "OnMovieFinished",
            &[],
        ) {
            borrow_state_mut(state)?.movie_playback.dispatching_finish = false;
            return Err(error);
        }
    }
}

pub(super) fn register_movie(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "EnableSubtitles", enable_subtitles)?;
    #[cfg(feature = "client-mists")]
    {
        table_set_rust_fn_static(state, metatable, "StartMovie", start_movie)?;
        table_set_rust_fn_static(state, metatable, "StopMovie", stop_movie)?;
    }
    Ok(())
}
