//! Explicit environment clock. Real elapsed time remains the default.
use super::methods::{borrow_state, borrow_state_mut};
use super::state::SimState;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ManualClock {
    elapsed: Duration,
    now: Instant,
}

impl SimState {
    /// The sole Lua-visible elapsed-time accessor; profiling stays real.
    pub(crate) fn clock_elapsed(&self) -> Duration {
        self.manual_clock
            .map_or_else(|| self.start_time.elapsed(), |clock| clock.elapsed)
    }

    /// Scheduling coordinate, independent of the chosen GetTime epoch.
    pub(crate) fn clock_now(&self) -> Instant {
        self.manual_clock
            .map_or_else(Instant::now, |clock| clock.now)
    }
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetManualClock", set_manual_clock)?
        .set_function("AdvanceTime", advance_time)
}

fn duration(seconds: f64) -> LuaResult<Duration> {
    Duration::try_from_secs_f64(seconds).map_err(|_| {
        runtime_error("manual clock seconds must be finite, nonnegative and representable")
    })
}

fn set_manual_clock(state: &mut LuaState) -> LuaResult<u32> {
    let elapsed = duration(f64::from_stack(state, 1)?)?;
    let mut sim = borrow_state_mut(state)?;
    if sim.manual_clock.is_some() {
        return Err(runtime_error("manual clock is already enabled"));
    }
    sim.manual_clock = Some(ManualClock {
        elapsed,
        now: Instant::now(),
    });
    Ok(0)
}

fn advance_time(state: &mut LuaState) -> LuaResult<u32> {
    let delta = f64::from_stack(state, 1)?;
    let duration = duration(delta)?;
    advance_clock(state, duration)?;
    #[cfg(feature = "retail-12-0-5")]
    super::execution_budget::reset_frame_budgets(state)?;
    let taints = super::taint::clear_active_stack_taint(state);
    let result = (|| {
        super::on_update::fire_state(state, delta)
            .map_err(|error| runtime_error(error.to_string()))?;
        super::manual_timers::drain(state)?;
        Ok(0)
    })();
    super::taint::restore_active_stack_taint(state, taints);
    result
}

fn advance_clock(state: &LuaState, delta: Duration) -> LuaResult<()> {
    let clock = borrow_state(state)?
        .manual_clock
        .ok_or_else(|| runtime_error("AdvanceTime requires SetManualClock"))?;
    let elapsed = clock
        .elapsed
        .checked_add(delta)
        .ok_or_else(|| runtime_error("manual elapsed time overflow"))?;
    let now = clock
        .now
        .checked_add(delta)
        .ok_or_else(|| runtime_error("manual scheduling time overflow"))?;
    borrow_state_mut(state)?.manual_clock = Some(ManualClock { elapsed, now });
    Ok(())
}
