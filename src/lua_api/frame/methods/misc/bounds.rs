//! Frame bounds, resize, and position offset methods.

use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, IntoStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "GetBoundsRect", get_bounds_rect)?;
    table_set_rust_fn_static(state, mt, "GetClampRectInsets", get_clamp_rect_insets)?;
    table_set_rust_fn_static(state, mt, "GetResizeBounds", get_resize_bounds)?;
    table_set_rust_fn_static(state, mt, "SetClampRectInsets", set_clamp_rect_insets)?;
    table_set_rust_fn_static(state, mt, "SetMinResize", set_min_resize)?;
    table_set_rust_fn_static(state, mt, "SetMaxResize", set_max_resize)?;
    table_set_rust_fn_static(state, mt, "SetResizeBounds", set_resize_bounds)?;
    table_set_rust_fn_static(state, mt, "SetPointsOffset", set_points_offset)?;
    table_set_rust_fn_static(state, mt, "UpdateHeight", update_height)?;
    Ok(())
}

pub fn get_bounds_rect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if borrow_state(state)?.widgets.is_rect_dirty(id) {
        borrow_state_mut(state)?.resolve_rect_if_dirty(id);
    }
    let (left, bottom, width, height) = read_bounds_rect(state, id)?;
    (left, bottom, width, height).into_stack(state)
}

fn read_bounds_rect(state: &mut LuaState, id: u64) -> LuaResult<(f64, f64, f64, f64)> {
    let sim = borrow_state(state)?;
    let result = sim
        .widgets
        .get(id)
        .and_then(|frame| {
            frame
                .layout_rect
                .map(|rect| (rect, frame.effective_scale.max(1e-6)))
        })
        .map(|(rect, eff_scale)| {
            (
                (rect.x / eff_scale) as f64,
                ((sim.screen_height - rect.y - rect.height) / eff_scale) as f64,
                (rect.width / eff_scale) as f64,
                (rect.height / eff_scale) as f64,
            )
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    Ok(result)
}

pub fn get_clamp_rect_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (left, right, top, bottom) = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| f.clamp_rect_insets)
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    state.push(Val::Num(left as f64));
    state.push(Val::Num(right as f64));
    state.push(Val::Num(top as f64));
    state.push(Val::Num(bottom as f64));
    Ok(4)
}

pub fn set_clamp_rect_insets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let left = f64::from_stack(state, 2).unwrap_or(0.0) as f32;
    let right = f64::from_stack(state, 3).unwrap_or(0.0) as f32;
    let top = f64::from_stack(state, 4).unwrap_or(0.0) as f32;
    let bottom = f64::from_stack(state, 5).unwrap_or(0.0) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.clamp_rect_insets = (left, right, top, bottom);
    }
    Ok(0)
}

pub fn set_points_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = f64::from_stack(state, 2)?;
    let y = f64::from_stack(state, 3)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetPointsOffset");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        for anchor in &mut frame.anchors {
            anchor.x_offset = x as f32;
            anchor.y_offset = y as f32;
        }
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

pub fn get_resize_bounds(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (min_width, min_height, max_width, max_height) = sim
        .widgets
        .get(id)
        .map(|frame| {
            let (min_width, min_height) = frame.resize_bounds_min;
            let (max_width, max_height) = frame
                .resize_bounds_max
                .map(|(w, h)| (Val::Num(w as f64), Val::Num(h as f64)))
                .unwrap_or((Val::Nil, Val::Nil));
            (min_width, min_height, max_width, max_height)
        })
        .unwrap_or((0.0_f32, 0.0_f32, Val::Nil, Val::Nil));
    drop(sim);
    state.push(Val::Num(min_width as f64));
    state.push(Val::Num(min_height as f64));
    state.push(max_width);
    state.push(max_height);
    Ok(4)
}

pub fn set_min_resize(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let min_width = read_optional_f32(state, 2);
    let min_height = read_optional_f32(state, 3);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.resize_bounds_min = (min_width, min_height);
    }
    Ok(0)
}

pub fn set_max_resize(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max_width = read_optional_max_f32(state, 2);
    let max_height = read_optional_max_f32(state, 3);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.resize_bounds_max = max_width.zip(max_height);
    }
    Ok(0)
}

pub fn set_resize_bounds(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let min_width = read_optional_f32(state, 2);
    let min_height = read_optional_f32(state, 3);
    let max_width = read_optional_max_f32(state, 4);
    let max_height = read_optional_max_f32(state, 5);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.resize_bounds_min = (min_width, min_height);
        frame.resize_bounds_max = max_width.zip(max_height);
    }
    Ok(0)
}

fn read_optional_f32(state: &mut LuaState, index: i32) -> f32 {
    match stack_val(state, index) {
        Val::Num(value) => value.max(0.0) as f32,
        _ => 0.0,
    }
}

fn read_optional_max_f32(state: &mut LuaState, index: i32) -> Option<f32> {
    match stack_val(state, index) {
        Val::Num(value) => Some(value.max(0.0) as f32),
        _ => None,
    }
}

fn update_height(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
