//! Atlas and texture-resolve methods.

use super::super::shared::opt_bool;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn set_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let arg_count = state.top.saturating_sub(state.base);
    if arg_count < 2 {
        return set_atlas_usage_error();
    }
    let atlas_name = match stack_val(state, 2) {
        Val::Str(_) => super::super::shared::opt_string(state, 2),
        Val::Nil => None,
        Val::Num(element_id) if element_id > 0.0 => {
            crate::atlas::get_atlas_name_by_element_id(element_id as u32).map(str::to_string)
        }
        Val::Num(_) => None,
        _ => return set_atlas_usage_error(),
    };
    let Some(atlas_name) = atlas_name else {
        return Ok(0);
    };
    if atlas_name.trim().is_empty() {
        let mut sim = borrow_state_mut(state)?;
        clear_atlas_texture(&mut sim.widgets, id);
        return Ok(0);
    }
    let Some(lookup) = crate::atlas::get_render_atlas_info(&atlas_name) else {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.atlas = Some(atlas_name);
            frame.texture = None;
            frame.tex_coords = None;
            frame.atlas_tex_coords = None;
        }
        return Ok(0);
    };
    let use_atlas_size = opt_bool(state, 3).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    apply_atlas(&mut sim.widgets, id, &atlas_name, &lookup, use_atlas_size);
    Ok(0)
}

fn set_atlas_usage_error() -> LuaResult<u32> {
    Err(runtime_error(
        "SetAtlas(): Usage: (\"atlasName\"[, useAtlasSize, filterMode, resetTexCoords, \
         wrapModeHorizontal, wrapModeVertical])",
    ))
}

fn clear_atlas_texture(widgets: &mut crate::widget::WidgetRegistry, id: u64) {
    let parent_info = collect_parent_slot(widgets, id);
    if let Some(frame) = widgets.get_mut_visual(id) {
        frame.atlas = None;
        frame.texture = None;
        frame.tex_coords = None;
        frame.atlas_tex_coords = None;
    }
    if let Some((parent_id, parent_key)) = parent_info {
        clear_button_slot(widgets, parent_id, &parent_key);
    }
}

/// Write the atlas onto the child frame, then mirror the slot into the
/// parent button's matching texture slot when applicable.
fn apply_atlas(
    widgets: &mut crate::widget::WidgetRegistry,
    id: u64,
    atlas_name: &str,
    lookup: &crate::atlas::AtlasLookup,
    use_atlas_size: bool,
) {
    let info = lookup.info;
    let tex_coords = atlas_slot_tex_coords(info);
    let parent_info = collect_parent_slot(widgets, id);
    apply_atlas_to_frame(widgets, id, atlas_name, lookup, tex_coords, use_atlas_size);
    if let Some((parent_id, parent_key)) = parent_info {
        propagate_atlas_to_button_slot(
            widgets,
            parent_id,
            &parent_key,
            info.file.to_string(),
            tex_coords,
        );
        // Only the button's own state slots follow the button state; a child
        // with any other parentKey keeps the shown state it has (the bag
        // slots' `SlotHighlightTexture` is declared hidden and gets its atlas
        // from UpdateTextures long before a bag opens).
        if let Some(should_show) = button_texture_should_show(widgets, parent_id, &parent_key) {
            widgets.set_visible(id, should_show);
        }
    }
}

/// Atlas slot UV box `(left, right, top, bottom)` for the matched atlas entry.
fn atlas_slot_tex_coords(info: &crate::atlas::AtlasInfo) -> (f32, f32, f32, f32) {
    (
        info.left_tex_coord,
        info.right_tex_coord,
        info.top_tex_coord,
        info.bottom_tex_coord,
    )
}

/// Parent id + parentKey when both are set. Captured before the child borrow
/// so the propagation step can run after the child mutation without
/// re-borrowing state.
fn collect_parent_slot(widgets: &crate::widget::WidgetRegistry, id: u64) -> Option<(u64, String)> {
    let frame = widgets.get(id)?;
    let parent_id = frame.parent_id?;
    let parent_key = frame.parent_key.clone()?;
    Some((parent_id, parent_key))
}

/// Write atlas name, source texture, and atlas UVs into the child frame.
/// When `use_atlas_size` is true, also resize the frame to the slot dimensions.
pub(crate) fn apply_atlas_to_frame(
    widgets: &mut crate::widget::WidgetRegistry,
    id: u64,
    atlas_name: &str,
    lookup: &crate::atlas::AtlasLookup,
    tex_coords: (f32, f32, f32, f32),
    use_atlas_size: bool,
) {
    let Some(frame) = widgets.get_mut_visual(id) else {
        return;
    };
    let info = lookup.info;
    frame.atlas = Some(atlas_name.to_string());
    frame.texture = Some(info.file.to_string());
    frame.tex_coords = Some(tex_coords);
    frame.atlas_tex_coords = Some(tex_coords);
    frame.horiz_tile = info.tiles_horizontally;
    frame.vert_tile = info.tiles_vertically;
    if use_atlas_size {
        frame.set_size(lookup.width() as f32, lookup.height() as f32);
    }
}

