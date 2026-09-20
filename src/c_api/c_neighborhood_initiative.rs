//! Tracked initiative membership only; task records and events remain unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !cfg!(any(
        feature = "retail-12-0-0",
        feature = "client-wowforever"
    )) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_NeighborhoodInitiative")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTrackedInitiativeTasks",
        get_tracked_tasks,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AddTrackedInitiativeTask",
        add_tracked_task,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "RemoveTrackedInitiativeTask",
        remove_tracked_task,
    )
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
