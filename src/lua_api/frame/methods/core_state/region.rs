//! Region-query methods: IsRectValid, IsMouseMotionFocus, IsObjectLoaded,
//! IsMouseOver, StopAnimating, GetSourceLocation, Intersects,
//! IsDrawLayerEnabled, SetDrawLayerEnabled.

use super::helpers::{frame_id, opt_f32};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, extract_frame_id};
use crate::lua_bridge::FromStack;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn is_rect_valid(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    let has_anchors = sim
        .widgets
        .get(id)
        .map(|f| !f.anchors.is_empty())
        .unwrap_or(false);
    if has_anchors {
        sim.resolve_rect_if_dirty(id);
    }
    let result = has_anchors && !sim.widgets.is_rect_dirty(id);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn is_mouse_motion_focus(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    super::super::forbidden_aspects::ensure_forbidden_aspect_absent(
        state,
        id,
        "QueryFocus",
        "IsMouseMotionFocus",
    )?;
    let sim = borrow_state(state)?;
    let result = sim.hovered_frame == Some(id);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn is_object_loaded(state: &mut LuaState) -> LuaResult<u32> {
    let _id = frame_id(state, 1)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub fn is_mouse_over(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let left = opt_f32(state, 2);
    let right = opt_f32(state, 3);
    let top = opt_f32(state, 4);
    let bottom = opt_f32(state, 5);
    {
        let needs_resolve = borrow_state(state)?.widgets.is_rect_dirty(id);
        if needs_resolve {
            borrow_state_mut(state)?.resolve_rect_if_dirty(id);
        }
    }
    let sim = borrow_state(state)?;
    let result = is_mouse_over_bounds(&sim, id, left, right, top, bottom);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

fn is_mouse_over_bounds(
    state: &crate::lua_api::SimState,
    id: u64,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> bool {
    let Some((mouse_x, mouse_y)) = state.mouse_position else {
        return false;
    };
    let Some(frame) = state.widgets.get(id) else {
        return false;
    };
    if !frame.visible || !frame.mouse_enabled {
        return false;
    }
    let Some(rect) = frame.layout_rect else {
        return false;
    };
    mouse_x >= rect.x - left
        && mouse_x <= rect.x + rect.width + right
        && mouse_y >= rect.y - top
        && mouse_y <= rect.y + rect.height + bottom
}

pub fn stop_animating(state: &mut LuaState) -> LuaResult<u32> {
    let _id = frame_id(state, 1)?;
    Ok(0)
}

pub fn get_source_location(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        drop(sim);
        state.push(Val::Nil);
        return Ok(1);
    };
    let owner_addon = frame.owner_addon;
    let location = source_location_for_owner(&sim, owner_addon);
    drop(sim);
    match location {
        Some(loc) => {
            let val = create_string(state, &loc);
            state.push(val);
        }
        None => {
            state.push(Val::Nil);
        }
    }
    Ok(1)
}

fn source_location_for_owner(
    state: &crate::lua_api::state::SimState,
    owner_addon: Option<u16>,
) -> Option<String> {
    let addon = owner_addon.and_then(|idx| state.addons.get(idx as usize))?;
    let folder = addon.folder_name.as_str();
    if folder == "__BuiltIn" {
        return Some("Interface/FrameXML".to_string());
    }
    Some(format!("Interface/AddOns/{folder}"))
}

pub fn intersects(state: &mut LuaState) -> LuaResult<u32> {
    let this_id = frame_id(state, 1)?;
    let other_val = stack_val(state, 2);
    let Some(other_id) = extract_frame_id(state, other_val) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        sim.resolve_rect_if_dirty(this_id);
        sim.resolve_rect_if_dirty(other_id);
    }
    let sim = borrow_state(state)?;
    let Some(this_rect) = sim.widgets.get(this_id).and_then(|f| f.layout_rect) else {
        drop(sim);
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let Some(other_rect) = sim.widgets.get(other_id).and_then(|f| f.layout_rect) else {
        drop(sim);
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let result = layout_rects_intersect(this_rect, other_rect);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

fn layout_rects_intersect(a: crate::LayoutRect, b: crate::LayoutRect) -> bool {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    right > left && bottom > top
}

pub fn is_draw_layer_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let layer_name = String::from_stack(state, 2)?;
    let Some(layer) = crate::widget::DrawLayer::from_str(&layer_name) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let sim = borrow_state(state)?;
    let result = sim
        .widgets
        .get(id)
        .map(|frame| frame.is_draw_layer_enabled(layer))
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn set_draw_layer_enabled(state: &mut LuaState) -> LuaResult<u32> {
    use super::helpers::arg_bool;
    let id = frame_id(state, 1)?;
    let layer_name = String::from_stack(state, 2)?;
    let enabled = arg_bool(state, 3);
    let Some(layer) = crate::widget::DrawLayer::from_str(&layer_name) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.set_draw_layer_enabled(layer, enabled);
    }
    Ok(0)
}
