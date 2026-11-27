//! Spirit-release timer (`GetReleaseTimeRemaining`, PlayerScriptDocumentation.lua:
//! Retail :766, PTR :965, Forever :955; one non-nilable number).
//!
//! `A_Admin.SetReleaseTimeRemaining(seconds)` supplies the timer the server
//! starts at death: a positive number of seconds, `-1` for a death without a
//! release timer (the `DEATH_RELEASE_NOTIMER` branch of the DEATH dialog), or
//! `0` to clear it. INFERRED R88g9 (native evidence pending): a positive timer
//! counts down on the simulation clock, rounds up to whole seconds and stops
//! at 0; without a supplied timer the result is 0, the value the Lua fallback
//! reported before this model.

use std::time::{Duration, Instant};

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// Supplied release timer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReleaseTimer {
    /// Dead without an automatic release timer (`-1`).
    Untimed,
    /// Automatic release at this simulation-clock instant.
    Until(Instant),
}

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "GetReleaseTimeRemaining", get_release_time_remaining)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetReleaseTimeRemaining", set_release_time_remaining)
}

fn get_release_time_remaining(state: &mut LuaState) -> LuaResult<u32> {
    let remaining = {
        let sim = borrow_state(state)?;
        match sim.release_timer {
            None => 0.0,
            Some(ReleaseTimer::Untimed) => -1.0,
            Some(ReleaseTimer::Until(deadline)) => deadline
                .saturating_duration_since(sim.clock_now())
                .as_secs_f64()
                .ceil(),
        }
    };
    state.push(Val::Num(remaining));
    Ok(1)
}

fn set_release_time_remaining(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = f64::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.release_timer = if seconds == 0.0 {
        None
    } else if seconds == -1.0 {
        Some(ReleaseTimer::Untimed)
    } else if seconds.is_finite() && seconds > 0.0 {
        let deadline = sim.clock_now() + Duration::from_secs_f64(seconds);
        Some(ReleaseTimer::Until(deadline))
    } else {
        return Err(runtime_error(
            "A_Admin.SetReleaseTimeRemaining: expected positive seconds, -1 or 0",
        ));
    };
    Ok(0)
}
