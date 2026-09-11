//! PTR script-event lifecycle. Queue/track/filter/EditMode policies remain separate.
mod model;
mod queries;
mod request;
mod timer;

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use model::EventState;
pub(crate) use model::Timeline;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_EncounterTimeline")?;
    table_set_rust_fn_static(state, namespace, "AddScriptEvent", add)?;
    table_set_rust_fn_static(state, namespace, "CancelAllScriptEvents", cancel_all)?;
    table_set_rust_fn_static(state, namespace, "CancelScriptEvent", |s| {
        change(s, EventState::Canceled)
    })?;
    table_set_rust_fn_static(state, namespace, "FinishScriptEvent", |s| {
        change(s, EventState::Finished)
    })?;
    table_set_rust_fn_static(state, namespace, "PauseScriptEvent", |s| {
        change(s, EventState::Paused)
    })?;
    table_set_rust_fn_static(state, namespace, "ResumeScriptEvent", |s| {
        change(s, EventState::Active)
    })?;
    queries::register(state, namespace)
}

fn add(state: &mut LuaState) -> LuaResult<u32> {
    let (info, paused) = request::read(state)?;
    let id = borrow_state_mut(state)?
        .encounter_timeline
        .add(info, paused)
        .ok_or_else(|| runtime_error("script event ID space exhausted"))?;
    let info = borrow_state(state)?.encounter_timeline.events[&id]
        .info
        .clone();
    let value = queries::info_table(state, &info);
    state.push(value);
    let result = dispatch_event_now(state, "ENCOUNTER_TIMELINE_EVENT_ADDED", &[value]);
    state.pop();
    result?;
    dispatch_event_now(state, "ENCOUNTER_TIMELINE_STATE_UPDATED", &[])?;
    state.push(Val::Num(f64::from(id)));
    Ok(1)
}

pub(super) fn read_id(state: &LuaState) -> LuaResult<u32> {
    match stack_val(state, 1) {
        Val::Num(id) if (0.0..=f64::from(u32::MAX)).contains(&id) && id.fract() == 0.0 => {
            Ok(id as u32)
        }
        _ => Err(runtime_error(
            "timeline event ID must be a nonnegative integer",
        )),
    }
}

fn change(state: &mut LuaState, next: EventState) -> LuaResult<u32> {
    let id = read_id(state)?;
    transition(state, id, next)?;
    Ok(0)
}

fn transition(state: &mut LuaState, id: u32, next: EventState) -> LuaResult<()> {
    let changed = borrow_state_mut(state)?
        .encounter_timeline
        .transition(id, next);
    if changed {
        dispatch_event_now(
            state,
            "ENCOUNTER_TIMELINE_EVENT_STATE_CHANGED",
            &[Val::Num(f64::from(id))],
        )?;
        dispatch_event_now(state, "ENCOUNTER_TIMELINE_STATE_UPDATED", &[])?;
    }
    Ok(())
}

fn cancel_all(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .encounter_timeline
        .events
        .keys()
        .copied()
        .collect::<Vec<_>>();
    for id in ids {
        transition(state, id, EventState::Canceled)?;
    }
    Ok(0)
}

pub(crate) fn begin_tick(state: &mut LuaState, elapsed: f64) -> LuaResult<()> {
    if !elapsed.is_finite() || elapsed < 0.0 {
        return Err(runtime_error(
            "timeline tick must be finite and nonnegative",
        ));
    }
    let ids = borrow_state_mut(state)?
        .encounter_timeline
        .begin_tick(elapsed);
    for id in ids {
        let active = borrow_state(state)?
            .encounter_timeline
            .events
            .get(&id)
            .is_some_and(|event| event.state == EventState::Active);
        if active {
            transition(state, id, EventState::Finished)?;
        }
    }
    Ok(())
}

pub(crate) fn end_tick(state: &mut LuaState) -> LuaResult<()> {
    let ids = borrow_state(state)?.encounter_timeline.expired_ids();
    for id in ids {
        let removed = borrow_state_mut(state)?
            .encounter_timeline
            .events
            .remove(&id)
            .is_some();
        if removed {
            dispatch_event_now(
                state,
                "ENCOUNTER_TIMELINE_EVENT_REMOVED",
                &[Val::Num(f64::from(id))],
            )?;
            dispatch_event_now(state, "ENCOUNTER_TIMELINE_STATE_UPDATED", &[])?;
        }
    }
    Ok(())
}
