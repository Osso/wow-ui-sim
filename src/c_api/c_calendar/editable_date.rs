//! Local date edits on the supplied editable Calendar event.
use super::{
    civil::numeric_argument,
    editable_access::can_edit,
    editable_time::{read_component, report_invalid},
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetDate", set_date)
}

fn set_date(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Calendar.EventSetDate(month, monthDay, year)";
    let month = read_index(state, 1, USAGE)?;
    let day = read_index(state, 2, USAGE)?;
    let year = read_component(state, 3, USAGE)?.wrapping_sub(2000) as i32;
    let year = if year >= 2000 { year - 2000 } else { year };
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
        if event.date != Some((month, day, year)) {
            event.dirty = true;
        }
        if month < 12 && day < 31 && year <= 31 {
            event.date = Some((month, day, year));
            return Ok(0);
        }
    }
    report_invalid(state, "CALENDAR_ERROR_INVALID_DATE")
}

fn read_index(state: &mut LuaState, index: i32, usage: &str) -> LuaResult<i32> {
    let number = numeric_argument(state, index)?
        .filter(|n| n.is_finite() && (0.0..=f64::from(u32::MAX)).contains(n))
        .ok_or_else(|| runtime_error(usage.to_owned()))?
        - 1.0;
    if number >= f64::from(i32::MAX) + 1.0 {
        Ok(i32::MIN)
    } else {
        Ok(number as i32)
    }
}
