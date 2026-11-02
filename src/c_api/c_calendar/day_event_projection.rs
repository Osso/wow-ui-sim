//! Fresh, rooted Lua snapshots of host-owned Calendar day records.

use super::day_event_record::{CalendarDayEventRecord, CalendarTimeRecord};
use crate::lua_api::methods::{create_string, create_table, table_set_static};
use rilua::Val;
use rilua::vm::state::LuaState;

/// Push exactly one result, retaining the outer table as the result's stack root.
pub(super) fn push_day_event(state: &mut LuaState, record: Option<CalendarDayEventRecord>) {
    let Some(record) = record else {
        state.push(Val::Nil);
        return;
    };
    let table = create_table(state);
    state.push(table);
    publish_string_fields(state, table, &record);
    publish_number_fields(state, table, &record);
    publish_boolean_fields(state, table, &record);
    publish_time(state, table, "startTime", &record.start_time);
    publish_time(state, table, "endTime", &record.end_time);
}

fn publish_string_fields(state: &mut LuaState, table: Val, record: &CalendarDayEventRecord) {
    for (key, text) in [
        ("eventID", &record.event_id),
        ("clubID", &record.club_id),
        ("title", &record.title),
        ("calendarType", &record.calendar_type),
        ("sequenceType", &record.sequence_type),
        ("modStatus", &record.mod_status),
        ("invitedBy", &record.invited_by),
        ("difficultyName", &record.difficulty_name),
    ] {
        let value = create_string(state, text);
        table_set_static(state, table, key, value);
    }
}

fn publish_number_fields(state: &mut LuaState, table: Val, record: &CalendarDayEventRecord) {
    for (key, value) in [
        ("eventType", f64::from(record.event_type)),
        ("inviteStatus", f64::from(record.invite_status)),
        ("inviteType", f64::from(record.invite_type)),
        ("difficulty", record.difficulty),
        ("sequenceIndex", f64::from(record.sequence_index)),
        ("numSequenceDays", record.num_sequence_days),
    ] {
        table_set_static(state, table, key, Val::Num(value));
    }
    if let Some(icon) = record.icon_texture {
        table_set_static(state, table, "iconTexture", Val::Num(f64::from(icon)));
    }
}

fn publish_boolean_fields(state: &mut LuaState, table: Val, record: &CalendarDayEventRecord) {
    for (key, value) in [
        ("isCustomTitle", record.is_custom_title),
        ("dontDisplayBanner", record.dont_display_banner),
        ("dontDisplayEnd", record.dont_display_end),
        ("isLocked", record.is_locked),
    ] {
        table_set_static(state, table, key, Val::Bool(value));
    }
}

pub(super) fn publish_time(
    state: &mut LuaState,
    outer: Val,
    key: &'static str,
    time: &CalendarTimeRecord,
) {
    let table = create_table(state);
    state.push(table);
    for (field, value) in [
        ("monthDay", f64::from(time.month_day)),
        ("month", f64::from(time.month)),
        ("weekday", f64::from(time.weekday)),
        ("year", time.year),
        ("hour", time.hour),
        ("minute", time.minute),
    ] {
        table_set_static(state, table, field, Val::Num(value));
    }
    table_set_static(state, outer, key, table);
    state.pop();
}
