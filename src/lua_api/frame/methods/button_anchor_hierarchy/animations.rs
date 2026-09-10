//! Animation group and animation creation/control methods.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, frame_ref};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

mod creation;
mod fields;
mod runtime;

pub(super) use creation::{
    animation_config_noop, create_animation, create_animation_group, create_control_point,
    set_scale_dispatch,
};
pub(super) use fields::{
    animation_get_duration, animation_get_flipbook_columns, animation_get_flipbook_frame_height,
    animation_get_flipbook_frame_width, animation_get_flipbook_frames, animation_get_flipbook_rows,
    animation_set_duration, animation_set_end_delay, animation_set_flipbook_columns,
    animation_set_flipbook_frame_height, animation_set_flipbook_frame_width,
    animation_set_flipbook_frames, animation_set_flipbook_rows, animation_set_order,
    animation_set_start_delay, set_animation_child_key,
};
pub(crate) use runtime::{advance_animation_groups, stop_animation_groups_for_hidden_subtree};

use fields::{push_anim_field, with_animation_state_mut};
use runtime::{apply_group_flipbook_state, sync_action_bar_busy_for_group};

// ── Resolve helpers ───────────────────────────────────────────────────────────

pub(super) fn resolve_animation_group_id(
    sim: &crate::lua_api::SimState,
    frame_id: u64,
) -> Option<u64> {
    sim.anim_frame_to_group.get(&frame_id).copied().or_else(|| {
        sim.anim_frame_to_anim
            .get(&frame_id)
            .map(|(group_id, _)| *group_id)
    })
}

fn refresh_active_animation_group(sim: &mut crate::lua_api::SimState, group_id: u64) {
    let active = sim
        .animation_groups
        .get(&group_id)
        .is_some_and(|group| group.playing || group.paused || group.pending_finish);
    if active {
        sim.active_animation_groups.insert(group_id);
    } else {
        sim.active_animation_groups.remove(&group_id);
    }
}

pub(super) fn resolve_anim_target_id(
    sim: &crate::lua_api::SimState,
    owner_id: u64,
    child_key: Option<&str>,
) -> Option<u64> {
    let Some(key) = child_key else {
        return Some(owner_id);
    };
    sim.widgets.get(owner_id).and_then(|owner| {
        owner
            .children_keys
            .get(key)
            .copied()
            .or_else(|| find_child_by_key(sim, owner, key))
    })
}

fn find_child_by_key(
    sim: &crate::lua_api::SimState,
    owner: &crate::widget::Frame,
    key: &str,
) -> Option<u64> {
    owner.children.iter().copied().find(|child_id| {
        sim.widgets.get(*child_id).is_some_and(|child| {
            child.parent_key.as_deref() == Some(key) || child.name.as_deref() == Some(key)
        })
    })
}

// ── Animation group queries ───────────────────────────────────────────────────

pub(super) fn get_animation_groups(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let ag_frame_ids: Vec<u64> = {
        let sim = borrow_state(state)?;
        sim.anim_frame_to_group
            .iter()
            .filter(|&(_, &gid)| {
                sim.animation_groups
                    .get(&gid)
                    .is_some_and(|g| g.owner_frame_id == id)
            })
            .map(|(&fid, _)| fid)
            .collect()
    };
    let count = ag_frame_ids.len() as u32;
    for fid in ag_frame_ids {
        let val = frame_ref(state, fid)?;
        state.push(val);
    }
    Ok(count)
}

pub(super) fn get_animations(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let mut animation_frame_ids: Vec<(usize, u64)> = {
        let sim = borrow_state(state)?;
        let Some(group_id) = sim.anim_frame_to_group.get(&group_frame_id).copied() else {
            return Ok(0);
        };
        sim.anim_frame_to_anim
            .iter()
            .filter_map(|(&frame_id, &(mapped_group_id, animation_index))| {
                (mapped_group_id == group_id).then_some((animation_index, frame_id))
            })
            .collect()
    };
    animation_frame_ids.sort_unstable_by_key(|(animation_index, _)| *animation_index);
    let count = animation_frame_ids.len() as u32;
    for (_, frame_id) in animation_frame_ids {
        let animation_ref = frame_ref(state, frame_id)?;
        state.push(animation_ref);
    }
    Ok(count)
}

