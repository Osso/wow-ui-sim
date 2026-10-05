//! Initiative publication and deferred service replies for the viewed neighborhood.

use super::model::{InitiativeActivityLogInfo, NeighborhoodInitiativeInfo};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
};
use crate::lua_api::next_timer_id;
use crate::lua_api::timer_layout::{RiluaPendingTimer, store_timer_callback};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, RustFn, Val};
use serde::Serialize;
use std::time::Instant;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, callback) in [
        ("GetActiveNeighborhood", get_active_neighborhood as RustFn),
        ("SetActiveNeighborhood", set_active_neighborhood),
        ("SetViewingNeighborhood", set_viewing_neighborhood),
        (
            "IsViewingActiveNeighborhood",
            is_viewing_active_neighborhood,
        ),
        ("GetNeighborhoodInitiativeInfo", get_initiative_info),
        ("GetInitiativeActivityLogInfo", get_activity_log),
        ("GetInitiativeTaskChatLink", get_task_chat_link),
        ("GetRequiredLevel", get_required_level),
        ("PlayerMeetsRequiredLevel", player_meets_required_level),
        ("PlayerHasInitiativeAccess", player_has_access),
        (
            "IsPlayerInNeighborhoodGroup",
            is_player_in_neighborhood_group,
        ),
        ("RequestNeighborhoodInitiativeInfo", request_initiative),
        ("RequestInitiativeActivityLog", request_activity_log),
    ] {
        table_set_rust_fn_static(state, namespace, name, callback)?;
    }
    Ok(())
}

fn get_active_neighborhood(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?
        .housing
        .initiative
        .active_neighborhood
        .clone();
    let value = create_string(state, &guid);
    state.push(value);
    Ok(1)
}

fn set_active_neighborhood(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let initiative = &mut sim.housing.initiative;
        let changed = initiative.active_neighborhood != guid;
        initiative.active_neighborhood = guid;
        changed
    };
    // INFERRED: refresh the dashboard after an active-neighborhood change;
    // unchanged writes must not recursively redispatch from handlers.
    if changed {
        dispatch_event_now(state, "NEIGHBORHOOD_INITIATIVE_UPDATED", &[])?;
    }
    Ok(0)
}

fn set_viewing_neighborhood(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    let initiative = &mut sim.housing.initiative;
    initiative.viewing_neighborhood = guid.clone();
    // INFERRED: an unknown neighborhood is in the choosing stage, never a
    // fabricated initiative. SetViewing does not deliver a service reply.
    initiative
        .neighborhoods
        .entry(guid.clone())
        .or_insert_with(|| NeighborhoodInitiativeInfo {
            neighborhood_guid: guid,
            ..Default::default()
        });
    Ok(0)
}

fn is_viewing_active_neighborhood(state: &mut LuaState) -> LuaResult<u32> {
    let viewing_active = {
        let sim = borrow_state(state)?;
        let initiative = &sim.housing.initiative;
        !initiative.active_neighborhood.is_empty()
            && initiative.active_neighborhood == initiative.viewing_neighborhood
    };
    state.push(Val::Bool(viewing_active));
    Ok(1)
}

fn get_initiative_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = {
        let sim = borrow_state(state)?;
        let initiative = &sim.housing.initiative;
        initiative
            .neighborhoods
            .get(&initiative.viewing_neighborhood)
            .cloned()
            .map(|mut info| {
                info.neighborhood_guid = initiative.viewing_neighborhood.clone();
                for task in &mut info.tasks {
                    task.tracked = sim.neighborhood_tracked_tasks.contains(&task.id);
                }
                info
            })
    };
    push_record(state, info)
}

fn get_activity_log(state: &mut LuaState) -> LuaResult<u32> {
    let info = {
        let sim = borrow_state(state)?;
        let initiative = &sim.housing.initiative;
        initiative
            .activity_logs
            .get(&initiative.viewing_neighborhood)
            .cloned()
            .map(|mut info| {
                info.neighborhood_guid = initiative.viewing_neighborhood.clone();
                info
            })
    };
    push_record(state, info)
}

fn get_task_chat_link(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let link = {
        let sim = borrow_state(state)?;
        let initiative = &sim.housing.initiative;
        initiative
            .neighborhoods
            .get(&initiative.active_neighborhood)
            .and_then(|info| info.tasks.iter().find(|task| task.id == id))
            .map(|task| task.chat_link.clone())
            .unwrap_or_default()
    };
    // INFERRED: unknown tasks or missing host links return the non-nil empty
    // string. No speculative native hyperlink encoding.
    let value = create_string(state, &link);
    state.push(value);
    Ok(1)
}

fn get_required_level(state: &mut LuaState) -> LuaResult<u32> {
    let level = borrow_state(state)?.housing.initiative.required_level;
    state.push(Val::Num(f64::from(level)));
    Ok(1)
}

fn player_meets_required_level(state: &mut LuaState) -> LuaResult<u32> {
    let meets = {
        let sim = borrow_state(state)?;
        sim.player.level >= sim.housing.initiative.required_level
    };
    state.push(Val::Bool(meets));
    Ok(1)
}

fn player_has_access(state: &mut LuaState) -> LuaResult<u32> {
    let access = borrow_state(state)?.housing.initiative.player_has_access;
    state.push(Val::Bool(access));
    Ok(1)
}

