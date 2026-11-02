//! Guild-list rematching and context-menu clipboard/type reads.
use super::{civil, push_event_index_info, selection::resolve_selection};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetGuildEventSelectionInfo",
        guild_event_selection,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ContextMenuEventClipboard",
        clipboard_exists,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ContextMenuEventGetCalendarType",
        context_calendar_type,
    )
}

fn guild_event_selection(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local info = C_Calendar.GetGuildEventSelectionInfo(index)";
    let index = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(USAGE))?;
    let index = (index - 1.0) as i64;
    let info = {
        let host = borrow_state(state)?;
        usize::try_from(index)
            .ok()
            .and_then(|index| host.calendar_guild_selections.get(index))
            .and_then(|selection| resolve_selection(&host, selection))
    };
    if let Some(info) = info {
        push_event_index_info(state, Some(info));
        Ok(1)
    } else {
        Ok(0)
    }
}

fn clipboard_exists(state: &mut LuaState) -> LuaResult<u32> {
    let exists = borrow_state(state)?.calendar_clipboard.is_some();
    state.push(Val::Bool(exists));
    Ok(1)
}

fn context_calendar_type(state: &mut LuaState) -> LuaResult<u32> {
    let calendar_type = {
        let host = borrow_state(state)?;
        host.calendar_context_selection
            .as_ref()
            .and_then(|selection| resolve_selection(&host, selection))
            .and_then(|info| {
                host.calendar_day_events
                    .get(&(info.offset_months, info.month_day as u32))
                    .and_then(|rows| rows.get(info.event_index as usize - 1))
            })
            .map(|row| row.calendar_type.clone())
    };
    let value = match calendar_type {
        Some(name) => {
            let recognized = matches!(
                name.as_str(),
                "PLAYER"
                    | "GUILD_ANNOUNCEMENT"
                    | "GUILD_EVENT"
                    | "COMMUNITY_EVENT"
                    | "SYSTEM"
                    | "HOLIDAY"
                    | "RAID_LOCKOUT"
                    | "RAID_RESET"
            );
            create_string(state, if recognized { &name } else { "" })
        }
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}