pub(super) fn get_animation_target(state: &mut LuaState) -> LuaResult<u32> {
    let animation_frame_id = frame_id_from_stack(state, 1)?;
    let target_id = {
        let sim = borrow_state(state)?;
        let Some((group_id, animation_index)) = sim.anim_frame_to_anim.get(&animation_frame_id)
        else {
            return Ok(0);
        };
        let Some(group) = sim.animation_groups.get(group_id) else {
            return Ok(0);
        };
        let child_key = group
            .animations
            .get(*animation_index)
            .and_then(|animation| animation.child_key.as_deref());
        resolve_anim_target_id(&sim, group.owner_frame_id, child_key)
    };
    let Some(target_id) = target_id else {
        return Ok(0);
    };
    let target_ref = frame_ref(state, target_id)?;
    state.push(target_ref);
    Ok(1)
}

pub(super) fn get_region_parent(state: &mut LuaState) -> LuaResult<u32> {
    let animation_frame_id = frame_id_from_stack(state, 1)?;
    let owner_id = {
        let sim = borrow_state(state)?;
        let Some((group_id, _)) = sim.anim_frame_to_anim.get(&animation_frame_id) else {
            return Ok(0);
        };
        sim.animation_groups
            .get(group_id)
            .map(|group| group.owner_frame_id)
    };
    let Some(owner_id) = owner_id else {
        return Ok(0);
    };
    let owner_ref = frame_ref(state, owner_id)?;
    state.push(owner_ref);
    Ok(1)
}

// ── Animation group control ───────────────────────────────────────────────────

pub(super) fn animation_group_play(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let reverse = !matches!(stack_val(state, 2), Val::Nil | Val::Bool(false));
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = resolve_animation_group_id(&sim, group_frame_id) {
        if let Some(group) = sim.animation_groups.get_mut(&group_id) {
            group.playing = true;
            group.paused = false;
            group.done = false;
            group.pending_finish = false;
            group.reverse = reverse;
        }
        refresh_active_animation_group(&mut sim, group_id);
        apply_group_flipbook_state(&mut sim, group_id);
        sync_action_bar_busy_for_group(&mut sim, group_id);
    }
    Ok(0)
}

pub(super) fn animation_group_pause(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = resolve_animation_group_id(&sim, group_frame_id) {
        if let Some(group) = sim.animation_groups.get_mut(&group_id)
            && group.playing
        {
            group.playing = false;
            group.paused = true;
        }
        refresh_active_animation_group(&mut sim, group_id);
        sync_action_bar_busy_for_group(&mut sim, group_id);
    }
    Ok(0)
}

pub(super) fn animation_group_stop(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = resolve_animation_group_id(&sim, group_frame_id) {
        if let Some(group) = sim.animation_groups.get_mut(&group_id) {
            group.playing = false;
            group.paused = false;
            group.done = true;
            group.pending_finish = false;
            group.elapsed = 0.0;
            for animation in &mut group.animations {
                animation.elapsed = 0.0;
            }
        }
        refresh_active_animation_group(&mut sim, group_id);
        apply_group_flipbook_state(&mut sim, group_id);
        sync_action_bar_busy_for_group(&mut sim, group_id);
    }
    Ok(0)
}

pub(super) fn animation_group_is_playing(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let playing = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| {
                sim.animation_groups
                    .get(&group_id)
                    .map(|group| group.playing)
            })
            .unwrap_or(false)
    };
    state.push(Val::Bool(playing));
    Ok(1)
}

pub(super) fn animation_group_is_paused(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let paused = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| {
                sim.animation_groups
                    .get(&group_id)
                    .map(|group| group.paused)
            })
            .unwrap_or(false)
    };
    state.push(Val::Bool(paused));
    Ok(1)
}

pub(super) fn animation_group_set_playing(state: &mut LuaState) -> LuaResult<u32> {
    let playing = match stack_val(state, 2) {
        Val::Bool(value) => value,
        Val::Nil => false,
        _ => true,
    };
    if playing {
        animation_group_play(state)
    } else {
        animation_group_stop(state)
    }
}

pub(super) fn animation_group_restart(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = resolve_animation_group_id(&sim, frame_id) {
        if let Some(group) = sim.animation_groups.get_mut(&group_id) {
            group.playing = true;
            group.paused = false;
            group.done = false;
            group.pending_finish = false;
            group.elapsed = 0.0;
            for animation in &mut group.animations {
                animation.elapsed = 0.0;
            }
        }
        refresh_active_animation_group(&mut sim, group_id);
        apply_group_flipbook_state(&mut sim, group_id);
    }
    Ok(0)
}

