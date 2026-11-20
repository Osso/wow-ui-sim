//! INFERRED pending invite-confirmation state (native evidence requested as R88cf).

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub(crate) struct PendingInviteConfirmations {
    invites: Vec<Value>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PartyInfo")?;
    table_set_rust_fn_static(state, namespace, "ConfirmInviteUnit", confirm_invite_unit)?;
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
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPendingInviteConfirmations", set_pending_invites)
}

fn set_pending_invites(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let invites: Vec<Value> = serde_json::from_str(&input)
        .map_err(|_| runtime_error("SetPendingInviteConfirmations requires a JSON array"))?;
    validate_invites(&invites)?;
    let event_count = invites.len();
    borrow_state_mut(state)?.party_invite_confirmations.invites = invites;
    for _ in 0..event_count {
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "NEW_MATCHMAKING_PARTY_INVITE",
            &[],
        )?;
    }
    Ok(0)
}

fn validate_invites(invites: &[Value]) -> LuaResult<()> {
    let mut ids = std::collections::HashSet::new();
    for invite in invites {
        let Some(id) = invite.get("id").and_then(Value::as_str) else {
            return Err(runtime_error("pending invite requires a string id"));
        };
        if id.is_empty() || !ids.insert(id) {
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
        .and_then(|invite| invite.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned);
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
    let invite = find_invite(state, &invite_id)?;
    let Some(invite) = invite else { return Ok(0) };
    push_json_field(state, &invite, "confirmationType");
    for field in [
        "name",
        "guid",
        "rolesInvalid",
        "willConvertToRaid",
        "level",
        "spec",
        "itemLevel",
        "isCrossFaction",
        "playerFactionGroup",
        "localizedFaction",
    ] {
        push_json_field(state, &invite, field);
    }
    Ok(11)
}

fn get_invalid_queues(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let Some(invite) = find_invite(state, &invite_id)? else {
        return Ok(0);
    };
    let queues = invite
        .get("invalidQueues")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let value = value_to_lua(state, &queues);
    state.push(value);
    Ok(1)
}

fn get_referral_info(state: &mut LuaState) -> LuaResult<u32> {
    let invite_id = String::from_stack(state, 1)?;
    let Some(invite) = find_invite(state, &invite_id)? else {
        return Ok(0);
    };
    let referral = invite.get("referral").cloned().unwrap_or(Value::Null);
    for field in ["guid", "name", "relationship", "isQuickJoin", "clubId"] {
        push_json_field(state, &referral, field);
    }
    Ok(5)
}

fn confirm_invite_unit(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (R88cf pending): this local snapshot cannot model server-side group membership.
    let _target_name = String::from_stack(state, 1)?;
    Ok(0)
}

fn find_invite(state: &LuaState, invite_id: &str) -> LuaResult<Option<Value>> {
    Ok(borrow_state(state)?
        .party_invite_confirmations
        .invites
        .iter()
        .find(|invite| invite.get("id").and_then(Value::as_str) == Some(invite_id))
        .cloned())
}

fn push_json_field(state: &mut LuaState, object: &Value, field: &str) {
    let value = object
        .get(field)
        .map_or(Val::Nil, |value| value_to_lua(state, value));
    state.push(value);
}

fn value_to_lua(state: &mut LuaState, value: &Value) -> Val {
    match value {
        Value::Null => Val::Nil,
        Value::Bool(value) => Val::Bool(*value),
        Value::Number(value) => Val::Num(value.as_f64().unwrap_or_default()),
        Value::String(value) => create_string(state, value),
        Value::Array(values) => {
            let table = create_table(state);
            for (index, value) in values.iter().enumerate() {
                let item = value_to_lua(state, value);
                set_table_array(state, table, index as i64 + 1, item);
            }
            table
        }
        Value::Object(fields) => {
            let table = create_table(state);
            for (key, value) in fields {
                let item = value_to_lua(state, value);
                table_set(state, table, key, item);
            }
            table
        }
    }
}
