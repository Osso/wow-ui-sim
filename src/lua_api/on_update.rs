//! Shared LuaState-level OnUpdate bridge for host and manual-clock ticks.
use super::methods::{borrow_state, borrow_state_mut};
use super::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, Val};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Incremental collector work units, not simulation time.
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
    fire_state(env.rilua_mut().state_mut(), elapsed)
}

pub(crate) fn fire_state(
    state: &mut LuaState,
    elapsed: f64,
) -> crate::Result<OnUpdateStageTimings> {
    #[cfg(feature = "retail-12-0-7")]
    crate::c_api::c_party_info::solo_1207::tick(state)?;
    let total_started = Instant::now();
    LuaApiMut::gc_stop(state);
    begin_tick(state, elapsed)?;
    let frame_ids = on_update_frame_ids(state)?;
    let mut timings = dispatch_tick_handlers(state, &frame_ids, elapsed)?;
    #[cfg(feature = "retail-12-0-5")]
    super::frame::methods::widgets::tick_countdown_formatters(state)?;
    #[cfg(feature = "retail-12-1-0")]
    crate::c_api::c_encounter_timeline::end_tick(state)?;
    let started = Instant::now();
    super::env_runtime::finalize_frame_metrics_state(state, elapsed * 1000.0);
    timings.finalize_metrics = started.elapsed();
    let started = Instant::now();
    LuaApiMut::gc_step(state, ON_UPDATE_GC_BUDGET)?;
    timings.gc_step = started.elapsed();
    timings.total = total_started.elapsed();
    Ok(timings)
}

fn begin_tick(state: &mut LuaState, elapsed: f64) -> crate::Result<()> {
    super::globals::real::loot_specialization::apply_pending(state)?;
    #[cfg(feature = "player-cast-durations")]
    super::channeling::tick(state)?;
    #[cfg(feature = "retail-12-1-0")]
    crate::c_api::c_encounter_timeline::begin_tick(state, elapsed)?;
    {
        let mut sim = borrow_state_mut(state)?;
        super::message_frame::update_wrath_message_frames(&mut sim, elapsed);
    }
    super::editbox_update::flush_state(state)?;
    super::script_helpers::reconcile_on_update_runtime_cache_if_dirty(state);
    Ok(())
}

fn on_update_frame_ids(state: &LuaState) -> crate::Result<Vec<u64>> {
    let sim = borrow_state(state)?;
    let mut ids = sim.on_update_frames.iter().copied().collect::<Vec<_>>();
    ids.sort_unstable();
    Ok(ids)
}

fn dispatch_tick_handlers(
    state: &mut LuaState,
    ids: &[u64],
    elapsed: f64,
) -> crate::Result<OnUpdateStageTimings> {
    let mut timings = OnUpdateStageTimings::default();
    let started = Instant::now();
    super::script_helpers::dispatch_on_update(state, ids, elapsed)?;
    timings.dispatch_handlers = started.elapsed();
    crate::c_api::duration_text_binding::tick(state, elapsed)?;
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::private_aura_sounds::playback::tick(state);
    let started = Instant::now();
    super::frame::methods::button_anchor_hierarchy::advance_animation_groups_state(state, elapsed)?;
    timings.animation_groups = started.elapsed();
    super::frame::methods::widgets::tick_scroll_range_updates(state)?;
    fire_layout_size_changes(state)?;
    let started = Instant::now();
    fire_on_post_update_handlers(state, ids, elapsed)?;
    timings.on_post_update = started.elapsed();
    Ok(timings)
}

/// Resolve dirty rectangles before dispatch; handler changes report next tick.
fn fire_layout_size_changes(state: &mut LuaState) -> crate::Result<()> {
    let changes = {
        let mut sim = borrow_state_mut(state)?;
        sim.ensure_layout_rects();
        sim.widgets.drain_size_changes()
    };
    for (id, (width, height)) in changes {
        super::env_events::fire_script_handler_state(
            state,
            id,
            "OnSizeChanged",
            vec![Val::Num(width as f64), Val::Num(height as f64)],
        )?;
    }
    Ok(())
}

fn fire_on_post_update_handlers(
    state: &mut LuaState,
    ids: &[u64],
    elapsed: f64,
) -> crate::Result<()> {
    let ids = super::script_helpers::frames_with_on_post_update(state, ids);
    for id in ids {
        super::env_events::fire_script_handler_state(
            state,
            id,
            "OnPostUpdate",
            vec![Val::Num(elapsed)],
        )?;
    }
    Ok(())
}
