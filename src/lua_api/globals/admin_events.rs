//! Rilua A_Admin handlers — Events, Debug toggles.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Events ────────────────────────────────────────────────────────────────────

pub(super) fn fire_event_admin(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::stack_val;

    let event_name = String::from_stack(state, 1)?;
    let nargs = state.top as i32 - state.base as i32;
    let mut event_args = Vec::new();
    for i in 2..=nargs {
        event_args.push(stack_val(state, i));
    }
    if event_name == "COMBAT_TEXT_UPDATE" {
        crate::c_api::c_combat_text::dispatch_supplied_event(state, &event_args)?;
    } else {
        dispatch_event_now(state, &event_name, &event_args)?;
    }
    Ok(0)
}

// ── Debug toggles ─────────────────────────────────────────────────────────────

pub(super) fn toggle_debug_borders(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.debug_borders = !st.debug_borders;
    st.invalidate_strata_buckets();
    let result = st.debug_borders;
    drop(st);
    state.push(Val::Bool(result));
    Ok(1)
}

pub(super) fn toggle_debug_anchors(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.debug_anchors = !st.debug_anchors;
    st.invalidate_strata_buckets();
    let result = st.debug_anchors;
    drop(st);
    state.push(Val::Bool(result));
    Ok(1)
}
