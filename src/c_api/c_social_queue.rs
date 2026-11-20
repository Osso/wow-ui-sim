//! INFERRED Quick Join data supplied by the simulator host (R88bz pending).
//! Signatures follow both profiles' SocialQueueDocumentation.lua (GetAllGroups
//! 11-23, GetConfig 27-34, group queries 37-107, events 142-159).

use std::collections::HashSet;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SocialQueueState {
    groups: Vec<Value>,
    config: Value,
    system_enabled: bool,
    system_supported: bool,
}

impl Default for SocialQueueState {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            config: Value::Object(Map::new()),
            system_enabled: false,
            system_supported: false,
        }
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_SocialQueue")?;
    table_set_rust_fn_static(state, namespace, "GetAllGroups", get_all_groups)?;
    table_set_rust_fn_static(state, namespace, "GetConfig", get_config)?;
    table_set_rust_fn_static(state, namespace, "GetGroupForPlayer", get_group_for_player)?;
    table_set_rust_fn_static(state, namespace, "GetGroupInfo", get_group_info)?;
    table_set_rust_fn_static(state, namespace, "GetGroupMembers", get_group_members)?;
    table_set_rust_fn_static(state, namespace, "GetGroupQueues", get_group_queues)?;
    table_set_rust_fn_static(state, namespace, "IsSystemEnabled", is_system_enabled)?;
    table_set_rust_fn_static(state, namespace, "IsSystemSupported", is_system_supported)?;
    table_set_rust_fn_static(state, namespace, "RequestToJoin", request_to_join)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SignalToastDisplayed",
        signal_toast_displayed,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetSocialQueueGroups", set_social_queue_groups)?
        .set_function("SetSocialQueueConfig", set_social_queue_config)
}

fn set_social_queue_groups(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let groups = parse_groups(&input)?;
    let changed = changed_group_ids(&borrow_state(state)?.social_queue.groups, &groups);
    borrow_state_mut(state)?.social_queue.groups = groups;
    for guid in changed {
        let guid = create_string(state, &guid);
        dispatch_event_now(state, "SOCIAL_QUEUE_UPDATE", &[guid, Val::Nil])?;
    }
    Ok(0)
}

fn set_social_queue_config(state: &mut LuaState) -> LuaResult<u32> {
    let input = String::from_stack(state, 1)?;
    let config: Value = serde_json::from_str(&input)
        .map_err(|_| runtime_error("SetSocialQueueConfig requires a JSON object"))?;
    if !config.is_object() {
        return Err(runtime_error("SetSocialQueueConfig requires a JSON object"));
    }
    borrow_state_mut(state)?.social_queue.config = config;
    dispatch_event_now(state, "SOCIAL_QUEUE_CONFIG_UPDATED", &[])?;
    Ok(0)
}

fn parse_groups(input: &str) -> LuaResult<Vec<Value>> {
    let groups: Vec<Value> = serde_json::from_str(input)
        .map_err(|_| runtime_error("SetSocialQueueGroups requires a JSON group array"))?;
    let mut guids = HashSet::new();
    for group in &groups {
        let guid = validate_group(group)?;
        if !guids.insert(guid) {
            return Err(runtime_error(
                "SetSocialQueueGroups requires unique group guids",
            ));
        }
    }
    Ok(groups)
}

fn validate_group(group: &Value) -> LuaResult<String> {
    let guid = group["guid"]
        .as_str()
        .ok_or_else(|| runtime_error("SetSocialQueueGroups group requires a string guid"))?;
    let info = &group["info"];
    let required_booleans = [
        "canJoin",
        "needTank",
        "needHealer",
        "needDamage",
        "isSoloQueueParty",
        "questSessionActive",
    ];
    if !info["numQueues"].is_number()
        || !info["leaderGUID"].is_string()
        || required_booleans
            .iter()
            .any(|field| !info[field].is_boolean())
        || group["members"].as_array().is_none()
        || group["queues"].as_array().is_none()
    {
        return Err(runtime_error(
            "SetSocialQueueGroups group has an invalid shape",
        ));
    }
    Ok(guid.to_owned())
}

fn changed_group_ids(old: &[Value], new: &[Value]) -> Vec<String> {
    let mut changed = Vec::new();
    let mut seen = HashSet::new();
    for group in old.iter().chain(new) {
        let Some(guid) = group["guid"].as_str() else {
            continue;
        };
        let current = new.iter().find(|row| row["guid"].as_str() == Some(guid));
        let previous = old.iter().find(|row| row["guid"].as_str() == Some(guid));
        if current != previous && seen.insert(guid.to_owned()) {
            changed.push(guid.to_owned());
        }
    }
    changed
}

