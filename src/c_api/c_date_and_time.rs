//! Calendar comparison over supplied civil-time components.
//! Other date/time operations remain explicitly temporary providers.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::table_get;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::cmp::Ordering;

/// Weekday is derived metadata, not part of a civil timestamp's ordering.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct CalendarTime {
    year: i32,
    month: i32,
    month_day: i32,
    hour: i32,
    minute: i32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_DateAndTime")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CompareCalendarTime",
        compare_calendar_time,
    )
}

fn read_integer_field(state: &mut LuaState, data: Val, name: &str) -> LuaResult<i32> {
    let value = table_get(state, data, name);
    match rilua::table_security::unwrap_secret(state, value)? {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= f64::from(i32::MIN)
                && number <= f64::from(i32::MAX) =>
        {
            Ok(number as i32)
        }
        _ => Err(runtime_error(format!(
            "CalendarTime.{name} must be an integer"
        ))),
    }
}

fn read_calendar_time(state: &mut LuaState, argument: i32) -> LuaResult<CalendarTime> {
    let data = rilua::table_security::unwrap_secret(state, stack_val(state, argument))?;
    if !matches!(data, Val::Table(_)) {
        return Err(runtime_error(
            "CompareCalendarTime requires CalendarTime tables",
        ));
    }
    Ok(CalendarTime {
        year: read_integer_field(state, data, "year")?,
        month: read_integer_field(state, data, "month")?,
        month_day: read_integer_field(state, data, "monthDay")?,
        hour: read_integer_field(state, data, "hour")?,
        minute: read_integer_field(state, data, "minute")?,
    })
}

fn compare_calendar_time(state: &mut LuaState) -> LuaResult<u32> {
    let lhs = read_calendar_time(state, 1)?;
    let rhs = read_calendar_time(state, 2)?;
    // Blizzard's documented result orders rhs relative to lhs, not lhs to rhs.
    let comparison = match rhs.cmp(&lhs) {
        Ordering::Less => -1.0,
        Ordering::Equal => 0.0,
        Ordering::Greater => 1.0,
    };
    state.push(Val::Num(comparison));
    Ok(1)
}
