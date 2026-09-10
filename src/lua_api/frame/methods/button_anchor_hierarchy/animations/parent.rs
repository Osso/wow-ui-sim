//! Animation ownership transfer; existing group timelines are not restarted.
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state_mut, extract_frame_id};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn reparent_animation(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<bool> {
    let parent_id = extract_frame_id(state, stack_val(state, 2));
    let order_value = stack_val(state, 3);
    let mut sim = borrow_state_mut(state)?;
    let Some((old_group_id, old_index)) = sim.anim_frame_to_anim.get(&frame_id).copied() else {
        return Ok(false);
    };
    let parent_id =
        parent_id.ok_or_else(|| runtime_error("animation parent must be an AnimationGroup"))?;
    let new_group_id = validate_destination(&sim, parent_id)?;
    let order = read_order(order_value)?;
    validate_source(&sim, old_group_id, old_index)?;
    if old_group_id == new_group_id {
        if let Some(order) = order {
            sim.animation_groups
                .get_mut(&old_group_id)
                .unwrap()
                .animations[old_index]
                .order = order;
        }
    } else {
        transfer_animation(
            &mut sim,
            frame_id,
            old_group_id,
            old_index,
            new_group_id,
            order,
        );
    }
    super::super::hierarchy::apply_parent_change(&mut sim, frame_id, Some(parent_id));
    Ok(true)
}

fn validate_destination(sim: &SimState, parent_id: u64) -> LuaResult<u64> {
    let group_id = sim
        .anim_frame_to_group
        .get(&parent_id)
        .copied()
        .ok_or_else(|| runtime_error("animation parent must be an AnimationGroup"))?;
    if !sim.animation_groups.contains_key(&group_id) {
        return Err(runtime_error("animation destination group is missing"));
    }
    Ok(group_id)
}

fn validate_source(sim: &SimState, group_id: u64, index: usize) -> LuaResult<()> {
    let animation = sim
        .animation_groups
        .get(&group_id)
        .and_then(|group| group.animations.get(index));
    if animation.is_none() {
        return Err(runtime_error("animation source ownership is inconsistent"));
    }
    Ok(())
}

fn read_order(value: Val) -> LuaResult<Option<u32>> {
    match value {
        Val::Nil => Ok(None),
        Val::Num(order) if order.is_finite() && (0.0..=u32::MAX as f64).contains(&order) => {
            Ok(Some(order as u32))
        }
        _ => Err(runtime_error(
            "animation order must be a finite number in the u32 range",
        )),
    }
}

fn transfer_animation(
    sim: &mut SimState,
    frame_id: u64,
    old_group_id: u64,
    old_index: usize,
    new_group_id: u64,
    order: Option<u32>,
) {
    // Both groups and the source index were validated before any mutation.
    let mut animation = sim
        .animation_groups
        .get_mut(&old_group_id)
        .unwrap()
        .animations
        .remove(old_index);
    animation.elapsed = 0.0;
    if let Some(order) = order {
        animation.order = order;
    }
    for (group_id, index) in sim.anim_frame_to_anim.values_mut() {
        if *group_id == old_group_id && *index > old_index {
            *index -= 1;
        }
    }
    let group = sim.animation_groups.get_mut(&new_group_id).unwrap();
    let new_index = group.animations.len();
    group.animations.push(animation);
    sim.anim_frame_to_anim
        .insert(frame_id, (new_group_id, new_index));
}
