//! Wrath template-backed tooltip image attachment.

use super::line_frames::sync_tooltip_line_frame;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_api::state::SimState;
use crate::lua_bridge::stack_val;
use crate::widget::{Anchor, AnchorPoint};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const MAX_TEXTURES: usize = 10;
const ROW_GAP: f32 = 2.0;

pub(super) fn attach_texture(state: &mut LuaState, tooltip_id: u64) -> LuaResult<u32> {
    let filename = texture_filename(state, stack_val(state, 2));
    let Some((row, texture_id, adjacent)) = find_texture_slot(state, tooltip_id)? else {
        return Ok(0);
    };
    let Some(left_id) = sync_tooltip_line_frame(state, tooltip_id, false, row + 1)? else {
        return Ok(0);
    };
    let uv = read_texture_coords(state)?;
    let mut sim = borrow_state_mut(state)?;
    assign_slot_texture(&mut sim, texture_id, filename, uv);
    anchor_attachment(&mut sim, tooltip_id, left_id, (row, texture_id, adjacent));
    sim.tooltips
        .entry(tooltip_id)
        .or_default()
        .wrath_texture_rows
        .push((row, texture_id));
    Ok(0)
}

/// 12340 reads the argument with `lua_tolstring`: a string, a numeric string and
/// a number (formatted as a decimal filename) all name a file, and a number is
/// never resolved as a FileDataID. Any other value leaves the filename null
/// without a type error (tooltip-addtexture-12340).
fn texture_filename(state: &LuaState, value: Val) -> Option<String> {
    match value {
        Val::Str(_) => val_to_string(state, value),
        Val::Num(_) => Some(value.to_string()),
        _ => None,
    }
}

fn find_texture_slot(state: &LuaState, tooltip_id: u64) -> LuaResult<Option<(usize, u64, bool)>> {
    let sim = borrow_state(state)?;
    let Some(td) = sim.tooltips.get(&tooltip_id) else {
        return Ok(None);
    };
    if td.lines.len() < 2 || td.wrath_texture_rows.len() >= MAX_TEXTURES {
        return Ok(None);
    }
    let Some(name) = sim.widgets.get(tooltip_id).and_then(|f| f.name.as_ref()) else {
        return Ok(None);
    };
    let row = td.lines.len() - 1;
    let slot_name = format!("{name}Texture{}", td.wrath_texture_rows.len() + 1);
    let adjacent = td
        .wrath_texture_rows
        .last()
        .is_some_and(|&(previous, _)| previous + 1 == row);
    Ok(sim
        .widgets
        .get_id_by_name(&slot_name)
        .map(|id| (row, id, adjacent)))
}

fn read_texture_coords(state: &mut LuaState) -> LuaResult<(f32, f32, f32, f32)> {
    let mut values = [0.0, 1.0, 0.0, 1.0];
    for (index, component) in values.iter_mut().enumerate() {
        let value = stack_val(state, index as i32 + 3);
        if !matches!(value, Val::Nil) {
            *component = crate::lua_api::wrath_arguments::convert_number(state, value)?
                .unwrap_or(0.0) as f32;
        }
    }
    Ok((values[0], values[1], values[2], values[3]))
}

fn assign_slot_texture(
    sim: &mut SimState,
    id: u64,
    filename: Option<String>,
    uv: (f32, f32, f32, f32),
) {
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.texture = filename.filter(|name| !name.is_empty());
        frame.texture_file_data_id = None;
        frame.atlas = None;
        frame.color_texture = None;
        frame.atlas_tex_coords = None;
        frame.tex_coords = Some(uv);
        frame.tex_coords_quad = None;
        frame.visible = true;
    }
}

fn anchor_attachment(sim: &mut SimState, tooltip: u64, left: u64, slot: (usize, u64, bool)) {
    let (row, texture, adjacent) = slot;
    let width = sim.widgets.get(texture).map_or(0.0, |frame| frame.width);
    let gap = crate::lua_api::tooltip::wrath_texture_gap(sim.screen_width, sim.screen_height);
    let previous = sim
        .tooltips
        .get(&tooltip)
        .and_then(|td| td.left_line_ids.get(row - 1))
        .copied();
    if let Some(previous) = previous {
        let offset = if adjacent { 0.0 } else { width + gap };
        anchor_below(sim, left, previous, offset);
    }
    replace_anchor(
        sim,
        texture,
        left,
        (AnchorPoint::TopRight, AnchorPoint::TopLeft),
        (-gap, 0.0),
    );
    let next = sim
        .widgets
        .get(tooltip)
        .and_then(|frame| frame.name.as_ref())
        .and_then(|name| {
            sim.widgets
                .get_id_by_name(&format!("{name}TextLeft{}", row + 2))
        });
    if let Some(next) = next {
        anchor_below(sim, next, left, -(width + gap));
    }
}

fn anchor_below(sim: &mut SimState, id: u64, previous: u64, x: f32) {
    replace_anchor(
        sim,
        id,
        previous,
        (AnchorPoint::TopLeft, AnchorPoint::BottomLeft),
        (x, -ROW_GAP),
    );
}

fn replace_anchor(
    sim: &mut SimState,
    id: u64,
    relative: u64,
    points: (AnchorPoint, AnchorPoint),
    offset: (f32, f32),
) {
    sim.widgets.remove_all_anchor_dependents_for(id);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.anchors = vec![Anchor {
            point: points.0,
            relative_to: None,
            relative_to_id: Some(relative as usize),
            relative_point: points.1,
            x_offset: offset.0,
            y_offset: offset.1,
        }];
    }
    sim.widgets.add_anchor_dependent(relative, id);
    sim.widgets.mark_rect_dirty(id);
}
