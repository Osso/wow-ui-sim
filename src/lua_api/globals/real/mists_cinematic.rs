//! Mists cinematic start and completion state from R88aq.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::{CinematicFinishRecord, CinematicRecord, MoviePlaybackState};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const CINEMATIC_STARTED_USAGE: &str = "Usage: CinematicStarted(movieType, movieID [, canCancel])";
const CINEMATIC_TYPE_MIN: i32 = 0;
const CINEMATIC_TYPE_MAX: i32 = 3;
const MOVIE_ID_RANGE_ERROR: &str =
    "bad argument #2 to CinematicStarted (outside of expected range -2147483648 to 2147483647)";

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CinematicStarted", cinematic_started)?;
    LuaApiMut::register_function(lua, "CinematicFinished", cinematic_finished)?;
    Ok(())
}

/// VERIFIED (R88aq): start a movie record and update the manager ID when this type was inactive.
fn cinematic_started(state: &mut LuaState) -> LuaResult<u32> {
    let movie_type = read_cinematic_type(state)?;
    let movie_id = read_cinematic_movie_id(state)?;
    let can_cancel = !matches!(stack_val(state, 3), Val::Bool(false));
    if movie_id == 0 {
        return Ok(0);
    }
    start_cinematic(
        &mut borrow_state_mut(state)?.movie_playback,
        movie_type,
        movie_id,
        can_cancel,
    );
    Ok(0)
}

/// VERIFIED (R88aq): completion always clears the manager ID; inactive types keep their record.
fn cinematic_finished(state: &mut LuaState) -> LuaResult<u32> {
    let movie_type = i32::from_stack(state, 1)?;
    let user_canceled = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    let did_error = Option::<bool>::from_stack(state, 3)?.unwrap_or(false);
    finish_cinematic(
        &mut borrow_state_mut(state)?.movie_playback,
        movie_type,
        user_canceled,
        did_error,
    );
    Ok(0)
}

fn read_cinematic_type(state: &LuaState) -> LuaResult<i32> {
    let value = read_numeric_argument(state, 1)?;
    let movie_type = truncate_to_i32(value)
        .ok_or_else(|| runtime_error(format!("integer overflow attempting to store {value:.6}")))?;
    if !(CINEMATIC_TYPE_MIN..=CINEMATIC_TYPE_MAX).contains(&movie_type) {
        return Err(runtime_error(CINEMATIC_STARTED_USAGE));
    }
    Ok(movie_type)
}

fn read_cinematic_movie_id(state: &LuaState) -> LuaResult<i32> {
    let value = read_numeric_argument(state, 2)?;
    truncate_to_i32(value).ok_or_else(|| runtime_error(MOVIE_ID_RANGE_ERROR))
}

fn read_numeric_argument(state: &LuaState, index: i32) -> LuaResult<f64> {
    match stack_val(state, index) {
        Val::Num(value) => Ok(value),
        Val::Str(_) => {
            let value = String::from_stack(state, index)
                .map_err(|_| runtime_error(CINEMATIC_STARTED_USAGE))?;
            parse_numeric_string(&value).ok_or_else(|| runtime_error(CINEMATIC_STARTED_USAGE))
        }
        _ => Err(runtime_error(CINEMATIC_STARTED_USAGE)),
    }
}

fn parse_numeric_string(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let (sign, unsigned) = if let Some(rest) = trimmed.strip_prefix('-') {
        (-1.0, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (1.0, rest)
    } else {
        (1.0, trimmed)
    };
    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16)
            .ok()
            .map(|number| sign * number as f64);
    }
    trimmed.parse().ok()
}

fn truncate_to_i32(value: f64) -> Option<i32> {
    let truncated = value.trunc();
    if !truncated.is_finite() || truncated < i32::MIN as f64 || truncated > i32::MAX as f64 {
        return None;
    }
    Some(truncated as i32)
}

fn start_cinematic(
    playback: &mut MoviePlaybackState,
    movie_type: i32,
    movie_id: i32,
    can_cancel: bool,
) {
    let restarts_active_type = playback
        .cinematic_records
        .get(&movie_type)
        .is_some_and(|record| record.active);
    if restarts_active_type {
        finish_cinematic(playback, movie_type, false, false);
    } else {
        playback.manager_movie_id = Some(movie_id);
    }
    playback.cinematic_records.insert(
        movie_type,
        CinematicRecord {
            movie_type,
            movie_id,
            can_cancel,
            started_at: std::time::Instant::now(),
            active: true,
        },
    );
}

fn finish_cinematic(
    playback: &mut MoviePlaybackState,
    movie_type: i32,
    user_canceled: bool,
    did_error: bool,
) {
    let finished = match playback.cinematic_records.get_mut(&movie_type) {
        Some(record) if record.active => {
            record.active = false;
            Some(CinematicFinishRecord {
                movie_type,
                elapsed: record.started_at.elapsed(),
                user_canceled,
                did_error,
            })
        }
        _ => None,
    };
    if let Some(finished) = finished {
        playback.last_cinematic_finish = Some(finished);
    }
    // UNKNOWN (R88aq): type-1 follow-up and observer effects are not modeled.
    playback.manager_movie_id = None;
}