pub(super) fn animation_group_finish(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = resolve_animation_group_id(&sim, frame_id) {
        if let Some(group) = sim.animation_groups.get_mut(&group_id) {
            group.playing = true;
            group.paused = false;
            group.pending_finish = true;
        }
        refresh_active_animation_group(&mut sim, group_id);
    }
    Ok(0)
}

pub(super) fn animation_group_is_done(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let done = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id).map(|group| group.done))
            .unwrap_or(false)
    };
    state.push(Val::Bool(done));
    Ok(1)
}

pub(super) fn animation_group_get_duration(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let duration = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .map(animation_group_total_duration)
            .unwrap_or(0.0)
    };
    state.push(Val::Num(duration));
    Ok(1)
}

pub(super) fn animation_group_set_looping(state: &mut LuaState) -> LuaResult<u32> {
    use super::shared::opt_string;
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let looping = opt_string(state, 2).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = sim.anim_frame_to_group.get(&group_frame_id).copied()
        && let Some(group) = sim.animation_groups.get_mut(&group_id)
    {
        group.looping = crate::lua_api::animation::LoopType::from_str(&looping);
    }
    Ok(0)
}

pub(super) fn animation_group_get_looping(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let looping = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .map(group_loop_name)
            .unwrap_or("NONE")
    };
    let value = crate::lua_api::methods::create_string_static(state, looping);
    state.push(value);
    Ok(1)
}

pub(super) fn animation_group_get_loop_state(state: &mut LuaState) -> LuaResult<u32> {
    animation_group_get_looping(state)
}

pub(super) fn animation_group_set_animation_speed_multiplier(
    state: &mut LuaState,
) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let speed = match stack_val(state, 2) {
        Val::Num(value) if value.is_finite() => value,
        _ => 1.0,
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = sim.anim_frame_to_group.get(&group_frame_id).copied()
        && let Some(group) = sim.animation_groups.get_mut(&group_id)
    {
        group.speed_multiplier = speed.max(0.0);
    }
    Ok(0)
}

pub(super) fn animation_group_get_animation_speed_multiplier(
    state: &mut LuaState,
) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let speed = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .map(|group| group.speed_multiplier)
            .unwrap_or(1.0)
    };
    state.push(Val::Num(speed));
    Ok(1)
}

pub(super) fn animation_group_set_to_final_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let set_to_final_alpha = matches!(stack_val(state, 2), Val::Bool(true));
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = sim.anim_frame_to_group.get(&group_frame_id).copied()
        && let Some(group) = sim.animation_groups.get_mut(&group_id)
    {
        group.set_to_final_alpha = set_to_final_alpha;
    }
    Ok(0)
}

pub(super) fn animation_group_is_set_to_final_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .is_some_and(|group| group.set_to_final_alpha)
    };
    push_group_bool(state, value)
}

pub(super) fn animation_group_get_to_final_alpha(state: &mut LuaState) -> LuaResult<u32> {
    animation_group_is_set_to_final_alpha(state)
}

pub(super) fn animation_group_is_pending_finish(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .is_some_and(|group| group.pending_finish)
    };
    push_group_bool(state, value)
}

pub(super) fn animation_group_is_reverse(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .is_some_and(|group| group.reverse)
    };
    push_group_bool(state, value)
}

pub(super) fn animation_group_get_elapsed(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .map(|group| group.elapsed)
            .unwrap_or(0.0)
    };
    push_group_num(state, value)
}

pub(super) fn animation_group_get_progress(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        resolve_animation_group_id(&sim, group_frame_id)
            .and_then(|group_id| sim.animation_groups.get(&group_id))
            .map(group_elapsed_progress)
            .unwrap_or(0.0)
    };
    push_group_num(state, value)
}

pub(super) fn animation_group_get_smooth_progress(state: &mut LuaState) -> LuaResult<u32> {
    animation_group_get_progress(state)
}

pub(super) fn animation_group_remove_animations(state: &mut LuaState) -> LuaResult<u32> {
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    let Some(group_id) = sim.anim_frame_to_group.get(&group_frame_id).copied() else {
        return Ok(0);
    };
    let removed: Vec<u64> = sim
        .anim_frame_to_anim
        .iter()
        .filter_map(|(&frame_id, &(mapped_group_id, _))| {
            (mapped_group_id == group_id).then_some(frame_id)
        })
        .collect();
    sim.anim_frame_to_anim
        .retain(|_, (mapped_group_id, _)| *mapped_group_id != group_id);
    if let Some(group) = sim.animation_groups.get_mut(&group_id) {
        group.animations.clear();
        group.elapsed = 0.0;
        group.pending_finish = false;
        group.done = false;
    }
    for frame_id in removed {
        if let Some(frame) = sim.widgets.get_mut(frame_id) {
            frame.parent_id = Some(group_frame_id);
        }
    }
    Ok(0)
}