/// Copy atlas texture/UV data from a child texture onto the parent Button's
/// corresponding slot field when `parent_key` names a standard button slot.
fn propagate_atlas_to_button_slot(
    widgets: &mut crate::widget::WidgetRegistry,
    parent_id: u64,
    parent_key: &str,
    texture_path: String,
    tex_coords: (f32, f32, f32, f32),
) {
    let Some(parent) = widgets.get_mut_visual(parent_id) else {
        return;
    };
    if !matches!(
        parent.widget_type,
        crate::widget::WidgetType::Button | crate::widget::WidgetType::CheckButton
    ) {
        return;
    }
    match parent_key {
        "NormalTexture" => {
            parent.normal_texture = Some(texture_path);
            parent.normal_tex_coords = Some(tex_coords);
        }
        "PushedTexture" => {
            parent.pushed_texture = Some(texture_path);
            parent.pushed_tex_coords = Some(tex_coords);
        }
        "HighlightTexture" => {
            parent.highlight_texture = Some(texture_path);
            parent.highlight_tex_coords = Some(tex_coords);
        }
        "DisabledTexture" => {
            parent.disabled_texture = Some(texture_path);
            parent.disabled_tex_coords = Some(tex_coords);
        }
        "CheckedTexture" => {
            parent.checked_texture = Some(texture_path);
            parent.checked_tex_coords = Some(tex_coords);
        }
        "DisabledCheckedTexture" => {
            parent.disabled_checked_texture = Some(texture_path);
            parent.disabled_checked_tex_coords = Some(tex_coords);
        }
        _ => {}
    }
}

fn clear_button_slot(
    widgets: &mut crate::widget::WidgetRegistry,
    parent_id: u64,
    parent_key: &str,
) {
    let Some(parent) = widgets.get_mut_visual(parent_id) else {
        return;
    };
    match parent_key {
        "NormalTexture" => {
            parent.normal_texture = None;
            parent.normal_tex_coords = None;
        }
        "PushedTexture" => {
            parent.pushed_texture = None;
            parent.pushed_tex_coords = None;
        }
        "HighlightTexture" => {
            parent.highlight_texture = None;
            parent.highlight_tex_coords = None;
        }
        "DisabledTexture" => {
            parent.disabled_texture = None;
            parent.disabled_tex_coords = None;
        }
        "CheckedTexture" => {
            parent.checked_texture = None;
            parent.checked_tex_coords = None;
        }
        "DisabledCheckedTexture" => {
            parent.disabled_checked_texture = None;
            parent.disabled_checked_tex_coords = None;
        }
        _ => {}
    }
}

/// Whether a button's state-slot texture is shown for the button's current
/// state; `None` for a parentKey that is not one of the six slots.
fn button_texture_should_show(
    widgets: &crate::widget::WidgetRegistry,
    button_id: u64,
    parent_key: &str,
) -> Option<bool> {
    let (enabled, checked, button_state) = widgets
        .get(button_id)
        .map(|frame| {
            let enabled = frame
                .attributes
                .get("__enabled")
                .and_then(|value| match value {
                    crate::widget::AttributeValue::Boolean(flag) => Some(*flag),
                    _ => None,
                })
                .unwrap_or(true);
            let checked = frame
                .attributes
                .get("__checked")
                .and_then(|value| match value {
                    crate::widget::AttributeValue::Boolean(flag) => Some(*flag),
                    _ => None,
                })
                .unwrap_or(false);
            (enabled, checked, frame.button_state)
        })
        .unwrap_or((true, false, 0));

    match parent_key {
        "NormalTexture" => Some(enabled && button_state == 0),
        "PushedTexture" => Some(enabled && button_state == 1),
        "DisabledTexture" => Some(!enabled),
        "HighlightTexture" => Some(false),
        "CheckedTexture" => Some(enabled && checked),
        "DisabledCheckedTexture" => Some(!enabled && checked),
        _ => None,
    }
}

