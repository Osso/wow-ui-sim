//! Playback state is committed before dispatching reentrant OnPlay handlers.
use super::{
    apply_group_flipbook_state, refresh_active_animation_group, resolve_animation_group_id,
    sync_action_bar_busy_for_group,
};
use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack, frame_ref};
use crate::lua_api::script_helpers::{call_void_function_state, get_scripts_for_dispatch};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn animation_group_play(state: &mut LuaState) -> LuaResult<u32> {
    let caller_id = frame_id_from_stack(state, 1)?;
    let reverse = !matches!(stack_val(state, 2), Val::Nil | Val::Bool(false));
    let notification = start_group(state, caller_id, reverse)?;
    if let Some(frame_id) = notification {
        let frame = frame_ref(state, frame_id)?;
        for handler in get_scripts_for_dispatch(state, frame_id, "OnPlay") {
            call_void_function_state(state, handler, &[frame]).map_err(rilua::runtime_error)?;
        }
    }
    Ok(0)
}

fn start_group(state: &LuaState, caller_id: u64, reverse: bool) -> LuaResult<Option<u64>> {
    let mut sim = borrow_state_mut(state)?;
    let Some(group_id) = resolve_animation_group_id(&sim, caller_id) else {
        return Ok(None);
    };
    let notification = if let Some(group) = sim.animation_groups.get_mut(&group_id) {
        let started = !group.playing;
        group.playing = true;
        group.paused = false;
        group.done = false;
        group.pending_finish = false;
        group.reverse = reverse;
        if started { group.frame_id } else { None }
    } else {
        None
    };
    refresh_active_animation_group(&mut sim, group_id);
    apply_group_flipbook_state(&mut sim, group_id);
    sync_action_bar_busy_for_group(&mut sim, group_id);
    Ok(notification)
}