// ── Animation state accessors ─────────────────────────────────────────────────

pub(super) fn animation_set_smoothing(state: &mut LuaState) -> LuaResult<u32> {
    let smoothing = super::shared::opt_string(state, 2).unwrap_or_else(|| "NONE".to_string());
    with_animation_state_mut(state, |a| a.smoothing = smoothing.clone())?;
    Ok(0)
}

pub(super) fn animation_get_smoothing(state: &mut LuaState) -> LuaResult<u32> {
    let value = {
        let animation_frame_id = frame_id_from_stack(state, 1)?;
        let sim = borrow_state(state)?;
        sim.anim_frame_to_anim
            .get(&animation_frame_id)
            .and_then(|(group_id, animation_index)| {
                sim.animation_groups
                    .get(group_id)
                    .and_then(|group| group.animations.get(*animation_index))
            })
            .map(|animation| animation_smoothing_name(animation).to_string())
            .unwrap_or_else(|| "NONE".to_string())
    };
    let value = crate::lua_api::methods::create_string(state, &value);
    state.push(value);
    Ok(1)
}

pub(super) fn animation_set_from_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let value = match stack_val(state, 2) {
        Val::Num(n) => n,
        _ => 0.0,
    };
    with_animation_state_mut(state, |a| a.from_alpha = value)?;
    Ok(0)
}

pub(super) fn animation_get_from_alpha(state: &mut LuaState) -> LuaResult<u32> {
    push_anim_field(state, |a| a.from_alpha)
}

pub(super) fn animation_set_to_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let value = match stack_val(state, 2) {
        Val::Num(n) => n,
        _ => 1.0,
    };
    with_animation_state_mut(state, |a| a.to_alpha = value)?;
    Ok(0)
}

pub(super) fn animation_get_to_alpha(state: &mut LuaState) -> LuaResult<u32> {
    push_anim_field(state, |a| a.to_alpha)
}

pub(super) fn animation_set_change(state: &mut LuaState) -> LuaResult<u32> {
    let change = match stack_val(state, 2) {
        Val::Num(n) => n,
        _ => 0.0,
    };
    with_animation_state_mut(state, |a| a.to_alpha = a.from_alpha + change)?;
    Ok(0)
}

pub(super) fn animation_get_order(state: &mut LuaState) -> LuaResult<u32> {
    push_anim_field(state, |a| a.order as f64)
}

pub(super) fn animation_get_start_delay(state: &mut LuaState) -> LuaResult<u32> {
    push_anim_field(state, |a| a.start_delay)
}

pub(super) fn animation_get_end_delay(state: &mut LuaState) -> LuaResult<u32> {
    push_anim_field(state, |a| a.end_delay)
}

pub(super) fn animation_get_elapsed(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        if let Some(group_id) = sim.anim_frame_to_group.get(&frame_id).copied() {
            sim.animation_groups
                .get(&group_id)
                .map(|group| group.elapsed)
                .unwrap_or(0.0)
        } else {
            sim.anim_frame_to_anim
                .get(&frame_id)
                .and_then(|(group_id, animation_index)| {
                    sim.animation_groups
                        .get(group_id)
                        .and_then(|group| group.animations.get(*animation_index))
                        .map(|animation| animation.elapsed)
                })
                .unwrap_or(0.0)
        }
    };
    push_group_num(state, value)
}

pub(super) fn animation_get_progress(state: &mut LuaState) -> LuaResult<u32> {
    let animation_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        animation_progress_for_frame(&sim, animation_frame_id)
    };
    push_group_num(state, value)
}

pub(super) fn animation_get_smooth_progress(state: &mut LuaState) -> LuaResult<u32> {
    animation_get_progress(state)
}

pub(super) fn animation_is_stopped(state: &mut LuaState) -> LuaResult<u32> {
    let animation_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        sim.anim_frame_to_anim
            .get(&animation_frame_id)
            .and_then(|(group_id, _)| sim.animation_groups.get(group_id))
            .is_none_or(|group| !group.playing)
    };
    push_group_bool(state, value)
}

