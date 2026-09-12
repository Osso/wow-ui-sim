//! Blend mode, tex coords, thickness, tiling, snapping, slice, and vertex offset methods.

use super::super::shared::{opt_bool, opt_f32, opt_string, val_to_f64};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, frame_id_from_stack};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn set_blend_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(mode_name) = opt_string(state, 2) else {
        return Ok(0);
    };
    let normalized = mode_name.to_ascii_uppercase();
    let blend_mode = match normalized.as_str() {
        "ADD" => crate::BlendMode::Additive,
        _ => crate::BlendMode::Alpha,
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.alpha_mode = Some(normalized);
        frame.blend_mode = blend_mode;
    }
    Ok(0)
}

pub(super) fn get_blend_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.alpha_mode.clone())
        .unwrap_or_else(|| "BLEND".to_string());
    let mode_val = create_string(state, &mode);
    state.push(mode_val);
    Ok(1)
}

pub(super) fn set_tex_coord(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let coords: Vec<f32> = (2..=9).filter_map(|index| opt_f32(state, index)).collect();
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return Ok(0);
    };
    match coords.len() {
        4 => apply_rect_tex_coords(frame, [coords[0], coords[1], coords[2], coords[3]]),
        8 => apply_quad_tex_coords(
            frame,
            [
                coords[0], coords[1], coords[2], coords[3], coords[4], coords[5], coords[6],
                coords[7],
            ],
        ),
        _ => {}
    }
    Ok(0)
}

/// Rect form `(ULx, LRx, ULy, LRy)`. Atlas remapping is applied when the
/// frame has an active atlas slot, otherwise the values pass through.
pub(super) fn apply_rect_tex_coords(frame: &mut crate::widget::Frame, rect: [f32; 4]) {
    frame.tex_coords = Some(remap_tex_coords(
        frame.atlas_tex_coords,
        rect[0],
        rect[1],
        rect[2],
        rect[3],
    ));
    frame.tex_coords_quad = None;
}

/// Quad form (eight floats — TLx, TLy, BLx, BLy, TRx, TRy, BRx, BRy).
/// Records the raw quad for downstream rendering and computes a
/// rect bounding box for atlas remapping.
fn apply_quad_tex_coords(frame: &mut crate::widget::Frame, quad: [f32; 8]) {
    let left = quad[0].min(quad[2]).min(quad[4]).min(quad[6]);
    let right = quad[0].max(quad[2]).max(quad[4]).max(quad[6]);
    let top = quad[1].min(quad[3]).min(quad[5]).min(quad[7]);
    let bottom = quad[1].max(quad[3]).max(quad[5]).max(quad[7]);
    frame.tex_coords = Some(remap_tex_coords(
        frame.atlas_tex_coords,
        left,
        right,
        top,
        bottom,
    ));
    frame.tex_coords_quad = Some(quad);
}

/// Remap UV coordinates into an atlas sub-region when one is active.
///
/// When an atlas is set, the caller's [0,1] UV space maps onto the atlas slot.
/// Without an atlas the coords pass through unchanged.
pub(super) fn remap_tex_coords(
    atlas_tex_coords: Option<(f32, f32, f32, f32)>,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> (f32, f32, f32, f32) {
    if let Some((al, ar, at, ab)) = atlas_tex_coords {
        let aw = ar - al;
        let ah = ab - at;
        (
            al + left * aw,
            al + right * aw,
            at + top * ah,
            at + bottom * ah,
        )
    } else {
        (left, right, top, bottom)
    }
}

pub(super) fn get_tex_coord(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    if let Some(frame) = sim.widgets.get(id) {
        if let Some(quad) = frame.tex_coords_quad {
            drop(sim);
            for value in quad {
                state.push(Val::Num(value as f64));
            }
            return Ok(8);
        }
        if let Some((left, right, top, bottom)) = frame.tex_coords {
            drop(sim);
            return push_rect_tex_coords_as_corners(state, left, right, top, bottom);
        }
    }
    drop(sim);
    push_rect_tex_coords_as_corners(state, 0.0, 1.0, 0.0, 1.0)
}

fn push_rect_tex_coords_as_corners(
    state: &mut LuaState,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> LuaResult<u32> {
    for value in [left, top, left, bottom, right, top, right, bottom] {
        state.push(Val::Num(value as f64));
    }
    Ok(8)
}

pub(super) fn reset_tex_coord(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.tex_coords = frame.atlas_tex_coords;
        frame.tex_coords_quad = None;
    }
    Ok(0)
}

pub(super) fn set_thickness(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let thickness = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.line_thickness = thickness;
    }
    Ok(0)
}

