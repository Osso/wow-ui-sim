use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, frame_ref};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

use super::super::font_strings::resolve_child_name;
use super::super::shared::bind_named_child_global;

pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn animation_config_noop(
    _state: &mut LuaState,
) -> LuaResult<u32> {
    Ok(0)
}

pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn set_scale_dispatch(
    state: &mut LuaState,
) -> LuaResult<u32> {
    use crate::lua_api::methods::extract_frame_id;
    if extract_frame_id(state, stack_val(state, 1)).is_some() {
        return crate::lua_api::frame::methods::core_state::set_scale(state);
    }
    animation_config_noop(state)
}

// ── Creation ──────────────────────────────────────────────────────────────────

/// CreateAnimationGroup([name [, inherits]]) -> animationGroup
pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn create_animation_group(
    state: &mut LuaState,
) -> LuaResult<u32> {
    use crate::lua_api::animation::AnimGroupState;
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let _inherits: Option<String> = Option::<String>::from_stack(state, 3)?;
    let name = resolve_child_name(state, name_raw, parent_id);
    let mut child = Frame::new(WidgetType::Frame, name.clone(), Some(parent_id));
    child.object_type_name = Some("AnimationGroup".to_string());
    let child_id = child.id;
    {
        let mut sim = borrow_state_mut(state)?;
        let gid = sim.next_anim_group_id;
        sim.next_anim_group_id += 1;
        let mut group = AnimGroupState::new(parent_id);
        group.name = name.clone();
        group.frame_id = Some(child_id);
        sim.animation_groups.insert(gid, group);
        sim.anim_frame_to_group.insert(child_id, gid);
        sim.widgets.register(child);
        sim.widgets.add_child(parent_id, child_id);
        sim.invalidate_strata_buckets();
    }
    if let Some(ref n) = name {
        bind_named_child_global(state, n, child_id)?;
    }
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}

/// CreateAnimation([type [, name [, templateName]]]) -> animation
pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn create_animation(
    state: &mut LuaState,
) -> LuaResult<u32> {
    use crate::lua_api::animation::AnimationType;
    let group_frame_id = frame_id_from_stack(state, 1)?;
    let anim_type_str = super::super::shared::opt_string(state, 2);
    let anim_name_raw: Option<String> = Option::<String>::from_stack(state, 3)?;
    let template_name = Option::<String>::from_stack(state, 4)?;
    let group_id = lookup_anim_group_id(state, group_frame_id)?;
    let template = super::templates::prepare_template(state, template_name.as_deref())?;
    let anim_type = AnimationType::from_str(anim_type_str.as_deref().unwrap_or("Animation"));
    let name = resolve_child_name(state, anim_name_raw, group_frame_id);
    let (child_id, anim) = build_animation_child(state, group_frame_id, name.clone(), anim_type)?;
    register_animation(
        state,
        group_frame_id,
        group_id,
        child_id,
        name.clone(),
        anim,
    )?;
    if let Some(name) = &name {
        bind_named_child_global(state, name, child_id)?;
    }
    let val = frame_ref(state, child_id)?;
    super::templates::apply_template(state, template, val)?;
    state.push(val);
    Ok(1)
}

fn lookup_anim_group_id(state: &mut LuaState, group_frame_id: u64) -> LuaResult<u64> {
    let sim = borrow_state(state)?;
    sim.anim_frame_to_group
        .get(&group_frame_id)
        .copied()
        .ok_or_else(|| runtime_error("CreateAnimation called on non-AnimationGroup"))
}

fn build_animation_child(
    state: &mut LuaState,
    group_frame_id: u64,
    name: Option<String>,
    anim_type: crate::lua_api::animation::AnimationType,
) -> LuaResult<(u64, crate::lua_api::animation::AnimState)> {
    use crate::lua_api::animation::AnimState;
    use crate::widget::{Frame, WidgetType};
    let mut child = Frame::new(WidgetType::Frame, name.clone(), Some(group_frame_id));
    child.object_type_name = Some(anim_type.as_str().to_string());
    let child_id = child.id;
    let mut anim = AnimState::new(anim_type);
    anim.name = name;

    let mut sim = borrow_state_mut(state)?;
    sim.widgets.register(child);
    Ok((child_id, anim))
}

fn register_animation(
    state: &mut LuaState,
    group_frame_id: u64,
    group_id: u64,
    child_id: u64,
    _name: Option<String>,
    anim: crate::lua_api::animation::AnimState,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let group = sim
        .animation_groups
        .get_mut(&group_id)
        .ok_or_else(|| runtime_error("Animation group not found"))?;
    let idx = group.animations.len();
    group.animations.push(anim);
    sim.anim_frame_to_anim.insert(child_id, (group_id, idx));
    sim.widgets.add_child(group_frame_id, child_id);
    sim.invalidate_strata_buckets();
    Ok(())
}

/// CreateControlPoint([name]) -> controlPoint
pub(in crate::lua_api::frame::methods::button_anchor_hierarchy) fn create_control_point(
    state: &mut LuaState,
) -> LuaResult<u32> {
    use crate::widget::{Frame, WidgetType};
    let parent_id = frame_id_from_stack(state, 1)?;
    let name_raw: Option<String> = Option::<String>::from_stack(state, 2)?;
    let name = resolve_child_name(state, name_raw, parent_id);
    let mut child = Frame::new(WidgetType::Frame, name.clone(), Some(parent_id));
    child.object_type_name = Some("ControlPoint".to_string());
    let child_id = child.id;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.register(child);
        sim.widgets.add_child(parent_id, child_id);
        sim.invalidate_strata_buckets();
    }
    let val = frame_ref(state, child_id)?;
    state.push(val);
    Ok(1)
}
