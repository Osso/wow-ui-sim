//! Host-supplied club calendar event records.
use crate::c_api::c_calendar::day_event_record::CalendarDayEventRecord;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const INPUT_ERROR: &str = "A_Admin.SetCalendarClubEvents: expected a JSON event array";

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function("SetCalendarClubEvents", set_club_events)
    } else {
        Ok(builder)
    }
}

fn set_club_events(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let records = serde_json::from_str::<Vec<CalendarDayEventRecord>>(&input)
        .map_err(|error| runtime_error(format!("{INPUT_ERROR}: {error}")))?;
    borrow_state_mut(state)?.calendar_club_events = records;
    Ok(0)
}
