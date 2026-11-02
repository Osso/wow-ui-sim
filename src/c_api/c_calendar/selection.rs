//! Opened and context-menu selections resolve against current supplied day rows.
use super::{
    CalendarEventIndexInfo, CalendarSelection, DisplayedMonth, civil, push_event_index_info,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetEventIndex", get_opened)?;
    table_set_rust_fn_static(state, namespace, "ContextMenuGetEventIndex", get_context)?;
    table_set_rust_fn_static(state, namespace, "OpenEvent", open_event)?;
    table_set_rust_fn_static(state, namespace, "ContextMenuSelectEvent", select_context)?;
    table_set_rust_fn_static(state, namespace, "CloseEvent", close_event)
}

fn month_number(month: DisplayedMonth) -> i64 {
    i64::from(month.year) * 12 + i64::from(month.month) - 1
}

fn read_position(state: &mut LuaState, name: &str) -> LuaResult<(i32, u32, usize)> {
    let mut values = [0.0; 3];
    for (index, value) in values.iter_mut().enumerate() {
        *value = civil::numeric_argument(state, index as i32 + 1)?
            .filter(|n| n.is_finite() && n.fract() == 0.0)
            .ok_or_else(|| {
                runtime_error(format!(
                    "Usage: C_Calendar.{name}(offsetMonths, monthDay, eventIndex)"
                ))
            })?;
    }
    if values[0] < f64::from(i32::MIN)
        || values[0] > f64::from(i32::MAX)
        || values[1] < 1.0
        || values[1] > f64::from(u32::MAX)
        || values[2] < 1.0
        || values[2] > f64::from(u32::MAX)
    {
        return Err(runtime_error(format!(
            "Usage: C_Calendar.{name}(offsetMonths, monthDay, eventIndex)"
        )));
    }
    Ok((values[0] as i32, values[1] as u32, values[2] as usize - 1))
}

fn find_selection(state: &mut LuaState, name: &str) -> LuaResult<Option<CalendarSelection>> {
    let (offset, day, index) = read_position(state, name)?;
    let host = borrow_state(state)?;
    Ok(host
        .calendar_day_events
        .get(&(offset, day))
        .and_then(|rows| rows.get(index))
        .map(|row| CalendarSelection {
            event_id: row.event_id.clone(),
            month: month_number(host.calendar_displayed_month) + i64::from(offset),
            month_day: day,
        }))
}

fn open_event(state: &mut LuaState) -> LuaResult<u32> {
    let selection = find_selection(state, "OpenEvent")?;
    let found = selection.is_some();
    if found {
        borrow_state_mut(state)?.calendar_opened_selection = selection;
    }
    state.push(Val::Bool(found));
    Ok(1)
}

fn select_context(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(selection) = find_selection(state, "ContextMenuSelectEvent")? {
        borrow_state_mut(state)?.calendar_context_selection = Some(selection);
    }
    Ok(0)
}

fn close_event(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.calendar_opened_selection = None;
    Ok(0)
}

fn get_opened(state: &mut LuaState) -> LuaResult<u32> {
    get_selection(state, false)
}
fn get_context(state: &mut LuaState) -> LuaResult<u32> {
    get_selection(state, true)
}

fn get_selection(state: &mut LuaState, context: bool) -> LuaResult<u32> {
    let result = {
        let host = borrow_state(state)?;
        let selection = if context {
            &host.calendar_context_selection
        } else {
            &host.calendar_opened_selection
        };
        selection.as_ref().and_then(|selection| {
            let offset = selection.month - month_number(host.calendar_displayed_month);
            if !(-1..=1).contains(&offset) {
                return None;
            }
            let rows = host
                .calendar_day_events
                .get(&(offset as i32, selection.month_day))?;
            let index = rows
                .iter()
                .position(|row| row.event_id == selection.event_id)?;
            Some(CalendarEventIndexInfo {
                offset_months: offset as i32,
                month_day: selection.month_day as i32,
                event_index: index as i32 + 1,
            })
        })
    };
    if let Some(info) = result {
        push_event_index_info(state, Some(info));
        Ok(1)
    } else {
        Ok(0)
    }
}
