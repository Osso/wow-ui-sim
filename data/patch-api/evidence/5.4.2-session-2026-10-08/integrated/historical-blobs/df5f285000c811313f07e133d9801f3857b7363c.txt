//! WoW taint integration for rilua.

use rilua::LuaApiMut;
use rilua::vm::state::LuaState;

/// Enable taint tracking on the rilua state.
pub fn enable_taint_mode(lua: &mut rilua::Lua) {
    lua.state_mut().taint_mode = true;
}

/// Stamp addon taint on a compiled function.
///
/// Rilua owns generation-safe weak closure keys, so collected closures cannot
/// transfer their taint to a later allocation in the same arena slot.
pub fn stamp_addon_taint(
    lua: &mut rilua::Lua,
    func: &rilua::Function,
    addon_name: &str,
) -> rilua::LuaResult<()> {
    stamp_addon_taint_state(lua.state_mut(), func, addon_name)
}

pub fn stamp_addon_taint_state(
    state: &mut LuaState,
    func: &rilua::Function,
    addon_name: &str,
) -> rilua::LuaResult<()> {
    rilua::stdlib::taint::set_closure_taint(state, func.gc_ref(), Some(addon_name))
}

/// Set taint for the current frame before dispatching a script handler.
pub fn set_frame_taint(state: &mut LuaState, addon_name: Option<&str>) {
    if let Some(ci) = state.call_stack.get_mut(state.ci) {
        ci.taint = addon_name.map(|s| s.to_string());
    }
}

/// Clear taint for secure (Blizzard) code execution.
pub fn clear_frame_taint(state: &mut LuaState) {
    if let Some(ci) = state.call_stack.get_mut(state.ci) {
        ci.taint = None;
    }
}

pub fn clear_active_stack_taint(state: &mut LuaState) -> Vec<Option<String>> {
    let active_depth = state.ci.saturating_add(1);
    let mut saved_taints = Vec::with_capacity(active_depth);
    for call_info in state.call_stack.iter_mut().take(active_depth) {
        saved_taints.push(call_info.taint.clone());
        call_info.taint = None;
    }
    saved_taints
}

pub fn restore_active_stack_taint(state: &mut LuaState, saved_taints: Vec<Option<String>>) {
    for (call_info, taint) in state.call_stack.iter_mut().zip(saved_taints) {
        call_info.taint = taint;
    }
}
