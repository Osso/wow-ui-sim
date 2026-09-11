use super::{layout, notifications};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn get(state: &mut LuaState) -> LuaResult<u32> {
    let view = borrow_state(state)?.encounter_timeline.view;
    state.push(Val::Num(f64::from(view)));
    Ok(1)
}

pub(super) fn set(state: &mut LuaState) -> LuaResult<u32> {
    let next = match stack_val(state, 1) {
        Val::Num(value) if (0.0..=2.0).contains(&value) && value.fract() == 0.0 => value as u8,
        _ => {
            return Err(runtime_error(
                "timeline view must be None, Timeline, or Bars",
            ));
        }
    };
    let (old, changes) = {
        let mut sim = borrow_state_mut(state)?;
        let timeline = &mut sim.encounter_timeline;
        let old = timeline.view;
        if old == next {
            return Ok(0);
        }
        timeline.view = next;
        (old, layout::refresh(timeline))
    };
    notify_view(state, old, next, changes)?;
    Ok(0)
}

fn notify_view(
    state: &mut LuaState,
    old: u8,
    next: u8,
    changes: layout::LayoutChanges,
) -> LuaResult<()> {
    dispatch_event_now(
        state,
        "ENCOUNTER_TIMELINE_VIEW_DEACTIVATED",
        &[Val::Num(f64::from(old))],
    )?;
    if borrow_state(state)?.encounter_timeline.view != next {
        return Ok(());
    }
    notifications::emit(state, changes)?;
    dispatch_event_now(state, "ENCOUNTER_TIMELINE_LAYOUT_UPDATED", &[])?;
    if borrow_state(state)?.encounter_timeline.view == next {
        if next != 0 {
            dispatch_event_now(
                state,
                "ENCOUNTER_TIMELINE_VIEW_ACTIVATED",
                &[Val::Num(f64::from(next))],
            )?;
        }
        dispatch_event_now(state, "ENCOUNTER_TIMELINE_STATE_UPDATED", &[])?;
    }
    Ok(())
}
