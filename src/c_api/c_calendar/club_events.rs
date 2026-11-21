//! Host-supplied club calendar event records.
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const INPUT_ERROR: &str = "A_Admin.SetCalendarClubEvents: expected an array of event records";

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarClubEventRecord {
    pub club_id: String,
    pub title: String,
    pub invite_status: i32,
    pub start_time: CalendarEventTime,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarEventTime {
    pub year: f64,
    pub month: f64,
    pub month_day: f64,
    pub weekday: f64,
    pub hour: f64,
    pub minute: f64,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCalendarClubEvents", set_club_events)
}

fn set_club_events(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let records = serde_json::from_str(&input).map_err(|_| runtime_error(INPUT_ERROR))?;
    borrow_state_mut(state)?.calendar_club_events = records;
    Ok(0)
}
