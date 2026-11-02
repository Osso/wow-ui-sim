//! Holiday and raid views of supplied calendar day rows.
use super::{civil, day_event_projection::publish_time, day_event_record::CalendarDayEventRecord};
use crate::lua_api::methods::{borrow_state, create_string, create_table};
use crate::lua_bridge::{table_set_rust_fn_static, table_set_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetHolidayInfo", holiday_info)?;
    table_set_rust_fn_static(state, namespace, "GetRaidInfo", raid_info)
}

fn read_row(state: &mut LuaState, method: &str) -> LuaResult<Option<CalendarDayEventRecord>> {
    let usage = match method {
        "GetHolidayInfo" => {
            "Usage: local event = C_Calendar.GetHolidayInfo(monthOffset, monthDay, index)"
        }
        _ => "Usage: local info = C_Calendar.GetRaidInfo(offsetMonths, monthDay, eventIndex)",
    };
    let offset = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= f64::from(i32::MIN) && *n <= f64::from(i32::MAX))
        .ok_or_else(|| runtime_error(usage))? as i32;
    let day = civil::numeric_argument(state, 2)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(usage))?;
    let index = civil::numeric_argument(state, 3)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(usage))?;
    let day = (day - 1.0) as i64;
    let index = (index - 1.0) as i64;
    if !(-1..=1).contains(&offset) || day < 0 || index < 0 {
        return Ok(None);
    }
    let day = day as u32 + 1;
    let host = borrow_state(state)?;
    let shown = host.calendar_displayed_month;
    let month = i64::from(shown.year) * 12 + i64::from(shown.month) - 1 + i64::from(offset);
    let year = month.div_euclid(12) as i32;
    let month = (month.rem_euclid(12) + 1) as u8;
    if day > u32::from(civil::days_in_month(year, month)) {
        return Ok(None);
    }
    Ok(host
        .calendar_day_events
        .get(&(offset, day))
        .and_then(|rows| rows.get(index as usize))
        .cloned())
}

fn set_text(state: &mut LuaState, table: Val, key: &'static str, value: &str) {
    let text = create_string(state, value);
    table_set_static(state, table, key, text);
}

fn holiday_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(row) = read_row(state, "GetHolidayInfo")? else {
        return Ok(0);
    };
    let Some(holiday) = row.holiday else {
        return Ok(0);
    };
    let table = create_table(state);
    state.push(table);
    set_text(state, table, "name", holiday.name.as_deref().unwrap_or(""));
    set_text(
        state,
        table,
        "description",
        holiday.description.as_deref().unwrap_or(""),
    );
    table_set_static(
        state,
        table,
        "texture",
        Val::Num(f64::from(holiday.texture)),
    );
    if holiday.multi_day && holiday.name.is_some() && holiday.description.is_some() {
        publish_time(state, table, "startTime", &row.start_time);
        publish_time(state, table, "endTime", &row.end_time);
    }
    Ok(1)
}

fn raid_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(row) = read_row(state, "GetRaidInfo")? else {
        return Ok(0);
    };
    let table = create_table(state);
    state.push(table);
    set_text(state, table, "name", row.raid_name.as_deref().unwrap_or(""));
    set_text(state, table, "calendarType", &row.calendar_type);
    table_set_static(state, table, "raidID", Val::Num(f64::from(row.raid_id)));
    table_set_static(state, table, "difficulty", Val::Num(row.difficulty));
    if !row.difficulty_name.is_empty() {
        set_text(state, table, "difficultyName", &row.difficulty_name);
    }
    publish_time(state, table, "time", &row.start_time);
    Ok(1)
}
