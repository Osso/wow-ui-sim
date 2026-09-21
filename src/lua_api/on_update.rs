//! rilua-backed OnUpdate bridge.

use super::state::SimState;
#[cfg(feature = "retail-12-1-0")]
use rilua::LuaApiMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Per-tick GC work budget (rilua work units; `GCSTEPSIZE` is 1024).
/// Starting value — tune once we have real measurements. Smaller values
/// spread collection across more ticks but may fail to keep up with
/// allocation churn; larger values batch work at the cost of per-tick
/// pauses.
const ON_UPDATE_GC_BUDGET: i64 = 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct OnUpdateStageTimings {
    pub(crate) total: Duration,
    pub(crate) dispatch_handlers: Duration,
    pub(crate) animation_groups: Duration,
    pub(crate) on_post_update: Duration,
    pub(crate) finalize_metrics: Duration,
    pub(crate) gc_step: Duration,
}

pub(crate) fn register<T>(_lua: &T, _state: &Rc<RefCell<SimState>>) -> crate::Result<()> {
    Ok(())
}

pub(crate) fn fire(
    env: &super::env::WowLuaEnv,
    elapsed: f64,
) -> crate::Result<OnUpdateStageTimings> {
    let total_started = Instant::now();
    let mut timings = OnUpdateStageTimings::default();

    // Pause auto-GC while the tick runs its handlers — batches all
    // collection work into a single gc_step at the end of the tick
    // instead of interleaving mid-dispatch.
    env.gc_stop();

    #[cfg(feature = "player-cast-durations")]
    super::channeling::tick(env.rilua_mut().state_mut())?;

    #[cfg(feature = "retail-12-1-5")]
    crate::c_api::c_encounter_timeline::begin_tick(env.rilua_mut().state_mut(), elapsed)?;

    reconcile_runtime_cache(env);
    let frame_ids = on_update_frame_ids(env);

    {
        let started = Instant::now();
        dispatch_on_update_handlers(env, &frame_ids, elapsed)?;
        timings.dispatch_handlers = started.elapsed();
    }

    let started = Instant::now();
    advance_animation_groups(env, elapsed)?;
    timings.animation_groups = started.elapsed();

    let started = Instant::now();
    fire_on_post_update_handlers(env, &frame_ids, elapsed)?;
    timings.on_post_update = started.elapsed();

    #[cfg(feature = "retail-12-1-5")]
    crate::c_api::c_encounter_timeline::end_tick(env.rilua_mut().state_mut())?;

    let started = Instant::now();
    finalize_frame_metrics(env, elapsed);
    timings.finalize_metrics = started.elapsed();

    // Advance the collector with a bounded budget. gc_step updates the
    // threshold internally so the next tick starts with a reasonable
    // auto-GC ceiling before our gc_stop pauses it again.
    let started = Instant::now();
    env.gc_step_with_budget(ON_UPDATE_GC_BUDGET)?;
    timings.gc_step = started.elapsed();

    timings.total = total_started.elapsed();
    Ok(timings)
}

fn reconcile_runtime_cache(env: &super::env::WowLuaEnv) {
    let mut lua = env.rilua_mut();
    super::script_helpers::reconcile_on_update_runtime_cache_if_dirty(&mut lua);
}

fn on_update_frame_ids(env: &super::env::WowLuaEnv) -> Vec<u64> {
    let sim = env.state().borrow();
    let mut frame_ids = sim.on_update_frames.iter().copied().collect::<Vec<_>>();
    frame_ids.sort_unstable();
    frame_ids
}

fn dispatch_on_update_handlers(
    env: &super::env::WowLuaEnv,
    frame_ids: &[u64],
    elapsed: f64,
) -> crate::Result<()> {
    let mut lua = env.rilua_mut();
    super::script_helpers::dispatch_on_update(&mut lua, frame_ids, elapsed)?;
    Ok(())
}

fn advance_animation_groups(env: &super::env::WowLuaEnv, elapsed: f64) -> crate::Result<()> {
    crate::lua_api::frame::methods::button_anchor_hierarchy::advance_animation_groups(env, elapsed)
}

fn fire_on_post_update_handlers(
    env: &super::env::WowLuaEnv,
    frame_ids: &[u64],
    elapsed: f64,
) -> crate::Result<()> {
    for frame_id in frame_ids {
        env.fire_script_handler(*frame_id, "OnPostUpdate", vec![rilua::Val::Num(elapsed)])?;
    }
    Ok(())
}

fn finalize_frame_metrics(env: &super::env::WowLuaEnv, elapsed: f64) {
    env.finalize_frame_metrics(elapsed * 1000.0);
}
