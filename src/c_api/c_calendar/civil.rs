//! Supplied civil time and displayed Gregorian month.
use super::DisplayedMonth;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_table_with_fields,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "SetAbsMonth", set_abs_month)?;
    table_set_rust_fn_static(state, namespace, "GetMonthInfo", get_month_info)
}

pub(crate) fn current_calendar_time(state: &mut LuaState) -> LuaResult<u32> {
    let date = borrow_state(state)?.calendar_now;
    let table = create_table_with_fields(
        state,
        &[
            ("year", Val::Num(f64::from(date.year))),
            ("month", Val::Num(f64::from(date.month))),
            ("monthDay", Val::Num(f64::from(date.day))),
            (
                "weekday",
                Val::Num(f64::from(weekday(date.year, date.month, date.day))),
            ),
            ("hour", Val::Num(f64::from(date.hour))),
            ("minute", Val::Num(f64::from(date.minute))),
        ],
    );
    state.push(table);
    Ok(1)
}

pub(super) fn numeric_argument(state: &mut LuaState, index: i32) -> LuaResult<Option<f64>> {
    let value = stack_val(state, index);
    let value = rilua::table_security::unwrap_secret(state, value)?;
    match value {
        Val::Num(number) => Ok(Some(number)),
        Val::Str(_) => {
            let tonumber = crate::c_api::global_val(state, "tonumber");
            let result = call_function_state(state, tonumber, &[value])?;
            Ok(if let Val::Num(number) = result {
                Some(number)
            } else {
                None
            })
        }
        _ => Ok(None),
    }
}

fn set_abs_month(state: &mut LuaState) -> LuaResult<u32> {
    let month = numeric_argument(state, 1)?;
    let year = numeric_argument(state, 2)?;
    let valid_month = month.filter(|n| n.is_finite() && (0.0..=4294967295.0).contains(n));
    let valid_year =
        year.filter(|n| n.is_finite() && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(n));
    let (Some(month), Some(year)) = (valid_month, valid_year) else {
        return Err(runtime_error("Usage: C_Calendar.SetAbsMonth(month, year)"));
    };
    let converted_month = if month - 1.0 >= f64::from(i32::MAX) + 1.0 {
        i32::MIN
    } else {
        (month - 1.0) as i32
    };
    let current_year = borrow_state(state)?.calendar_now.year;
    let relative_year = (year as i32).wrapping_sub(2000);
    let lower = current_year.saturating_sub(2005);
    let upper = current_year.saturating_sub(1995);
    let selected_year = relative_year.clamp(lower, upper).clamp(4, 30) + 2000;
    let selected_month = ((converted_month % 12).unsigned_abs() + 1) as u8;
    borrow_state_mut(state)?.calendar_displayed_month = DisplayedMonth {
        year: selected_year,
        month: selected_month,
    };
    dispatch_event_now(state, "CALENDAR_UPDATE_EVENT_LIST", &[])?;
    Ok(0)
}

fn get_month_info(state: &mut LuaState) -> LuaResult<u32> {
    let offset = numeric_argument(state, 1)?.unwrap_or(0.0) as i32;
    let shown = borrow_state(state)?.calendar_displayed_month;
    let total = i64::from(shown.year) * 12 + i64::from(shown.month) - 1 + i64::from(offset);
    let year = total.div_euclid(12) as i32;
    let month = (total.rem_euclid(12) + 1) as u8;
    let table = create_table_with_fields(
        state,
        &[
            ("year", Val::Num(f64::from(year))),
            ("month", Val::Num(f64::from(month))),
            ("numDays", Val::Num(f64::from(days_in_month(year, month)))),
            ("firstWeekday", Val::Num(f64::from(weekday(year, month, 1)))),
        ],
    );
    state.push(table);
    Ok(1)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn weekday(year: i32, month: u8, day: u8) -> u8 {
    let offsets = [0_i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = i64::from(year);
    if month < 3 {
        y -= 1;
    }
    ((y + y.div_euclid(4) - y.div_euclid(100)
        + y.div_euclid(400)
        + offsets[usize::from(month - 1)]
        + i64::from(day))
    .rem_euclid(7)
        + 1) as u8
}