pub(super) fn get_thickness(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let thickness = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.line_thickness as f64)
        .unwrap_or(1.0);
    state.push(Val::Num(thickness));
    Ok(1)
}

pub(super) fn set_horiz_tile(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.horiz_tile = enabled;
    }
    Ok(0)
}

pub(super) fn get_horiz_tile(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.horiz_tile)
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(super) fn set_vert_tile(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.vert_tile = enabled;
    }
    Ok(0)
}

pub(super) fn get_vert_tile(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.vert_tile)
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(super) fn set_texel_snapping_bias(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let bias = opt_f32(state, 2).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.texel_snapping_bias = bias;
    }
    Ok(0)
}

pub(super) fn get_texel_snapping_bias(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let bias = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.texel_snapping_bias)
        .unwrap_or_default();
    state.push(Val::Num(bias as f64));
    Ok(1)
}

pub(super) fn get_texture_slice_margins(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (left, right, top, bottom) = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.texture_slice_margins)
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    (left as f64, right as f64, top as f64, bottom as f64).into_stack(state)
}

pub(super) fn get_texture_slice_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.texture_slice_mode)
        .unwrap_or_default();
    state.push(Val::Num(mode as f64));
    Ok(1)
}

pub(super) fn set_snap_to_pixel_grid(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.snap_to_pixel_grid = enabled;
    }
    Ok(0)
}

pub(super) fn is_snapping_to_pixel_grid(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let snapping = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.snap_to_pixel_grid)
        .unwrap_or(false);
    state.push(Val::Bool(snapping));
    Ok(1)
}

// ---------------------------------------------------------------------------
// Nine-slice setters
// ---------------------------------------------------------------------------

pub(super) fn set_texture_slice_margins(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let left = opt_f32(state, 2).unwrap_or(0.0);
    let right = opt_f32(state, 3).unwrap_or(0.0);
    let top = opt_f32(state, 4).unwrap_or(0.0);
    let bottom = opt_f32(state, 5).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.texture_slice_margins = (left, right, top, bottom);
    }
    Ok(0)
}

pub(super) fn set_texture_slice_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = match stack_val(state, 2) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.texture_slice_mode = mode;
    }
    Ok(0)
}

pub(super) fn clear_texture_slice(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.texture_slice_margins = (0.0, 0.0, 0.0, 0.0);
        frame.texture_slice_mode = 0;
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// Vertex offsets
// ---------------------------------------------------------------------------

pub(super) fn set_vertex_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let index = match stack_val(state, 2) {
        Val::Num(n) => n as usize,
        _ => return Ok(0),
    };
    if index == 0 || index > 4 {
        return Ok(0);
    }
    let x = opt_f32(state, 3).unwrap_or(0.0);
    let y = opt_f32(state, 4).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let offsets = frame.vertex_offsets.get_or_insert([(0.0, 0.0); 4]);
        offsets[index - 1] = (x, y);
    }
    Ok(0)
}

pub(super) fn get_vertex_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let index = match stack_val(state, 2) {
        Val::Num(n) => n as usize,
        _ => {
            state.push(Val::Num(0.0));
            state.push(Val::Num(0.0));
            return Ok(2);
        }
    };
    if index == 0 || index > 4 {
        state.push(Val::Num(0.0));
        state.push(Val::Num(0.0));
        return Ok(2);
    }
    let (x, y) = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.vertex_offsets)
        .map(|offsets| offsets[index - 1])
        .unwrap_or((0.0, 0.0));
    state.push(Val::Num(x as f64));
    state.push(Val::Num(y as f64));
    Ok(2)
}

pub(super) fn clear_vertex_offsets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.vertex_offsets = None;
    }
    Ok(0)
}
