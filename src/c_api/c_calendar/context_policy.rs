//! Context-menu decisions over supplied calendar, guild and identity state.
use super::{CalendarContextPolicy, civil, day_event_record::CalendarDayEventRecord};
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "ContextMenuEventCanEdit", can_edit)?;
    table_set_rust_fn_static(state, namespace, "ContextMenuEventCanRemove", can_remove)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ContextMenuEventCanComplain",
        can_complain,
    )
}

fn read_coordinates(state: &mut LuaState, name: &str) -> LuaResult<Option<(i32, u32, usize)>> {
    let usage = format!(
        "Usage: local {} = C_Calendar.ContextMenuEvent{}(offsetMonths, monthDay, eventIndex)",
        match name {
            "CanEdit" => "canEdit",
            "CanRemove" => "canRemove",
            _ => "canComplain",
        },
        name
    );
    let offset = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= f64::from(i32::MIN) && *n <= f64::from(i32::MAX))
        .ok_or_else(|| runtime_error(usage.clone()))? as i32;
    let day = civil::numeric_argument(state, 2)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(usage.clone()))?;
    let index = civil::numeric_argument(state, 3)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(usage))?;
    let day = (day - 1.0) as i64;
    let index = (index - 1.0) as i64;
    if !(-1..=1).contains(&offset) || day < 0 || index < 0 {
        return Ok(None);
    }
    Ok(Some((offset, day as u32 + 1, index as usize)))
}

fn query(
    state: &mut LuaState,
    name: &str,
    decide: fn(&SimState, &CalendarDayEventRecord) -> bool,
) -> LuaResult<u32> {
    let coordinates = read_coordinates(state, name)?;
    let answer = {
        let host = borrow_state(state)?;
        coordinates
            .and_then(|(offset, day, index)| {
                let shown = host.calendar_displayed_month;
                let month =
                    i64::from(shown.year) * 12 + i64::from(shown.month) - 1 + i64::from(offset);
                let year = month.div_euclid(12) as i32;
                let month = (month.rem_euclid(12) + 1) as u8;
                if day > u32::from(civil::days_in_month(year, month)) {
                    return None;
                }
                host.calendar_day_events
                    .get(&(offset, day))
                    .and_then(|rows| rows.get(index))
            })
            .is_some_and(|row| decide(&host, row))
    };
    state.push(Val::Bool(answer));
    Ok(1)
}

fn can_edit(state: &mut LuaState) -> LuaResult<u32> {
    query(state, "CanEdit", |host, row| can_modify(host, row, false))
}
fn can_remove(state: &mut LuaState) -> LuaResult<u32> {
    query(state, "CanRemove", |host, row| can_modify(host, row, true))
}
fn can_complain(state: &mut LuaState) -> LuaResult<u32> {
    query(state, "CanComplain", complaint_allowed)
}

fn can_modify(host: &SimState, row: &CalendarDayEventRecord, removal: bool) -> bool {
    if row.category_flags & 0xc40 == 0 {
        return row.invite_role_markers & 0x6 != 0;
    }
    let policy = &host.calendar_context_policy;
    let Some(editable) = &policy.editable else {
        return false;
    };
    let player = existing_guid_for_unit(host, "player");
    if editable.category_flags & 0x800 == 0 {
        return editable.local_edit_credential.is_some_and(|id| id != 0)
            || player
                .as_deref()
                .is_some_and(|id| !id.is_empty() && id == editable.creator_identity);
    }
    if policy.guild_club_id.is_none() || policy.guild_club_id != editable.associated_id {
        return false;
    }
    let accepted_marker = if removal { 0x4 } else { 0x6 };
    row.invite_role_markers & accepted_marker != 0 || guild_privilege(host, policy, removal)
}

fn guild_privilege(host: &SimState, policy: &CalendarContextPolicy, removal: bool) -> bool {
    if !removal && !policy.edit_subsystem_available {
        return false;
    }
    if existing_guid_for_unit(host, "player").is_none()
        || host.world.guild_id == 0
        || policy.guild_club_id.is_none()
    {
        return false;
    }
    let right = if removal { 0x800000 } else { 0x100000 };
    policy.guild_rank_rights & right != 0
}

fn complaint_allowed(host: &SimState, row: &CalendarDayEventRecord) -> bool {
    let policy = &host.calendar_context_policy;
    if policy.action_pending || row.category_flags & 0xecc != 0 {
        return false;
    }
    let target = row
        .alternate_identity
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&row.primary_identity);
    if existing_guid_for_unit(host, "player")
        .as_deref()
        .is_some_and(|id| !id.is_empty() && id == target)
    {
        return false;
    }
    if host.cvars.get_bool("legacyFriendSystemEnabledClient")
        && host
            .social_friends
            .iter()
            .take(100)
            .any(|friend| !friend.guid.is_empty() && friend.guid == target)
    {
        return false;
    }
    policy.complaints_enabled || policy.complaint_policy != 0
}
