//! Host-supplied Calendar day records and synthetic event-index relations.
pub(crate) mod civil;
mod club;
mod context_policy;
mod context_reads;
mod day_event_projection;
mod day_event_queries;
pub mod day_event_record;
mod editable_access;
mod editable_flags;
mod editable_reads;
mod editable_status;
mod editable_time;
mod editable_title;
mod selection;
mod view_info;

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

/// Host-supplied civil time, independent of the displayed calendar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl Default for CivilDateTime {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 4,
            day: 14,
            hour: 12,
            minute: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayedMonth {
    pub year: i32,
    pub month: u8,
}

impl Default for DisplayedMonth {
    fn default() -> Self {
        let current = CivilDateTime::default();
        Self {
            year: current.year,
            month: current.month,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventIndexInfo {
    pub offset_months: i32,
    pub month_day: i32,
    pub event_index: i32,
}

/// Supplied inputs for context-menu policy decisions.
#[derive(Debug, Clone, Default)]
pub struct CalendarContextPolicy {
    pub action_pending: bool,
    pub complaints_enabled: bool,
    pub complaint_policy: i32,
    pub guild_club_id: Option<u64>,
    pub guild_rank_rights: u32,
    pub edit_subsystem_available: bool,
    pub announcement_edit_allowed: bool,
    pub editable: Option<CalendarEditableAccess>,
}

/// Access inputs from the separate being-edited event.
#[derive(Debug, Clone, Default)]
pub struct CalendarEditableAccess {
    pub event_id: String,
    pub category_flags: u32,
    pub associated_id: Option<u64>,
    pub creator_identity: String,
    pub title: Vec<u8>,
    pub description: Vec<u8>,
    pub time: Option<(u32, u32)>,
    pub auto_approve: bool,
    pub locked: bool,
    pub local_edit_credential: Option<u64>,
    pub dirty: bool,
    pub selected_invite_id: Option<String>,
    pub invites: Vec<CalendarInviteRecord>,
    pub sort_criterion: String,
    pub sort_reverse: bool,
    pub club_id: u64,
}

/// Host-supplied resolved texture metadata.
#[derive(Debug, Clone, Default)]
pub struct CalendarTextureRecord {
    pub title: String,
    pub icon_texture: u32,
    pub expansion_level: i32,
    pub difficulty_id: i32,
    pub map_id: i32,
    pub is_lfr: bool,
}

/// Host-supplied invite data for a being-edited event.
#[derive(Debug, Clone, Default)]
pub struct CalendarInviteRecord {
    pub invite_id: String,
    pub name: Option<String>,
    pub level: f64,
    pub class_name: Option<String>,
    pub class_filename: Option<String>,
    pub invite_status: Option<i32>,
    pub mod_status: Option<String>,
    pub invite_is_mine: bool,
    pub invite_type: i32,
    pub notes: String,
    pub class_id: Option<i32>,
    pub guid: String,
    pub response_time: Option<day_event_record::CalendarTimeRecord>,
}

/// Selected event identity and absolute calendar month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarSelection {
    pub event_id: String,
    pub month: i64,
    pub month_day: u32,
}

pub(crate) fn seeded_event_indices() -> HashMap<i32, CalendarEventIndexInfo> {
    serde_json::from_str(include_str!("../../data/calendar/demo-event-indices.json")).expect(
        "failed to parse calendar event index fixture data/calendar/demo-event-indices.json",
    )
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Calendar")?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        civil::register(state, namespace)?;
        club::register(state, namespace)?;
        selection::register(state, namespace)?;
        context_policy::register(state, namespace)?;
        context_reads::register(state, namespace)?;
        editable_reads::register(state, namespace)?;
        editable_status::register(state, namespace)?;
        editable_title::register(state, namespace)?;
        editable_time::register(state, namespace)?;
        editable_flags::register(state, namespace)?;
        view_info::register(state, namespace)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetFirstPendingInvite",
            day_event_queries::query_first_pending_invite,
        )?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "GetEventIndexInfo",
        query_event_index_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNumDayEvents",
        day_event_queries::query_num_day_events,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDayEvent",
        day_event_queries::query_day_event,
    )
}

fn read_query_arguments(state: &LuaState) -> LuaResult<(f64, Option<f64>, Option<f64>)> {
    let event_id = stack_val(state, 1);
    let month_offset = stack_val(state, 2);
    let month_day = stack_val(state, 3);
    // Authenticate every argument before any type validation or identity lookup.
    #[cfg(feature = "retail-12-1-0")]
    let (event_id, month_offset, month_day) = (
        rilua::table_security::unwrap_secret(state, event_id)?,
        rilua::table_security::unwrap_secret(state, month_offset)?,
        rilua::table_security::unwrap_secret(state, month_day)?,
    );
    let Val::Num(event_id) = event_id else {
        return Err(rilua::runtime_error(
            "C_Calendar.GetEventIndexInfo: eventID must be a number",
        ));
    };
    Ok((
        event_id,
        validate_optional_number(month_offset, "monthOffset")?,
        validate_optional_number(month_day, "monthDay")?,
    ))
}

fn validate_optional_number(value: Val, name: &str) -> LuaResult<Option<f64>> {
    match value {
        Val::Nil => Ok(None),
        Val::Num(number) => Ok(Some(number)),
        _ => Err(rilua::runtime_error(format!(
            "C_Calendar.GetEventIndexInfo: {name} must be a number or nil"
        ))),
    }
}

fn query_event_index_info(state: &mut LuaState) -> LuaResult<u32> {
    let (event_id, month_offset, month_day) = read_query_arguments(state)?;
    let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&event_id);
    let represented = event_id.is_finite() && event_id.fract() == 0.0 && in_range;
    let record = if represented {
        borrow_state(state)?
            .calendar_event_indices
            .get(&(event_id as i32))
            .copied()
    } else {
        None
    };
    let record = record.filter(|info| {
        let offset_matches = month_offset.is_none_or(|n| n == f64::from(info.offset_months));
        let day_matches = month_day.is_none_or(|n| n == f64::from(info.month_day));
        offset_matches && day_matches
    });
    push_event_index_info(state, record);
    Ok(1)
}

fn push_event_index_info(state: &mut LuaState, record: Option<CalendarEventIndexInfo>) {
    let Some(info) = record else {
        state.push(Val::Nil);
        return;
    };
    let result = create_table(state);
    state.push(result);
    for (key, value) in [
        ("offsetMonths", info.offset_months),
        ("monthDay", info.month_day),
        ("eventIndex", info.event_index),
    ] {
        table_set(state, result, key, Val::Num(f64::from(value)));
    }
}
