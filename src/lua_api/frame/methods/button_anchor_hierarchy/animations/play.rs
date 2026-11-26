//! Playback state is committed before dispatching reentrant OnPlay handlers.
use super::{
    apply_group_flipbook_state, apply_group_vertex_color_state, refresh_active_animation_group,
    resolve_animation_group_id, sync_action_bar_busy_for_group,
};
use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack, frame_ref};
use crate::lua_api::script_helpers::{call_void_function_state, get_scripts_for_dispatch};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn animation_group_play(state: &mut LuaState) -> LuaResult<u32> {
    let caller_id = frame_id_from_stack(state, 1)?;
    let reverse = match stack_val(state, 2) {
        Val::Nil => None,
        value => Some(!matches!(value, Val::Bool(false))),
    };
    let offset = match stack_val(state, 3) {
        Val::Num(value) => value,
        _ => 0.0,
    };
    let notification = start_group(state, caller_id, reverse, offset)?;
    if let Some(frame_id) = notification {
        let frame = frame_ref(state, frame_id)?;
        for handler in get_scripts_for_dispatch(state, frame_id, "OnPlay") {
            call_void_function_state(state, handler, &[frame]).map_err(rilua::runtime_error)?;
        }
    }
    Ok(0)
}

fn start_group(
    state: &LuaState,
    caller_id: u64,
    reverse: Option<bool>,
    offset: f64,
) -> LuaResult<Option<u64>> {
    let mut sim = borrow_state_mut(state)?;
    let Some(group_id) = resolve_animation_group_id(&sim, caller_id) else {
        return Ok(None);
    };
    let notification = if let Some(group) = sim.animation_groups.get_mut(&group_id) {
        let started = !group.playing;
        let resuming = group.paused;
        let reverse = reverse.unwrap_or(resuming && group.reverse);
        if started && !resuming {
            // INFERRED (R88g7): offset is elapsed timeline time; reverse mirrors it.
            group.elapsed = animation_group_start_elapsed(group, offset, reverse);
            for animation in &mut group.animations {
                animation.elapsed = 0.0;
            }
        }
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
    apply_group_vertex_color_state(&mut sim, group_id);
    super::apply_group_motion_state(&mut sim, group_id);
    sync_action_bar_busy_for_group(&mut sim, group_id);
    Ok(notification)
}

fn animation_group_start_elapsed(
    group: &crate::lua_api::animation::AnimGroupState,
    offset: f64,
    reverse: bool,
) -> f64 {
    let duration = group.total_duration().max(0.0);
    if duration == 0.0 {
        return 0.0;
    }
    let elapsed = match group.looping {
        crate::lua_api::animation::LoopType::None => offset.clamp(0.0, duration),
        _ => offset.rem_euclid(duration),
    };
    if reverse { duration - elapsed } else { elapsed }
}
