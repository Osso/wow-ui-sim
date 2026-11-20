//! INFERRED pending invite-confirmation state (native evidence requested as R88cf).

use std::collections::HashSet;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, registry_get,
    registry_set, table_get, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde::Deserialize;

const PENDING_INVITE_EVENT_CALLBACK: &str = "__pending_invite_confirmation_event_callback";

#[derive(Clone, Debug, Default)]
pub(crate) struct PendingInviteConfirmations {
    invites: Vec<PendingInviteConfirmation>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingInviteConfirmation {
    id: String,
    #[serde(default)]
    confirmation_type: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    guid: String,
    #[serde(default)]
    roles_invalid: bool,
    #[serde(default)]
    will_convert_to_raid: bool,
    level: Option<i32>,
    spec: Option<i32>,
    item_level: Option<f64>,
    #[serde(default)]
    is_cross_faction: bool,
    player_faction_group: Option<String>,
    localized_faction: Option<String>,
    #[serde(default)]
    invalid_queues: Vec<InvalidQueueInfo>,
    #[serde(default)]
    referral: InviteReferralInfo,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvalidQueueInfo {
    queue_type: Option<String>,
    map_name: Option<String>,
    battlefield_type: Option<String>,
    team_size: Option<i32>,
    is_brawl: Option<bool>,
    #[serde(default)]
    lfg_ids: Vec<i32>,
    lfg_list_id: Option<i64>,
    activity_id: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InviteReferralInfo {
    guid: Option<String>,
    name: Option<String>,
    relationship: Option<i32>,
    #[serde(default)]
    is_quick_join: bool,
    club_id: Option<i64>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_event_callback(state)?;
    let namespace = ensure_namespace(state, "C_PartyInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetInviteConfirmationInvalidQueues",
        get_invalid_queues,
    )?;
    table_set_rust_fn_static(state, namespace, "GetInviteReferralInfo", get_referral_info)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetNextPendingInviteConfirmation",
        next_pending,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetInviteConfirmationInfo",
        confirmation_info,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "RespondToInviteConfirmation",
        respond_to_invite_confirmation,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPendingInviteConfirmations", set_pending_invites)
}

fn set_pending_invites(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let invites: Vec<PendingInviteConfirmation> = serde_json::from_str(&input)
        .map_err(|_| runtime_error("SetPendingInviteConfirmations requires a JSON array"))?;
    validate_invites(&invites)?;
    let event_count = invites.len();
    borrow_state_mut(state)?.party_invite_confirmations.invites = invites;
    for _ in 0..event_count {
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "GROUP_INVITE_CONFIRMATION",
            &[],
        )?;
    }
    Ok(0)
}

fn validate_invites(invites: &[PendingInviteConfirmation]) -> LuaResult<()> {
    let mut ids = HashSet::new();
    for invite in invites {
        if invite.id.is_empty() || !ids.insert(&invite.id) {
            return Err(runtime_error(
                "pending invite ids must be non-empty and unique",
            ));
        }
    }
    Ok(())
}

fn next_pending(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = borrow_state(state)?
        .party_invite_confirmations
        .invites
        .first()
        .map(|invite| invite.id.clone());
    if let Some(invite_id) = invite_id {
        let value = create_string(state, &invite_id);
        state.push(value);
        Ok(1)
    } else {
        Ok(0)
    }
}

fn confirmation_info(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let Some(invite) = find_invite(state, &invite_id)? else {
        return Ok(0);
    };
    push_confirmation_info(state, &invite);
    Ok(11)
}

fn push_confirmation_info(state: &mut LuaState, invite: &PendingInviteConfirmation) {
    state.push(Val::Num(f64::from(invite.confirmation_type)));
    push_string(state, &invite.name);
    push_string(state, &invite.guid);
    state.push(Val::Bool(invite.roles_invalid));
    state.push(Val::Bool(invite.will_convert_to_raid));
    push_optional_i32(state, invite.level);
    push_optional_i32(state, invite.spec);
    push_optional_f64(state, invite.item_level);
    state.push(Val::Bool(invite.is_cross_faction));
    push_optional_string(state, invite.player_faction_group.as_deref());
    push_optional_string(state, invite.localized_faction.as_deref());
}

fn get_invalid_queues(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let Some(invite) = find_invite(state, &invite_id)? else {
        return Ok(0);
    };
    let queues = create_table(state);
    for (index, queue) in invite.invalid_queues.iter().enumerate() {
        let item = invalid_queue_to_lua(state, queue);
        set_table_array(state, queues, index as i64 + 1, item);
    }
    state.push(queues);
    Ok(1)
}

fn invalid_queue_to_lua(state: &mut LuaState, queue: &InvalidQueueInfo) -> Val {
    let table = create_table(state);
    set_optional_string_field(state, table, "queueType", queue.queue_type.as_deref());
    set_optional_string_field(state, table, "mapName", queue.map_name.as_deref());
    set_optional_string_field(
        state,
        table,
        "battlefieldType",
        queue.battlefield_type.as_deref(),
    );
    set_optional_i32_field(state, table, "teamSize", queue.team_size);
    set_optional_bool_field(state, table, "isBrawl", queue.is_brawl);
    let lfg_ids = create_table(state);
    for (index, id) in queue.lfg_ids.iter().enumerate() {
        set_table_array(state, lfg_ids, index as i64 + 1, Val::Num(f64::from(*id)));
    }
    table_set(state, table, "lfgIDs", lfg_ids);
    set_optional_i64_field(state, table, "lfgListID", queue.lfg_list_id);
    set_optional_i64_field(state, table, "activityID", queue.activity_id);
    table
}

fn get_referral_info(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let Some(invite) = find_invite(state, &invite_id)? else {
        return Ok(0);
    };
    push_optional_string(state, invite.referral.guid.as_deref());
    push_optional_string(state, invite.referral.name.as_deref());
    push_optional_i32(state, invite.referral.relationship);
    state.push(Val::Bool(invite.referral.is_quick_join));
    push_optional_i64(state, invite.referral.club_id);
    Ok(5)
}

fn respond_to_invite_confirmation(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let _accepted = bool::from_stack(state, 2)?;
    let has_pending_invites = remove_responded_invite(state, &invite_id)?;
    if has_pending_invites {
        schedule_confirmation_event(state)?;
    }
    Ok(0)
}

fn remove_responded_invite(state: &mut LuaState, invite_id: &str) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    sim.party_invite_confirmations
        .invites
        .retain(|invite| invite.id != invite_id);
    Ok(!sim.party_invite_confirmations.invites.is_empty())
}

fn schedule_confirmation_event(state: &mut LuaState) -> LuaResult<()> {
    let c_timer = table_get(state, Val::Table(state.global), "C_Timer");
    let after = table_get(state, c_timer, "After");
    let callback = registry_get(state, PENDING_INVITE_EVENT_CALLBACK);
    call_function_state(state, after, &[Val::Num(0.0), callback])?;
    Ok(())
}

fn register_event_callback(state: &mut LuaState) -> LuaResult<()> {
    let loader = table_get(state, Val::Table(state.global), "loadstring");
    let source = create_string(
        state,
        "return function() A_Admin.FireEvent('GROUP_INVITE_CONFIRMATION') end",
    );
    let chunk = call_function_state(state, loader, &[source])?;
    let callback = call_function_state(state, chunk, &[])?;
    registry_set(state, PENDING_INVITE_EVENT_CALLBACK, callback);
    Ok(())
}

fn find_invite(state: &LuaState, invite_id: &str) -> LuaResult<Option<PendingInviteConfirmation>> {
    Ok(borrow_state(state)?
        .party_invite_confirmations
        .invites
        .iter()
        .find(|invite| invite.id == invite_id)
        .cloned())
}

fn push_string(state: &mut LuaState, value: &str) {
    let value = create_string(state, value);
    state.push(value);
}

fn push_optional_string(state: &mut LuaState, value: Option<&str>) {
    if let Some(value) = value {
        push_string(state, value);
    } else {
        state.push(Val::Nil);
    }
}

fn push_optional_i32(state: &mut LuaState, value: Option<i32>) {
    state.push(value.map_or(Val::Nil, |value| Val::Num(f64::from(value))));
}

fn push_optional_i64(state: &mut LuaState, value: Option<i64>) {
    state.push(value.map_or(Val::Nil, |value| Val::Num(value as f64)));
}

fn push_optional_f64(state: &mut LuaState, value: Option<f64>) {
    state.push(value.map_or(Val::Nil, Val::Num));
}

fn set_optional_string_field(state: &mut LuaState, table: Val, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        let value = create_string(state, value);
        table_set(state, table, name, value);
    }
}

fn set_optional_i32_field(state: &mut LuaState, table: Val, name: &str, value: Option<i32>) {
    if let Some(value) = value {
        table_set(state, table, name, Val::Num(f64::from(value)));
    }
}

fn set_optional_i64_field(state: &mut LuaState, table: Val, name: &str, value: Option<i64>) {
    if let Some(value) = value {
        table_set(state, table, name, Val::Num(value as f64));
    }
}

fn set_optional_bool_field(state: &mut LuaState, table: Val, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        table_set(state, table, name, Val::Bool(value));
    }
}