fn is_player_in_neighborhood_group(state: &mut LuaState) -> LuaResult<u32> {
    let member = {
        let sim = borrow_state(state)?;
        let initiative = &sim.housing.initiative;
        initiative
            .group_neighborhoods
            .contains(&initiative.active_neighborhood)
    };
    state.push(Val::Bool(member));
    Ok(1)
}

fn request_initiative(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?
        .housing
        .initiative
        .viewing_neighborhood
        .clone();
    borrow_state_mut(state)?
        .housing
        .initiative
        .neighborhoods
        .entry(guid.clone())
        .or_insert_with(|| NeighborhoodInitiativeInfo {
            neighborhood_guid: guid,
            ..Default::default()
        });
    defer_reply(state, deliver_initiative)
}

fn request_activity_log(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?
        .housing
        .initiative
        .viewing_neighborhood
        .clone();
    borrow_state_mut(state)?
        .housing
        .initiative
        .activity_logs
        .entry(guid.clone())
        .or_insert_with(|| InitiativeActivityLogInfo {
            neighborhood_guid: guid,
            ..Default::default()
        });
    defer_reply(state, deliver_activity_log)
}

fn defer_reply(state: &mut LuaState, deliver: RustFn) -> LuaResult<u32> {
    let id = next_timer_id();
    let callback = Val::Function(state.gc.alloc_closure(Closure::Rust(RustClosure::new(
        deliver,
        "NeighborhoodInitiative.Reply",
    ))));
    store_timer_callback(state, id, callback);
    let mut sim = borrow_state_mut(state)?;
    let guid = sim.housing.initiative.viewing_neighborhood.clone();
    sim.housing.initiative.pending_requests.insert(id, guid);
    let owner_addon = sim.loading_addon_index.or(sim.executing_addon_index);
    // INFERRED: timer delivery models the service round trip. Cached dashboard
    // OnShow registers listeners AFTER requesting. IDs capture the requested
    // neighborhood so changing the view cannot load the wrong record.
    sim.rilua_timers.push_back(RiluaPendingTimer {
        id,
        fire_at: Instant::now(),
        interval: None,
        remaining: None,
        cancelled: false,
        owner_addon,
        callback_receives_timer: true,
        callback_arg: Some(Val::Num(id as f64)),
    });
    Ok(0)
}

fn deliver_initiative(state: &mut LuaState) -> LuaResult<u32> {
    let Some(guid) = take_request(state)? else {
        return Ok(0);
    };
    let notify = {
        let mut sim = borrow_state_mut(state)?;
        let initiative = &mut sim.housing.initiative;
        if let Some(info) = initiative.neighborhoods.get_mut(&guid) {
            info.is_loaded = true;
        }
        initiative.viewing_neighborhood == guid
    };
    if notify {
        dispatch_event_now(state, "NEIGHBORHOOD_INITIATIVE_UPDATED", &[])?;
    }
    Ok(0)
}

fn deliver_activity_log(state: &mut LuaState) -> LuaResult<u32> {
    let Some(guid) = take_request(state)? else {
        return Ok(0);
    };
    let notify = {
        let mut sim = borrow_state_mut(state)?;
        let initiative = &mut sim.housing.initiative;
        if let Some(info) = initiative.activity_logs.get_mut(&guid) {
            info.is_loaded = true;
        }
        initiative.viewing_neighborhood == guid
    };
    if notify {
        dispatch_event_now(state, "INITIATIVE_ACTIVITY_LOG_UPDATED", &[])?;
    }
    Ok(0)
}

fn take_request(state: &mut LuaState) -> LuaResult<Option<String>> {
    let id = f64::from_stack(state, 1)? as u64;
    Ok(borrow_state_mut(state)?
        .housing
        .initiative
        .pending_requests
        .remove(&id))
}

fn push_record(state: &mut LuaState, record: Option<impl Serialize>) -> LuaResult<u32> {
    let value = match record {
        Some(record) => {
            let json = serde_json::to_value(record)
                .map_err(|error| rilua::runtime_error(format!("initiative record: {error}")))?;
            encode_record(state, json)
        }
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

/// DTO-only serialization; every nested table is stack-rooted until attached.
fn encode_record(state: &mut LuaState, json: serde_json::Value) -> Val {
    use serde_json::Value;
    match json {
        Value::Null => Val::Nil,
        Value::Bool(value) => Val::Bool(value),
        Value::Number(value) => Val::Num(value.as_f64().expect("integer DTO field")),
        Value::String(value) => create_string(state, &value),
        Value::Array(values) => encode_array(state, values),
        Value::Object(values) => encode_fields(state, values),
    }
}

fn encode_array(state: &mut LuaState, values: Vec<serde_json::Value>) -> Val {
    let result = create_table(state);
    state.push(result);
    let Val::Table(table) = result else {
        unreachable!()
    };
    for (index, value) in values.into_iter().enumerate() {
        let value = encode_record(state, value);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    state.top -= 1;
    result
}

fn encode_fields(state: &mut LuaState, values: serde_json::Map<String, serde_json::Value>) -> Val {
    let result = create_table(state);
    state.push(result);
    for (key, value) in values {
        let value = encode_record(state, value);
        table_set(state, result, &key, value);
    }
    state.top -= 1;
    result
}
