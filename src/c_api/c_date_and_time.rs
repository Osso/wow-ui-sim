//! Calendar comparison over supplied civil-time components.
//! Other date/time operations remain explicitly temporary providers.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::cmp::Ordering;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_WEEK: i64 = 604_800;
const DAYS_PER_400_YEARS: i64 = 146_097;
const UNIX_EPOCH_DAY_OFFSET: i64 = 719_468;

/// INFERRED (R88df): default regional reset schedule; hosts can supply the region's values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WeeklyResetSchedule {
    pub weekday: u8,
    pub hour: u8,
}

impl Default for WeeklyResetSchedule {
    fn default() -> Self {
        Self {
            weekday: 3,
            hour: 7,
        }
    }
}

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
        state.global,
        "__wow_current_calendar_time",
        crate::c_api::c_calendar::civil::current_calendar_time,
    )?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetCurrentCalendarTime",
            crate::c_api::c_calendar::civil::current_calendar_time,
        )?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "CompareCalendarTime",
        compare_calendar_time,
    )?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetWeeklyResetStartTime",
            weekly_reset_start,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetSecondsUntilWeeklyReset",
            seconds_until_weekly_reset,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetSecondsUntilDailyReset",
            seconds_until_daily_reset,
        )?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        builder.set_function("SetWeeklyResetSchedule", set_weekly_reset_schedule)
    } else {
        Ok(builder)
    }
}

fn set_weekly_reset_schedule(state: &mut LuaState) -> LuaResult<u32> {
    let weekday = schedule_number(state, 1, 1, 7, "weekday")? as u8;
    let hour = schedule_number(state, 2, 0, 23, "hour")? as u8;
    borrow_state_mut(state)?.weekly_reset_schedule = WeeklyResetSchedule { weekday, hour };
    Ok(0)
}

fn schedule_number(state: &LuaState, index: i32, min: i32, max: i32, name: &str) -> LuaResult<i32> {
    let Val::Num(value) = stack_val(state, index) else {
        return Err(runtime_error(format!(
            "A_Admin.SetWeeklyResetSchedule: {name} must be an integer in {min}..{max}"
        )));
    };
    if !value.is_finite()
        || value.fract() != 0.0
        || value < f64::from(min)
        || value > f64::from(max)
    {
        return Err(runtime_error(format!(
            "A_Admin.SetWeeklyResetSchedule: {name} must be an integer in {min}..{max}"
        )));
    }
    Ok(value as i32)
}

fn weekly_reset_start(state: &mut LuaState) -> LuaResult<u32> {
    let start = weekly_reset_timestamp(state)?;
    state.push(Val::Num(start as f64));
    Ok(1)
}

fn seconds_until_weekly_reset(state: &mut LuaState) -> LuaResult<u32> {
    let now = unix_timestamp(borrow_state(state)?.calendar_now);
    let start = weekly_reset_timestamp(state)?;
    state.push(Val::Num((start + 7 * 86_400 - now) as f64));
    Ok(1)
}

fn seconds_until_daily_reset(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let now = unix_timestamp(sim.calendar_now);
    let hour = i64::from(sim.weekly_reset_schedule.hour);
    let day = now.div_euclid(SECONDS_PER_DAY);
    let reset = day * SECONDS_PER_DAY + hour * SECONDS_PER_HOUR;
    let next_reset = if reset > now {
        reset
    } else {
        reset + SECONDS_PER_DAY
    };
    state.push(Val::Num((next_reset - now) as f64));
    Ok(1)
}

fn weekly_reset_timestamp(state: &mut LuaState) -> LuaResult<i64> {
    let sim = borrow_state(state)?;
    let now = unix_timestamp(sim.calendar_now);
    let schedule = sim.weekly_reset_schedule;
    let day = now.div_euclid(SECONDS_PER_DAY);
    let weekday = weekday_from_unix_day(day);
    let days_since_reset = (weekday + 7 - schedule.weekday) % 7;
    let mut start = (day - i64::from(days_since_reset)) * SECONDS_PER_DAY
        + i64::from(schedule.hour) * SECONDS_PER_HOUR;
    if start > now {
        start -= SECONDS_PER_WEEK;
    }
    Ok(start)
}

fn weekday_from_unix_day(day: i64) -> u8 {
    ((day + 4).rem_euclid(7) + 1) as u8
}

fn unix_timestamp(time: crate::c_api::c_calendar::CivilDateTime) -> i64 {
    let adjusted_year = i64::from(time.year) - i64::from(time.month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month = i64::from(time.month);
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(time.day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * DAYS_PER_400_YEARS + day_of_era - UNIX_EPOCH_DAY_OFFSET;
    days_since_epoch * SECONDS_PER_DAY
        + i64::from(time.hour) * SECONDS_PER_HOUR
        + i64::from(time.minute) * SECONDS_PER_MINUTE
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