pub(super) fn set_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let texture_val = stack_val(state, 2);
    let horiz_tile = opt_bool(state, 3);
    let vert_tile = opt_bool(state, 4);
    let mut sim = borrow_state_mut(state)?;
    let mut order_changed = false;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let Some((path, file_data_id)) = resolve_texture_value(state, texture_val) else {
            return Ok(0);
        };
        let had_render_source = texture_has_render_source(frame);
        let has_render_source = path.is_some() || file_data_id.is_some();
        let changed = texture_assignment_changed(frame, &path, file_data_id, horiz_tile, vert_tile);
        frame.texture = path;
        frame.texture_file_data_id = file_data_id;
        frame.color_texture = None;
        clear_atlas_owned_tex_coords(frame);
        frame.atlas = None;
        frame.atlas_tex_coords = None;
        apply_texture_tiling_flags(frame, horiz_tile, vert_tile);
        order_changed = changed && !had_render_source && has_render_source;
        if order_changed {
            frame.region_order = crate::widget::next_region_order();
        }
    }
    if order_changed {
        sim.invalidate_strata_buckets();
    }
    Ok(0)
}

fn texture_has_render_source(frame: &crate::widget::Frame) -> bool {
    frame.texture.is_some()
        || frame.texture_file_data_id.is_some()
        || frame.color_texture.is_some()
        || frame.atlas.is_some()
        || frame.atlas_tex_coords.is_some()
}

fn texture_assignment_changed(
    frame: &crate::widget::Frame,
    path: &Option<String>,
    file_data_id: Option<i64>,
    horiz_tile: Option<bool>,
    vert_tile: Option<bool>,
) -> bool {
    frame.texture.as_ref() != path.as_ref()
        || frame.texture_file_data_id != file_data_id
        || frame.color_texture.is_some()
        || frame.atlas.is_some()
        || frame.atlas_tex_coords.is_some()
        || horiz_tile.is_some_and(|enabled| frame.horiz_tile != enabled)
        || vert_tile.is_some_and(|enabled| frame.vert_tile != enabled)
}

fn apply_texture_tiling_flags(
    frame: &mut crate::widget::Frame,
    horiz_tile: Option<bool>,
    vert_tile: Option<bool>,
) {
    if let Some(enabled) = horiz_tile {
        frame.horiz_tile = enabled;
    }
    if let Some(enabled) = vert_tile {
        frame.vert_tile = enabled;
    }
}

fn clear_atlas_owned_tex_coords(frame: &mut crate::widget::Frame) {
    if frame.atlas.is_none() {
        return;
    }
    if frame.tex_coords == frame.atlas_tex_coords {
        frame.tex_coords = None;
    }
}

fn resolve_texture_value(state: &LuaState, value: Val) -> Option<(Option<String>, Option<i64>)> {
    match value {
        Val::Str(_) => Some(resolve_texture_string(state, value)),
        Val::Num(number) if number == 0.0 => Some((None, None)),
        Val::Num(number) => {
            let file_data_id = number as u32;
            Some((
                Some(resolve_file_data_id_path(file_data_id)),
                Some(file_data_id as i64),
            ))
        }
        Val::Nil => Some((None, None)),
        _ => None,
    }
}

fn resolve_texture_string(state: &LuaState, value: Val) -> (Option<String>, Option<i64>) {
    let Some(raw) = val_to_string(state, value) else {
        return (None, None);
    };
    if raw.trim().is_empty() {
        return (None, None);
    }
    let Ok(file_data_id) = raw.parse::<u32>() else {
        let file_data_id = crate::limited_listfile::lookup_texture_path(&raw);
        return (Some(raw), file_data_id.map(i64::from));
    };
    (
        Some(resolve_file_data_id_path(file_data_id)),
        Some(file_data_id as i64),
    )
}

fn resolve_file_data_id_path(file_data_id: u32) -> String {
    crate::manifest_interface_data::get_texture_path(file_data_id)
        .map(|path| format!("Interface\\{}", path.replace('/', "\\")))
        .unwrap_or_else(|| file_data_id.to_string())
}

pub(super) fn get_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (file_id, path) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        (
            frame.and_then(|frame| frame.texture_file_data_id),
            frame.and_then(|frame| frame.texture.clone()),
        )
    };
    let value = if let Some(file_id) = file_id {
        Val::Num(file_id as f64)
    } else if let Some(path) = path {
        create_string(state, &path)
    } else {
        Val::Nil
    };
    state.push(value);
    Ok(1)
}

pub(super) fn get_texture_file_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let file_id = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.texture_file_data_id);
    match file_id {
        Some(file_id) => state.push(Val::Num(file_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn get_texture_file_path(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.texture.clone());
    match path {
        Some(path) => {
            let path_val = create_string(state, &path);
            state.push(path_val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn get_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let atlas = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.atlas.clone());
    match atlas {
        Some(atlas) => {
            let atlas_val = create_string(state, &atlas);
            state.push(atlas_val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}
