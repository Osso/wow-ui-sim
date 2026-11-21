//! Host-supplied Calendar event-type display records.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const INPUT_ERROR: &str = "A_Admin.SetCalendarEventTypes: expected a JSON event-type array";

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarEventTypeDisplayInfo {
    pub display_string: String,
    pub event_type: i32,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function("SetCalendarEventTypes", set_event_types)
    } else {
        Ok(builder)
    }
}

fn set_event_types(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let records = serde_json::from_str::<Vec<CalendarEventTypeDisplayInfo>>(&input)
        .map_err(|error| runtime_error(format!("{INPUT_ERROR}: {error}")))?;
    borrow_state_mut(state)?.calendar_event_types = records;
    Ok(0)
}
