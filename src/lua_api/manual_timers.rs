//! Deadline-ordered drain used only with an explicit manual environment clock.
use super::methods::borrow_state_mut;
use super::state::PendingTimer;
use super::timer_layout::{get_timer_callback, remove_timer_callback};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

pub(crate) fn drain(state: &mut LuaState) -> LuaResult<usize> {
    let eligible: HashSet<u64> = borrow_state_mut(state)?
        .rilua_timers
        .iter()
        .map(|timer| timer.id)
        .collect();
    let mut fired = drain_signals(state)?;
    let mut zero_interval_fired = HashSet::new();
    while let Some(mut timer) = take_next_due(state, &eligible, &zero_interval_fired)? {
        let callback = get_timer_callback(state, timer.id);
        if matches!(callback, Val::Nil) {
            continue;
        }
        super::timer_dispatch::fire(
            state,
            timer.owner_addon,
            callback,
            timer.id,
            timer.callback_receives_timer,
            timer.callback_arg,
        );
        fired += 1;
        if timer.interval.is_some_and(|interval| interval.is_zero()) {
            zero_interval_fired.insert(timer.id);
        }
        // Cancellation deletes the registered callback, including self-cancellation.
        let cancelled = matches!(get_timer_callback(state, timer.id), Val::Nil);
        let deadline = timer.fire_at;
        if !cancelled && super::timer_processing::reschedule_timer(&mut timer, deadline) {
            borrow_state_mut(state)?.rilua_timers.push_back(timer);
        } else {
            remove_timer_callback(state, timer.id);
        }
    }
    Ok(fired)
}

fn take_next_due(
    state: &LuaState,
    eligible: &HashSet<u64>,
    blocked: &HashSet<u64>,
) -> LuaResult<Option<PendingTimer>> {
    let mut sim = borrow_state_mut(state)?;
    let now = sim.clock_now();
    let index = sim
        .rilua_timers
        .iter()
        .enumerate()
        .filter(|(_, timer)| {
            !timer.cancelled
                && timer.fire_at <= now
                && eligible.contains(&timer.id)
                && !blocked.contains(&timer.id)
        })
        .min_by_key(|(_, timer)| (timer.fire_at, timer.id))
        .map(|(index, _)| index);
    Ok(index.and_then(|index| sim.rilua_timers.remove(index)))
}

#[cfg(feature = "timed-signal-maps")]
fn drain_signals(state: &mut LuaState) -> LuaResult<usize> {
    let signals = {
        let mut sim = borrow_state_mut(state)?;
        crate::c_api::timed_signal_map::take_due_signals(&mut sim)
    };
    let mut fired = 0;
    for signal in signals {
        let callback = get_timer_callback(state, signal.id);
        if !matches!(callback, Val::Nil) {
            super::timer_dispatch::fire(
                state,
                signal.owner_addon,
                callback,
                signal.id,
                false,
                Some(Val::Num(signal.key as f64)),
            );
            fired += 1;
        }
    }
    Ok(fired)
}

#[cfg(not(feature = "timed-signal-maps"))]
fn drain_signals(_state: &mut LuaState) -> LuaResult<usize> {
    Ok(0)
}
