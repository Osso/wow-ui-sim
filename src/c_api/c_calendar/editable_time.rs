//! Local time edits on the supplied editable Calendar event.
use super::{civil::numeric_argument, editable_access::can_edit};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetTime", set_time)
}

fn set_time(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Calendar.EventSetTime(hour, minute)";
    let hour = read_component(state, 1, USAGE)?;
    let minute = read_component(state, 2, USAGE)?;
    let permitted = {
        let host = borrow_state(state)?;
        can_edit(&host)
    };
    if permitted {
        let mut host = borrow_state_mut(state)?;
        let event = host
            .calendar_context_policy
            .editable
            .as_mut()
            .expect("editable event checked above");
        if event.time != Some((hour, minute)) {
            event.dirty = true;
        }
        if (hour as i32) < 24 && (minute as i32) < 60 {
            event.time = Some((hour, minute));
            return Ok(0);
        }
    }
    let message = create_string(state, "CALENDAR_ERROR_INVALID_TIME");
    dispatch_event_now(state, "CALENDAR_UPDATE_ERROR", &[message])?;
    borrow_state_mut(state)?
        .calendar_context_policy
        .action_pending = false;
    dispatch_event_now(state, "CALENDAR_ACTION_PENDING", &[Val::Bool(false)])?;
    Ok(0)
}

fn read_component(state: &mut LuaState, index: i32, usage: &str) -> LuaResult<u32> {
    let number = numeric_argument(state, index)?
        .filter(|n| n.is_finite() && (0.0..=f64::from(u32::MAX)).contains(n))
        .ok_or_else(|| runtime_error(usage.to_owned()))?;
    Ok(number as u32)
}
