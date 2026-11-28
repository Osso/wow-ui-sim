//! INFERRED (R88de): return supplied event-type display records in supplied order.
use crate::c_api::helpers::set_table_array;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_fields,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str = "A_Admin.SetCalendarEventTypes: expected a JSON event-type array";

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarEventTypeDisplayInfo {
    pub display_string: String,
    pub event_type: i32,
}

pub(crate) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    // Retail and Forever CalendarDocumentation.lua:329.
    table_set_rust_fn_static(state, namespace, "EventGetTypes", event_types)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "EventGetTypesDisplayOrdered",
        event_types_display_ordered,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function("SetCalendarEventTypes", set_event_types)
    } else {
        Ok(builder)
    }
}

fn event_types_display_ordered(state: &mut LuaState) -> LuaResult<u32> {
    let records = borrow_state(state)?.calendar_event_types.clone();
    let result = create_table(state);
    for (index, record) in records.iter().enumerate() {
        let display_string = create_string(state, &record.display_string);
        let info = create_table_with_fields(
            state,
            &[
                ("displayString", display_string),
                ("eventType", Val::Num(f64::from(record.event_type))),
            ],
        );
        set_table_array(state, result, index as i64 + 1, info);
    }
    state.push(result);
    Ok(1)
}

fn set_event_types(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let records = serde_json::from_str::<Vec<CalendarEventTypeDisplayInfo>>(&input)
        .map_err(|error| runtime_error(format!("{INPUT_ERROR}: {error}")))?;
    borrow_state_mut(state)?.calendar_event_types = records;
    Ok(0)
}

fn event_types(state: &mut LuaState) -> LuaResult<u32> {
    let records = borrow_state(state)?.calendar_event_types.clone();
    let result = create_table(state);
    for (index, record) in records.iter().enumerate() {
        let name = create_string(state, &record.display_string);
        set_table_array(state, result, index as i64 + 1, name);
    }
    state.push(result);
    Ok(1)
}