fn get_all_groups(state: &mut LuaState) -> LuaResult<u32> {
    let allow_non_joinable = stack_true(state, 1);
    let allow_non_queued = stack_true(state, 2);
    let groups = borrow_state(state)?.social_queue.groups.clone();
    let result = create_table(state);
    let mut index = 0;
    for group in &groups {
        if include_group(group, allow_non_joinable, allow_non_queued) {
            index += 1;
            let value = create_string(state, group["guid"].as_str().unwrap_or_default());
            set_table_array(state, result, index, value);
        }
    }
    state.push(result);
    Ok(1)
}

fn include_group(group: &Value, allow_non_joinable: bool, allow_non_queued: bool) -> bool {
    let info = &group["info"];
    (allow_non_joinable || info["canJoin"].as_bool().unwrap_or(false))
        && (allow_non_queued || info["numQueues"].as_i64().unwrap_or(0) > 0)
}

fn get_config(state: &mut LuaState) -> LuaResult<u32> {
    let config = borrow_state(state)?.social_queue.config.clone();
    let value = to_lua(state, &config);
    state.push(value);
    Ok(1)
}

fn get_group_for_player(state: &mut LuaState) -> LuaResult<u32> {
    let player_guid = String::from_stack(state, 1)?;
    let groups = borrow_state(state)?.social_queue.groups.clone();
    let found = groups.iter().find_map(|group| {
        let members = group["members"].as_array()?;
        if !members
            .iter()
            .any(|member| member["guid"].as_str() == Some(player_guid.as_str()))
        {
            return None;
        }
        Some((
            group["guid"].as_str()?.to_owned(),
            group["info"]["isSoloQueueParty"].as_bool().unwrap_or(false),
        ))
    });
    let Some((guid, solo)) = found else {
        return Ok(0);
    };
    let guid = create_string(state, &guid);
    state.push(guid);
    state.push(Val::Bool(solo));
    Ok(2)
}

fn get_group_info(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let group = borrow_state(state)?
        .social_queue
        .groups
        .iter()
        .find(|group| group["guid"].as_str() == Some(guid.as_str()))
        .cloned();
    let Some(group) = group else { return Ok(0) };
    let info = &group["info"];
    let can_join = Val::Bool(info["canJoin"].as_bool().unwrap_or(false));
    let num_queues = Val::Num(info["numQueues"].as_f64().unwrap_or(0.0));
    state.push(can_join);
    state.push(num_queues);
    for field in [
        "needTank",
        "needHealer",
        "needDamage",
        "isSoloQueueParty",
        "questSessionActive",
    ] {
        state.push(Val::Bool(info[field].as_bool().unwrap_or(false)));
    }
    let leader_guid = json_scalar(state, &info["leaderGUID"]);
    state.push(leader_guid);
    Ok(8)
}

fn get_group_members(state: &mut LuaState) -> LuaResult<u32> {
    push_group_array(state, "members")
}

fn get_group_queues(state: &mut LuaState) -> LuaResult<u32> {
    push_group_array(state, "queues")
}

fn push_group_array(state: &mut LuaState, field: &str) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let group = borrow_state(state)?
        .social_queue
        .groups
        .iter()
        .find(|group| group["guid"].as_str() == Some(guid.as_str()))
        .cloned();
    let Some(rows) = group.and_then(|group| group[field].as_array().cloned()) else {
        return Ok(0);
    };
    let result = create_table(state);
    for (index, row) in rows.iter().enumerate() {
        let value = to_lua(state, row);
        set_table_array(state, result, index as i64 + 1, value);
    }
    state.push(result);
    Ok(1)
}

fn is_system_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.social_queue.system_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn is_system_supported(state: &mut LuaState) -> LuaResult<u32> {
    let supported = borrow_state(state)?.social_queue.system_supported;
    state.push(Val::Bool(supported));
    Ok(1)
}

fn request_to_join(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let can_join = borrow_state(state)?
        .social_queue
        .groups
        .iter()
        .find(|group| group["guid"].as_str() == Some(guid.as_str()))
        .is_some_and(|group| group["info"]["canJoin"].as_bool().unwrap_or(false));
    state.push(Val::Bool(can_join));
    Ok(1)
}

fn signal_toast_displayed(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn stack_true(state: &LuaState, index: i32) -> bool {
    matches!(crate::lua_bridge::stack_val(state, index), Val::Bool(true))
}

fn json_scalar(state: &mut LuaState, value: &Value) -> Val {
    match value {
        Value::String(value) => create_string(state, value),
        Value::Number(value) => Val::Num(value.as_f64().unwrap_or(0.0)),
        Value::Bool(value) => Val::Bool(*value),
        _ => Val::Nil,
    }
}

fn to_lua(state: &mut LuaState, value: &Value) -> Val {
    match value {
        Value::Object(fields) => {
            let table = create_table(state);
            for (key, value) in fields {
                let value = to_lua(state, value);
                table_set(state, table, key, value);
            }
            table
        }
        Value::Array(values) => {
            let table = create_table(state);
            for (index, value) in values.iter().enumerate() {
                let value = to_lua(state, value);
                set_table_array(state, table, index as i64 + 1, value);
            }
            table
        }
        _ => json_scalar(state, value),
    }
}