pub(super) fn animation_is_delaying(state: &mut LuaState) -> LuaResult<u32> {
    let animation_frame_id = frame_id_from_stack(state, 1)?;
    let value = {
        let sim = borrow_state(state)?;
        sim.anim_frame_to_anim
            .get(&animation_frame_id)
            .and_then(|(group_id, animation_index)| {
                sim.animation_groups.get(group_id).and_then(|group| {
                    group
                        .animations
                        .get(*animation_index)
                        .map(|animation| animation_is_in_start_delay(group, animation))
                })
            })
            .unwrap_or(false)
    };
    push_group_bool(state, value)
}

fn animation_is_in_start_delay(
    group: &crate::lua_api::animation::AnimGroupState,
    animation: &crate::lua_api::animation::AnimState,
) -> bool {
    let mut preceding_orders = std::collections::BTreeMap::<u32, f64>::new();
    for prior in &group.animations {
        if prior.order >= animation.order {
            continue;
        }
        let duration = prior.total_time().max(0.0);
        preceding_orders
            .entry(prior.order)
            .and_modify(|longest| *longest = longest.max(duration))
            .or_insert(duration);
    }
    let order_start: f64 = preceding_orders.into_values().sum();
    let within_order = group.elapsed - order_start;
    // Active elapsed has already had this delay removed by the tick processor.
    within_order >= 0.0 && within_order < animation.start_delay.max(0.0)
}

pub(super) fn animation_group_total_duration(
    group: &crate::lua_api::animation::AnimGroupState,
) -> f64 {
    let mut duration_by_order = std::collections::BTreeMap::<u32, f64>::new();
    for animation in &group.animations {
        let total_time = animation.total_time();
        duration_by_order
            .entry(animation.order)
            .and_modify(|current| *current = current.max(total_time))
            .or_insert(total_time);
    }
    duration_by_order.into_values().sum()
}

pub(super) fn animation_group_frame_id(group: &crate::lua_api::animation::AnimGroupState) -> u64 {
    group.frame_id.unwrap_or(group.owner_frame_id)
}

fn push_group_bool(state: &mut LuaState, value: bool) -> LuaResult<u32> {
    state.push(Val::Bool(value));
    Ok(1)
}

pub(super) fn push_group_num(state: &mut LuaState, value: f64) -> LuaResult<u32> {
    state.push(Val::Num(value));
    Ok(1)
}

pub(super) fn current_group_total_duration(
    group: &crate::lua_api::animation::AnimGroupState,
) -> f64 {
    animation_group_total_duration(group)
}

fn group_elapsed_progress(group: &crate::lua_api::animation::AnimGroupState) -> f64 {
    let duration = current_group_total_duration(group);
    if duration <= 0.0 {
        0.0
    } else {
        (group.elapsed / duration).clamp(0.0, 1.0)
    }
}

fn animation_progress_for_frame(sim: &crate::lua_api::SimState, animation_frame_id: u64) -> f64 {
    sim.anim_frame_to_group
        .get(&animation_frame_id)
        .copied()
        .and_then(|group_id| sim.animation_groups.get(&group_id))
        .map(group_elapsed_progress)
        .or_else(|| animation_child_progress_for_frame(sim, animation_frame_id))
        .unwrap_or(0.0)
}

fn animation_child_progress_for_frame(
    sim: &crate::lua_api::SimState,
    animation_frame_id: u64,
) -> Option<f64> {
    let (group_id, animation_index) = sim.anim_frame_to_anim.get(&animation_frame_id)?;
    let group = sim.animation_groups.get(group_id)?;
    let animation = group.animations.get(*animation_index)?;
    Some(animation_elapsed_progress(animation))
}

fn animation_elapsed_progress(animation: &crate::lua_api::animation::AnimState) -> f64 {
    let duration = animation.duration.max(0.0);
    if duration <= 0.0 {
        0.0
    } else {
        (animation.elapsed / duration).clamp(0.0, 1.0)
    }
}

fn animation_smoothing_name(animation: &crate::lua_api::animation::AnimState) -> &str {
    animation.smoothing.as_str()
}

fn group_loop_name(group: &crate::lua_api::animation::AnimGroupState) -> &'static str {
    match group.looping {
        crate::lua_api::animation::LoopType::None => "NONE",
        crate::lua_api::animation::LoopType::Repeat => "REPEAT",
        crate::lua_api::animation::LoopType::Bounce => "BOUNCE",
    }
}
