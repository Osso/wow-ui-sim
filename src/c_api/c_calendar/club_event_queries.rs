//! INFERRED (R88dd): return supplied club events with start times in the inclusive request range, in supplied order.
use crate::c_api::c_calendar::day_event_record::{CalendarDayEventRecord, CalendarTimeRecord};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_table, table_get, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

type DateTimeKey = (f64, f64, f64, f64, f64);

pub(crate) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetClubCalendarEvents",
        club_calendar_events,
    )
}

fn club_calendar_events(state: &mut LuaState) -> LuaResult<u32> {
    let club_id = read_club_id(state)?;
    let start = read_time(state, 2)?;
    let end = read_time(state, 3)?;
    let records = borrow_state(state)?.calendar_club_events.clone();
    let result = create_table(state);
    let mut index = 1_i64;
    for event in records
        .iter()
        .filter(|event| is_in_range(event, &club_id, start, end))
    {
        let projected = project_event(state, event);
        set_table_array(state, result, index, projected);
        index += 1;
    }
    state.push(result);
    Ok(1)
}

fn is_in_range(
    event: &CalendarDayEventRecord,
    club_id: &str,
    start: DateTimeKey,
    end: DateTimeKey,
) -> bool {
    event.club_id == club_id
        && time_key(&event.start_time) >= start
        && time_key(&event.start_time) <= end
}

fn read_club_id(state: &LuaState) -> LuaResult<String> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    val_to_string(state, value)
        .ok_or_else(|| runtime_error("C_Calendar.GetClubCalendarEvents: clubId must be a string"))
}

fn read_time(state: &mut LuaState, index: i32) -> LuaResult<DateTimeKey> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error(
            "C_Calendar.GetClubCalendarEvents: time must be a table",
        ));
    }
    Ok((
        time_field(state, value, "year")?,
        time_field(state, value, "month")?,
        time_field(state, value, "monthDay")?,
        time_field(state, value, "hour")?,
        time_field(state, value, "minute")?,
    ))
}

fn time_field(state: &mut LuaState, table: Val, key: &'static str) -> LuaResult<f64> {
    match table_get(state, table, key) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(runtime_error(
            "C_Calendar.GetClubCalendarEvents: invalid time field",
        )),
    }
}

fn time_key(time: &CalendarTimeRecord) -> DateTimeKey {
    (
        time.year,
        f64::from(time.month),
        f64::from(time.month_day),
        time.hour,
        time.minute,
    )
}

fn project_event(state: &mut LuaState, event: &CalendarDayEventRecord) -> Val {
    super::day_event_projection::push_day_event(state, Some(event.clone()));
    let projected = stack_val(state, -1);
    state.pop();
    projected
}
