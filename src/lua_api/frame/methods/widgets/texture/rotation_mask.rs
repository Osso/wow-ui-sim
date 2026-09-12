//! Rotation, mask, gradient, visuals, and sprite sheet methods.

use super::super::shared::{opt_f32, opt_string};
use super::color::color_from_table;
use crate::lua_api::methods::val_to_string;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_table, frame_id_from_stack,
    get_or_create_frame_fields, table_get, table_set,
};
use crate::lua_bridge::stack_val;
use crate::widget::{Frame, WidgetType};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn set_rotation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let radians = opt_f32(state, 2).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.rotation = radians;
    }
    Ok(0)
}

pub(super) fn get_rotation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let radians = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.rotation as f64)
        .unwrap_or(0.0);
    state.push(Val::Num(radians));
    Ok(1)
}

// ---------------------------------------------------------------------------
// SetMask
// ---------------------------------------------------------------------------

pub(super) fn set_mask(state: &mut LuaState) -> LuaResult<u32> {
    let texture_id = frame_id_from_stack(state, 1)?;
    let mask_path = val_to_string(state, stack_val(state, 2));
    let mask_id = create_set_mask_texture(state, texture_id, mask_path)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(texture) = sim.widgets.get_mut_visual(texture_id) {
        texture.mask_textures.clear();
        texture.mask_textures.extend(mask_id);
    }
    Ok(0)
}

fn create_set_mask_texture(
    state: &mut LuaState,
    texture_id: u64,
    mask_path: Option<String>,
) -> LuaResult<Option<u64>> {
    let Some(mask_path) = mask_path else {
        return Ok(None);
    };
    let parent_id = borrow_state(state)?
        .widgets
        .get(texture_id)
        .and_then(|texture| texture.parent_id);
    let Some(parent_id) = parent_id else {
        return Ok(None);
    };

    let mut mask = Frame::new(WidgetType::Texture, None, Some(parent_id));
    mask.is_mask = true;
    mask.object_type_name = Some("MaskTexture".to_string());
    mask.texture = Some(mask_path);
    copy_texture_layout_to_mask(state, texture_id, &mut mask)?;
    let mask_id = mask.id;

    let mut sim = borrow_state_mut(state)?;
    sim.widgets.register(mask);
    sim.widgets.add_child(parent_id, mask_id);
    sim.invalidate_strata_buckets();
    Ok(Some(mask_id))
}

fn copy_texture_layout_to_mask(
    state: &LuaState,
    texture_id: u64,
    mask: &mut Frame,
) -> LuaResult<()> {
    let sim = borrow_state(state)?;
    let Some(texture) = sim.widgets.get(texture_id) else {
        return Ok(());
    };
    mask.width = texture.width;
    mask.height = texture.height;
    mask.anchors = texture.anchors.clone();
    mask.layout_rect = texture.layout_rect;
    Ok(())
}

// ---------------------------------------------------------------------------
// SetGradient
// ---------------------------------------------------------------------------

pub(super) fn set_gradient(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let orientation = opt_string(state, 2).unwrap_or_else(|| "VERTICAL".to_string());
    let vertical = orientation.to_ascii_uppercase() != "HORIZONTAL";
    let min_val = stack_val(state, 3);
    let max_val = stack_val(state, 4);
    let min_color = color_from_table(state, min_val);
    let max_color = color_from_table(state, max_val);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.gradient = Some(crate::widget::Gradient {
            vertical,
            min_color,
            max_color,
        });
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// SetVisuals — no-op (matches master)
// ---------------------------------------------------------------------------

pub(super) fn set_visuals(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    let override_fn = table_get(state, fields, "SetVisuals");
    if matches!(override_fn, Val::Function(_)) {
        let arg_count = state.top.saturating_sub(state.base) as i32;
        let args: Vec<Val> = (1..=arg_count)
            .map(|index| stack_val(state, index))
            .collect();
        let _ = call_function_state(state, override_fn, &args)?;
        return Ok(0);
    }
    let visual_args = create_table(state);
    if let Val::Table(table_ref) = visual_args {
        let arg_count = state.top.saturating_sub(state.base) as i32;
        let values: Vec<Val> = (2..=arg_count)
            .map(|index| stack_val(state, index))
            .collect();
        if let Some(table) = state.gc.tables.get_mut(table_ref) {
            for (offset, value) in values.into_iter().enumerate() {
                let key = (offset + 1) as f64;
                let _ = table.raw_set(Val::Num(key), value, &state.gc.string_arena);
            }
        }
        state.gc.barrier_back(table_ref);
    }
    table_set(state, fields, "visualArgs", visual_args);
    Ok(0)
}

// ---------------------------------------------------------------------------
// SetSpriteSheetCell — bounded, one-based row-major normalized UV mapping
// ---------------------------------------------------------------------------

fn positive_sprite_integer(value: Val, name: &str) -> LuaResult<f64> {
    match value {
        Val::Num(number) if number.is_finite() && number > 0.0 && number.fract() == 0.0 => {
            Ok(number)
        }
        _ => Err(rilua::runtime_error(&format!(
            "SetSpriteSheetCell: {name} must be a finite positive integer"
        ))),
    }
}

fn read_sprite_sheet_rect(state: &LuaState) -> LuaResult<[f32; 4]> {
    if !matches!(stack_val(state, 5), Val::Nil) || !matches!(stack_val(state, 6), Val::Nil) {
        return Err(rilua::runtime_error(
            "SetSpriteSheetCell: nonnil cellWidth/cellHeight are unmodeled",
        ));
    }
    let cell = positive_sprite_integer(stack_val(state, 2), "cell")?;
    let rows = positive_sprite_integer(stack_val(state, 3), "numRows")?;
    let columns = positive_sprite_integer(stack_val(state, 4), "numColumns")?;
    if cell > rows * columns {
        return Err(rilua::runtime_error(
            "SetSpriteSheetCell: cell exceeds grid",
        ));
    }
    let remainder = cell % columns;
    let column_end = if remainder == 0.0 { columns } else { remainder };
    let row_end = (cell / columns).ceil();
    Ok([
        ((column_end - 1.0) / columns) as f32,
        (column_end / columns) as f32,
        ((row_end - 1.0) / rows) as f32,
        (row_end / rows) as f32,
    ])
}

pub(super) fn set_sprite_sheet_cell(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let rect = read_sprite_sheet_rect(state)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        super::coords::apply_rect_tex_coords(frame, rect);
    }
    Ok(0)
}
