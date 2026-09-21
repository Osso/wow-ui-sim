//! Color, vertex color, desaturation, and color-table helpers.

use super::super::shared::{rgba_from_stack, val_to_bool, val_to_f64};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, table_get};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn set_color_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(color) = rgba_from_stack(state, 2) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.color_texture = Some(color);
        clear_texture_sources(frame);
    }
    Ok(0)
}

fn clear_texture_sources(frame: &mut crate::widget::Frame) {
    frame.secret_texture = false;
    frame.texture = None;
    frame.texture_file_data_id = None;
    frame.atlas = None;
    frame.atlas_tex_coords = None;
}

pub(super) fn set_vertex_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(color) = rgba_from_stack(state, 2) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get(id)
        .map(|frame| frame.vertex_color != Some(color))
        .unwrap_or(false);
    if changed && let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.vertex_color = Some(color);
    }
    Ok(0)
}

#[cfg(feature = "retail-12-0-0")]
pub(super) fn set_vertex_color_from_boolean(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Val::Bool(condition) = stack_val(state, 2) else {
        return Err(rilua::runtime_error("boolean expected"));
    };
    // Simulator validation policy; native coercion and error order are unverified.
    let when_true = require_rgba_color(state, 3)?;
    let when_false = require_rgba_color(state, 4)?;
    let color = if condition { when_true } else { when_false };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.vertex_color = Some(color);
    }
    Ok(0)
}

#[cfg(feature = "retail-12-0-0")]
fn require_rgba_color(state: &mut LuaState, index: i32) -> LuaResult<crate::widget::Color> {
    let value = stack_val(state, index);
    if !matches!(value, Val::Table(_)) {
        return Err(rilua::runtime_error("color table expected"));
    }
    let mut channels = [0.0; 4];
    for (channel, name) in channels.iter_mut().zip(["r", "g", "b", "a"]) {
        let Val::Num(number) = table_get(state, value, name) else {
            return Err(rilua::runtime_error(
                "color must contain numeric RGBA channels",
            ));
        };
        *channel = number as f32;
    }
    Ok(crate::widget::Color::new(
        channels[0],
        channels[1],
        channels[2],
        channels[3],
    ))
}

pub(super) fn get_vertex_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (r, g, b, a) = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.vertex_color)
        .map(|c| (c.r as f64, c.g as f64, c.b as f64, c.a as f64))
        .unwrap_or((1.0, 1.0, 1.0, 1.0));
    (r, g, b, a).into_stack(state)
}

pub(super) fn set_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desaturated = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.desaturated = desaturated;
    }
    Ok(0)
}

pub(super) fn is_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desaturated = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.desaturated)
        .unwrap_or(false);
    state.push(Val::Bool(desaturated));
    Ok(1)
}

pub(super) fn set_desaturation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desaturated = val_to_f64(stack_val(state, 2)) > 0.0;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.desaturated = desaturated;
    }
    Ok(0)
}

pub(super) fn get_desaturation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desaturation = if borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.desaturated)
        .unwrap_or(false)
    {
        1.0
    } else {
        0.0
    };
    state.push(Val::Num(desaturation));
    Ok(1)
}

// ---------------------------------------------------------------------------
// SetCenterColor — no-op (matches master)
// ---------------------------------------------------------------------------

pub(super) fn set_center_color(state: &mut LuaState) -> LuaResult<u32> {
    let _ = frame_id_from_stack(state, 1);
    Ok(0)
}

// ---------------------------------------------------------------------------
// Color-table helpers (used by color.rs and rotation_mask.rs)
// ---------------------------------------------------------------------------

pub(super) fn color_from_table(state: &mut LuaState, val: Val) -> crate::widget::Color {
    let r = f32_from_table_field(state, val, "r");
    let g = f32_from_table_field(state, val, "g");
    let b = f32_from_table_field(state, val, "b");
    let a = f32_from_table_field_or(state, val, "a", 1.0);
    crate::widget::Color::new(r, g, b, a)
}

pub(super) fn f32_from_table_field(state: &mut LuaState, table: Val, key: &str) -> f32 {
    match table_get(state, table, key) {
        Val::Num(n) => n as f32,
        _ => 0.0,
    }
}

pub(super) fn f32_from_table_field_or(
    state: &mut LuaState,
    table: Val,
    key: &str,
    default: f32,
) -> f32 {
    match table_get(state, table, key) {
        Val::Num(n) => n as f32,
        _ => default,
    }
}
