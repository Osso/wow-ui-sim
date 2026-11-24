//! Supplied inputs for legacy performance globals consumed by Blizzard UI.
//!
//! INFERRED (R88fg, native evidence pending): omitted frame-rate input keeps the retired 60 FPS default;
//! the Forever helper's 240 values are sample count, not FPS.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetFramerate", get_framerate)?;
    table_set_rust_fn_static(state, state.global, "GetAddOnMemoryUsage", get_memory)?;
    table_set_rust_fn_static(state, state.global, "UpdateAddOnMemoryUsage", update_memory)?;
    table_set_rust_fn_static(state, state.global, "GetAddOnCPUUsage", get_cpu)?;
    table_set_rust_fn_static(state, state.global, "UpdateAddOnCPUUsage", update_cpu)?;
    table_set_rust_fn_static(state, state.global, "ResetCPUUsage", reset_cpu)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetFramerate", set_framerate)?
        .set_function("SetAddOnUsage", set_addon_usage)
}

fn get_framerate(state: &mut LuaState) -> LuaResult<u32> {
    let fps = borrow_state(state)?
        .app_frame_metrics
        .supplied_framerate
        .unwrap_or(60.0);
    state.push(Val::Num(fps));
    Ok(1)
}

fn set_framerate(state: &mut LuaState) -> LuaResult<u32> {
    let fps = f64::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .app_frame_metrics
        .supplied_framerate = Some(fps);
    Ok(0)
}

fn set_addon_usage(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let memory = f64::from_stack(state, 2)?;
    let cpu = f64::from_stack(state, 3)?;
    let mut sim = borrow_state_mut(state)?;
    let addon = sim
        .addons
        .iter_mut()
        .find(|addon| addon.folder_name == name)
        .ok_or_else(|| runtime_error(format!("unknown addon '{name}' in A_Admin.SetAddOnUsage")))?;
    addon.runtime.supplied_memory_kb = memory;
    addon.runtime.supplied_cpu_ms = cpu;
    Ok(0)
}

fn update_memory(state: &mut LuaState) -> LuaResult<u32> {
    for addon in &mut borrow_state_mut(state)?.addons {
        addon.runtime.memory_snapshot_kb = addon.runtime.supplied_memory_kb;
    }
    Ok(0)
}

fn update_cpu(state: &mut LuaState) -> LuaResult<u32> {
    for addon in &mut borrow_state_mut(state)?.addons {
        addon.runtime.cpu_snapshot_ms = addon.runtime.supplied_cpu_ms;
    }
    Ok(0)
}

fn reset_cpu(state: &mut LuaState) -> LuaResult<u32> {
    for addon in &mut borrow_state_mut(state)?.addons {
        addon.runtime.cpu_snapshot_ms = 0.0;
        addon.runtime.supplied_cpu_ms = 0.0;
    }
    Ok(0)
}

fn get_memory(state: &mut LuaState) -> LuaResult<u32> {
    let value = addon_usage(state, true)?;
    state.push(Val::Num(value));
    Ok(1)
}

fn get_cpu(state: &mut LuaState) -> LuaResult<u32> {
    let value = addon_usage(state, false)?;
    state.push(Val::Num(value));
    Ok(1)
}

fn addon_usage(state: &mut LuaState, memory: bool) -> LuaResult<f64> {
    let index = resolve_addon_index(state)?;
    let sim = borrow_state(state)?;
    Ok(index
        .and_then(|index| sim.addons.get(index))
        .map_or(0.0, |addon| snapshot_value(&addon.runtime, memory)))
}

fn resolve_addon_index(state: &mut LuaState) -> LuaResult<Option<usize>> {
    match Val::from_stack(state, 1)? {
        Val::Num(index) => Ok((index >= 1.0).then_some(index as usize - 1)),
        Val::Str(_) => {
            let name = String::from_stack(state, 1)?;
            Ok(borrow_state(state)?
                .addons
                .iter()
                .position(|addon| addon.folder_name == name))
        }
        _ => Ok(None),
    }
}

fn snapshot_value(runtime: &crate::lua_api::state_types::AddonRuntimeMetrics, memory: bool) -> f64 {
    if memory {
        runtime.memory_snapshot_kb
    } else {
        runtime.cpu_snapshot_ms
    }
}
