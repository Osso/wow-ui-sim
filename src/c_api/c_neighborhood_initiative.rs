//! Host-backed initiative records, neighborhood selection, request replies,
//! tracked task membership and reward scales.

pub mod model;
mod publication;

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    // Pinned Mists/Era/Anniversary NeighborhoodInitiativeDocumentation.lua:93-107
    // declare GetTrackedInitiativeTasks and IsInitiativeEnabled; the pinned 3.3.5
    // Wrath source references no C_NeighborhoodInitiative.
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_NeighborhoodInitiative")?;
    // Forever 1.60.1 pins the same NeighborhoodInitiativeDocumentation.lua as
    // Retail (byte-identical; publication members at lines 20-193).
    let forever = crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever;
    if cfg!(feature = "retail-12-0-0") || forever {
        publication::register(state, namespace)?;
    }
    for (name, callback) in [
        (
            "GetTrackedInitiativeTasks",
            get_tracked_tasks as rilua::RustFn,
        ),
        ("AddTrackedInitiativeTask", add_tracked_task),
        ("RemoveTrackedInitiativeTask", remove_tracked_task),
    ] {
        table_set_rust_fn_static(state, namespace, name, callback)?;
    }
    if cfg!(feature = "retail-12-1-0") || forever {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetInitiativeTaskRewardScaling",
            get_task_reward_scaling,
        )?;
    }
    Ok(())
}

/// Applies the task's host reward multiplier to a reward count. INFERRED:
/// scaled counts round down so a reward never exceeds its multiplied amount.
fn get_task_reward_scaling(state: &mut LuaState) -> LuaResult<u32> {
    let id = task_id(state)?;
    let Val::Num(num_items) = stack_val(state, 2) else {
        return Err(rilua::runtime_error(
            "GetInitiativeTaskRewardScaling requires a numeric item count",
        ));
    };
    let scale = borrow_state(state)?
        .neighborhood_task_reward_scales
        .get(&id)
        .copied()
        .unwrap_or(1.0);
    state.push(Val::Num((num_items * scale).floor()));
    Ok(1)
}

fn task_id(state: &LuaState) -> LuaResult<i32> {
    let Val::Num(id) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "Initiative tracking requires a numeric task ID",
        ));
    };
    Ok(id as i32)
}

fn add_tracked_task(state: &mut LuaState) -> LuaResult<u32> {
    let id = task_id(state)?;
    borrow_state_mut(state)?
        .neighborhood_tracked_tasks
        .insert(id);
    Ok(0)
}

fn remove_tracked_task(state: &mut LuaState) -> LuaResult<u32> {
    let id = task_id(state)?;
    borrow_state_mut(state)?
        .neighborhood_tracked_tasks
        .remove(&id);
    Ok(0)
}

fn get_tracked_tasks(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<_> = borrow_state(state)?
        .neighborhood_tracked_tasks
        .iter()
        .copied()
        .collect();
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, id) in ids.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(id as f64));
    }
    let result = create_table(state);
    table_set(state, result, "trackedIDs", array);
    state.push(result);
    Ok(1)
}
