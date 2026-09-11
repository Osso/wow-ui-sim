use super::{layout, model::EventState};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn refresh(state: &mut LuaState) -> LuaResult<()> {
    let changes = layout::refresh(&mut borrow_state_mut(state)?.encounter_timeline);
    emit(state, changes)
}

pub(super) fn emit(state: &mut LuaState, changes: layout::LayoutChanges) -> LuaResult<()> {
    for (id, position) in changes.tracks {
        let current = borrow_state(state)?
            .encounter_timeline
            .events
            .get(&id)
            .is_some_and(|event| {
                event.track == position.track && event.track_index == position.index
            });
        if current {
            event(state, "ENCOUNTER_TIMELINE_EVENT_TRACK_CHANGED", id)?;
        }
    }
    for id in changes.highlights {
        let current = borrow_state(state)?
            .encounter_timeline
            .events
            .get(&id)
            .is_some_and(|event| event.state == EventState::Active && layout::visible(event));
        if current {
            event(state, "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED", id)?;
            let active = borrow_state(state)?
                .encounter_timeline
                .events
                .get(&id)
                .is_some_and(|event| event.state == EventState::Active && layout::visible(event));
            if active {
                event(state, "ENCOUNTER_TIMELINE_EVENT_HIGHLIGHT", id)?;
            }
        }
    }
    Ok(())
}

pub(super) fn event(state: &mut LuaState, name: &str, id: u32) -> LuaResult<()> {
    dispatch_event_now(state, name, &[Val::Num(f64::from(id))])
}
