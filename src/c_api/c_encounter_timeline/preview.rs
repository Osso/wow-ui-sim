//! Repeating Edit Mode refresh owns three synthetic slots, never script events.
use super::{
    model::{EventInfo, EventState},
    notifications,
};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) const REFRESH_INTERVAL: f64 = 30.0;

pub(super) fn add(state: &mut LuaState) -> LuaResult<u32> {
    let existing = source_ids(state)?;
    for (slot, duration) in [8.0, 35.0, 90.0].into_iter().enumerate() {
        if let Some(&id) = existing.get(slot) {
            reset(state, id)?;
        } else {
            let info = fixture(slot, duration);
            let id = borrow_state_mut(state)?
                .encounter_timeline
                .add(info, false)
                .ok_or_else(|| runtime_error("preview event ID space exhausted"))?;
            super::announce_added(state, id)?;
        }
    }
    notifications::refresh(state)?;
    dispatch_event_now(state, "ENCOUNTER_TIMELINE_STATE_UPDATED", &[])?;
    state.push(Val::Num(REFRESH_INTERVAL));
    Ok(1)
}

fn source_ids(state: &LuaState) -> LuaResult<Vec<u32>> {
    Ok(borrow_state(state)?
        .encounter_timeline
        .events
        .iter()
        .filter_map(|(&id, event)| (event.info.source == 2).then_some(id))
        .collect())
}

fn reset(state: &mut LuaState, id: u32) -> LuaResult<()> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let now = sim.encounter_timeline.now.get();
        let Some(event) = sim.encounter_timeline.events.get_mut(&id) else {
            return Ok(());
        };
        let changed = event.state != EventState::Active;
        event.state = EventState::Active;
        event.terminal_tick = None;
        event.highlighted = false;
        let mut clock = event.clock.borrow_mut();
        clock.elapsed = 0.0;
        clock.running_since = Some(now);
        changed
    };
    notifications::refresh(state)?;
    notifications::event(state, "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED", id)?;
    if changed {
        notifications::event(state, "ENCOUNTER_TIMELINE_EVENT_STATE_CHANGED", id)?;
    }
    Ok(())
}

fn fixture(slot: usize, duration: f64) -> EventInfo {
    EventInfo {
        id: 0,
        source: 2,
        spell_id: 0,
        icon: 134400,
        name: format!("Simulator preview {}", slot + 1).into_bytes(),
        duration,
        max_queue_duration: REFRESH_INTERVAL,
        icons: 0,
        severity: slot as u8,
    }
}

pub(super) fn cancel(state: &mut LuaState) -> LuaResult<u32> {
    for id in source_ids(state)? {
        super::transition(state, id, EventState::Canceled)?;
    }
    Ok(0)
}
