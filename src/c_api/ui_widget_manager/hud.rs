//! Standard HUD widget-set selections.
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = crate::c_api::ensure_namespace(state, "C_UIWidgetManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTopCenterWidgetSetID",
        query_top_center_widget_set_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetBelowMinimapWidgetSetID",
        query_below_minimap_widget_set_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPowerBarWidgetSetID",
        query_power_bar_widget_set_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetObjectiveTrackerWidgetSetID",
        query_objective_tracker_widget_set_id,
    )
}

fn query_top_center_widget_set_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn query_below_minimap_widget_set_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(2.0));
    Ok(1)
}

fn query_power_bar_widget_set_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(283.0));
    Ok(1)
}

fn query_objective_tracker_widget_set_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(240.0));
    Ok(1)
}
