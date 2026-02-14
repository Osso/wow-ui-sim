//! Texture-related methods: SetTexture, SetAtlas, SetTexCoord, etc.

use super::methods_helpers::resolve_file_data_id_or_path;
use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use crate::widget::{Frame, WidgetType};
use mlua::{LightUserData, Lua, Value};

/// Add texture-related methods to the shared methods table.
pub fn add_texture_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_texture_path_methods(lua, methods)?;
    add_tiling_methods(lua, methods)?;
    add_blend_and_desaturation_methods(lua, methods)?;
    add_atlas_methods(lua, methods)?;
    add_pixel_grid_methods(lua, methods)?;
    add_nine_slice_methods(lua, methods)?;
    add_vertex_color_methods(lua, methods)?;
    add_tex_coord_methods(lua, methods)?;
    add_mask_methods(lua, methods)?;
    add_rotation_methods(lua, methods)?;
    add_draw_layer_methods(lua, methods)?;
    add_visual_methods(lua, methods)?;
    Ok(())
}

/// SetTexture, GetTexture, SetColorTexture.
fn add_texture_path_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTexture", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let path = args_vec.first().map(resolve_file_data_id_or_path).unwrap_or(None);
        let horiz_tile = args_vec.get(1).and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            _ => None,
        });
        let vert_tile = args_vec.get(2).and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            _ => None,
        });
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.texture = path;
            if let Some(h) = horiz_tile { frame.horiz_tile = h; }
            if let Some(v) = vert_tile { frame.vert_tile = v; }
        }
        Ok(())
    })?)?;

    methods.set("GetTexture", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let texture = state
            .widgets
            .get(id)
            .and_then(|f| f.texture.clone());
        Ok(texture)
    })?)?;

    methods.set("SetColorTexture", lua.create_function(|lua, (ud, r, g, b, a): (LightUserData, f32, f32, f32, Option<f32>)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.color_texture =
                Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
            // Clear file texture when setting color texture
            frame.texture = None;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// SetHorizTile, GetHorizTile, SetVertTile, GetVertTile.
fn add_tiling_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetHorizTile", lua.create_function(|lua, (ud, tile): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.horiz_tile = tile;
        }
        Ok(())
    })?)?;

    methods.set("GetHorizTile", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let tile = state
            .widgets
            .get(id)
            .map(|f| f.horiz_tile)
            .unwrap_or(false);
        Ok(tile)
    })?)?;

    methods.set("SetVertTile", lua.create_function(|lua, (ud, tile): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.vert_tile = tile;
        }
        Ok(())
    })?)?;

    methods.set("GetVertTile", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let tile = state
            .widgets
            .get(id)
            .map(|f| f.vert_tile)
            .unwrap_or(false);
        Ok(tile)
    })?)?;

    Ok(())
}

/// SetBlendMode, GetBlendMode, SetDesaturated, IsDesaturated.
fn add_blend_and_desaturation_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_blend_mode_methods(lua, methods)?;
    add_desaturation_methods(lua, methods)?;
    Ok(())
}

/// SetBlendMode, GetBlendMode.
fn add_blend_mode_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetBlendMode", lua.create_function(|lua, (ud, mode): (LightUserData, Option<String>)| {
        let id = lud_to_id(ud);
        let blend = match mode.as_deref() {
            Some("ADD") => crate::render::BlendMode::Additive,
            _ => crate::render::BlendMode::Alpha,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut_visual(id) {
            f.blend_mode = blend;
        }
        Ok(())
    })?)?;

    methods.set("GetBlendMode", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(match state.widgets.get(id).map(|f| f.blend_mode) {
            Some(crate::render::BlendMode::Additive) => "ADD",
            _ => "BLEND",
        })
    })?)?;

    Ok(())
}

/// SetDesaturated, IsDesaturated, GetDesaturation, SetDesaturation.
fn add_desaturation_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetDesaturated", lua.create_function(|lua, (ud, desaturated): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut_visual(id) {
            f.desaturated = desaturated;
        }
        Ok(())
    })?)?;

    methods.set("IsDesaturated", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.desaturated).unwrap_or(false))
    })?)?;

    methods.set("GetDesaturation", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(if state.widgets.get(id).map(|f| f.desaturated).unwrap_or(false) { 1.0_f64 } else { 0.0 })
    })?)?;

    methods.set("SetDesaturation", lua.create_function(|lua, (ud, desat): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(f) = state.widgets.get_mut_visual(id) {
            f.desaturated = desat > 0.0;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Resolve atlas name from a Lua value (string or numeric element ID).
fn resolve_atlas_name(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.to_string_lossy().to_string()),
        Value::Integer(id) => {
            crate::atlas::get_atlas_name_by_element_id(*id as u32)
                .map(|s| s.to_string())
        }
        _ => None,
    }
}

/// SetAtlas, GetAtlas.
fn add_atlas_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords)
    methods.set("SetAtlas", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let atlas_name = args_vec.first().and_then(resolve_atlas_name);
        let use_atlas_size = args_vec
            .get(1)
            .map(|v| matches!(v, Value::Boolean(true)))
            .unwrap_or(false);

        if let Some(name) = atlas_name {
            apply_set_atlas(lua, id, &name, use_atlas_size)?;
        }
        Ok(())
    })?)?;

    // GetAtlas() - Get current atlas name
    // NOTE: Mixins can override GetAtlas with a Lua function. Since the methods
    // table takes priority over __index, we check for Lua overrides
    // in __frame_fields before using the default.
    methods.set("GetAtlas", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        if let Some(result) = call_lua_override(lua, id, "GetAtlas")? {
            return Ok(result);
        }
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let atlas = state.widgets.get(id).and_then(|f| f.atlas.clone());
        match atlas {
            Some(name) => Ok(Value::String(lua.create_string(&name)?)),
            None => Ok(Value::Nil),
        }
    })?)?;

    Ok(())
}

/// Apply SetAtlas logic: look up atlas info, apply nine-slice or regular atlas.
fn apply_set_atlas(lua: &Lua, id: u64, name: &str, use_atlas_size: bool) -> mlua::Result<()> {
    let lookup = crate::atlas::get_atlas_info(name);
    // When the only match is a -2x fallback, prefer a nine-slice kit
    let prefer_nine_slice = lookup.as_ref().is_some_and(|l| l.is_2x_fallback);
    let ns_info = if lookup.is_none() || prefer_nine_slice {
        crate::atlas::get_nine_slice_atlas_info(name)
    } else {
        None
    };

    let state_rc = get_sim_state(lua);
    if let Some(ns_info) = ns_info {
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.nine_slice_atlas = Some(ns_info);
            frame.atlas = Some(name.to_string());
            frame.texture = None;
            frame.tex_coords = None;
            frame.tex_coords_quad = None;
        }
    } else if let Some(lookup) = lookup {
        let atlas_info = lookup.info;
        let mut state = state_rc.borrow_mut();
        let parent_info = find_parent_key(&state.widgets, id);
        apply_atlas_to_frame(&mut state.widgets, id, atlas_info, name, &lookup, use_atlas_size);
        propagate_atlas_to_button(&mut state.widgets, parent_info, atlas_info);
        if use_atlas_size {
            state.invalidate_layout_with_dependents(id);
        }
    } else {
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.atlas = Some(name.to_string());
        }
    }
    Ok(())
}

/// Check __frame_fields for a Lua override of a method and call it if present.
fn call_lua_override(
    lua: &Lua,
    id: u64,
    method_name: &str,
) -> mlua::Result<Option<Value>> {
    if let Some(fields_table) = crate::lua_api::script_helpers::get_frame_fields_table(lua)
        && let Ok(frame_fields) = fields_table.get::<mlua::Table>(id)
            && let Ok(Value::Function(f)) = frame_fields.get::<Value>(method_name) {
                return Ok(Some(f.call::<Value>(frame_lud(id))?));
            }
    Ok(None)
}

/// Find which parent_key this frame is registered as in its parent's children_keys.
fn find_parent_key(
    widgets: &crate::widget::WidgetRegistry,
    frame_id: u64,
) -> Option<(u64, Option<String>)> {
    widgets.get(frame_id).and_then(|f| {
        f.parent_id.and_then(|pid| {
            widgets.get(pid).map(|parent| {
                let key = parent
                    .children_keys
                    .iter()
                    .find(|(_, child_id)| **child_id == frame_id)
                    .map(|(k, _)| k.clone());
                (pid, key)
            })
        })
    })
}

/// Apply atlas info to a frame: set texture, UVs, tiling, atlas name, and optionally size.
fn apply_atlas_to_frame(
    widgets: &mut crate::widget::WidgetRegistry,
    frame_id: u64,
    atlas_info: &crate::atlas::AtlasInfo,
    atlas_name: &str,
    lookup: &crate::atlas::AtlasLookup,
    use_atlas_size: bool,
) {
    if let Some(frame) = widgets.get_mut_visual(frame_id) {
        frame.texture = Some(atlas_info.file.to_string());
        let atlas_uvs = (
            atlas_info.left_tex_coord,
            atlas_info.right_tex_coord,
            atlas_info.top_tex_coord,
            atlas_info.bottom_tex_coord,
        );
        frame.atlas_tex_coords = Some(atlas_uvs);
        frame.tex_coords = Some(atlas_uvs);
        frame.horiz_tile = atlas_info.tiles_horizontally;
        frame.vert_tile = atlas_info.tiles_vertically;
        frame.atlas = Some(atlas_name.to_string());
        frame.three_slice_h = three_slice_caps_for_atlas(atlas_name, atlas_info.width);
        if use_atlas_size {
            frame.width = lookup.width() as f32;
            frame.height = lookup.height() as f32;
        }
    }
}

/// Return horizontal three-slice cap info for known atlas entries.
/// Returns (left_cap_px, right_cap_px, atlas_entry_width_px).
fn three_slice_caps_for_atlas(atlas_name: &str, atlas_width: u32) -> Option<(f32, f32, f32)> {
    let w = atlas_width as f32;
    match atlas_name {
        "common-dropdown-textholder" => Some((12.0, 12.0, w)),
        _ => None,
    }
}

/// If the frame is a standard button texture child (NormalTexture, PushedTexture, etc.),
/// propagate the atlas texture path and UV coords to the parent button.
fn propagate_atlas_to_button(
    widgets: &mut crate::widget::WidgetRegistry,
    parent_info: Option<(u64, Option<String>)>,
    atlas_info: &crate::atlas::AtlasInfo,
) {
    let Some((parent_id, Some(parent_key))) = parent_info else {
        return;
    };
    let Some(parent) = widgets.get_mut_visual(parent_id) else {
        return;
    };
    if !matches!(
        parent.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return;
    }
    let texture_path = atlas_info.file.to_string();
    let tex_coords = (
        atlas_info.left_tex_coord,
        atlas_info.right_tex_coord,
        atlas_info.top_tex_coord,
        atlas_info.bottom_tex_coord,
    );
    set_button_texture_field(parent, &parent_key, texture_path, tex_coords);
}

/// Set the appropriate texture field on a button based on the parent key name.
fn set_button_texture_field(
    parent: &mut Frame,
    parent_key: &str,
    texture_path: String,
    tex_coords: (f32, f32, f32, f32),
) {
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

/// SetSnapToPixelGrid, IsSnappingToPixelGrid, SetTexelSnappingBias, GetTexelSnappingBias.
fn add_pixel_grid_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetSnapToPixelGrid", lua.create_function(|_, (_ud, _snap): (LightUserData, bool)| Ok(()))?)?;
    methods.set("IsSnappingToPixelGrid", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetTexelSnappingBias", lua.create_function(|_, (_ud, _bias): (LightUserData, f32)| Ok(()))?)?;
    methods.set("GetTexelSnappingBias", lua.create_function(|_, _ud: LightUserData| Ok(0.0_f32))?)?;
    Ok(())
}

/// SetTextureSliceMargins, GetTextureSliceMargins, SetTextureSliceMode,
/// GetTextureSliceMode, ClearTextureSlice.
fn add_nine_slice_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextureSliceMargins", lua.create_function(
        |_, (_ud, _left, _right, _top, _bottom): (LightUserData, f32, f32, f32, f32)| Ok(()),
    )?)?;
    methods.set("GetTextureSliceMargins", lua.create_function(
        |_, _ud: LightUserData| Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32)),
    )?)?;
    methods.set("SetTextureSliceMode", lua.create_function(|_, (_ud, _mode): (LightUserData, i32)| Ok(()))?)?;
    methods.set("GetTextureSliceMode", lua.create_function(|_, _ud: LightUserData| Ok(0i32))?)?;
    methods.set("ClearTextureSlice", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}

/// SetVertexColor, GetVertexColor, SetCenterColor.
fn add_vertex_color_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetVertexColor", lua.create_function(
        |lua, (ud, r, g, b, a): (LightUserData, f32, f32, f32, Option<f32>)| {
            let id = lud_to_id(ud);
            let new_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
            let state_rc = get_sim_state(lua);
            let already_set = state_rc.borrow().widgets.get(id)
                .and_then(|f| f.vertex_color.as_ref())
                .map(|c| c.r == new_color.r && c.g == new_color.g && c.b == new_color.b && c.a == new_color.a)
                .unwrap_or(false);
            if !already_set {
                let mut state = state_rc.borrow_mut();
                if let Some(frame) = state.widgets.get_mut_visual(id) {
                    frame.vertex_color = Some(new_color);
                }
            }
            Ok(())
        },
    )?)?;

    methods.set("GetVertexColor", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(color) = &frame.vertex_color {
                return Ok((color.r, color.g, color.b, color.a));
            }
        Ok((1.0f32, 1.0f32, 1.0f32, 1.0f32)) // Default white
    })?)?;

    // SetCenterColor(r, g, b, a) - for NineSlice frames (sets center fill color)
    methods.set("SetCenterColor", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    Ok(())
}

/// GetTexCoord, SetTexCoord - with atlas-relative coordinate remapping.
fn add_tex_coord_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // GetTexCoord() - returns UL.x, UL.y, LL.x, LL.y, UR.x, UR.y, LR.x, LR.y (8 values)
    methods.set("GetTexCoord", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some((left, right, top, bottom)) = frame.tex_coords {
                // Return 8 values: UL, LL, UR, LR corners
                return Ok((left, top, left, bottom, right, top, right, bottom));
            }
        // Default: full texture
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 1.0_f32))
    })?)?;

    methods.set("SetTexCoord", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        let (raw_quad, left, right, top, bottom) = parse_tex_coord_args(&args_vec);
        let (Some(left), Some(right), Some(top), Some(bottom)) = (left, right, top, bottom) else {
            return Ok(());
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.tex_coords =
                Some(remap_tex_coords(frame.atlas_tex_coords, left, right, top, bottom));
            frame.tex_coords_quad = raw_quad;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Parse SetTexCoord arguments into raw quad and (left, right, top, bottom).
/// Returns (raw_quad, left, right, top, bottom) where None values mean insufficient args.
fn parse_tex_coord_args(args_vec: &[Value]) -> (Option<[f32; 8]>, Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
    if args_vec.len() >= 8 {
        parse_tex_coord_8_args(args_vec)
    } else if args_vec.len() >= 4 {
        let coords = (
            value_to_f32(&args_vec[0], 0.0),
            value_to_f32(&args_vec[1], 1.0),
            value_to_f32(&args_vec[2], 0.0),
            value_to_f32(&args_vec[3], 1.0),
        );
        (None, Some(coords.0), Some(coords.1), Some(coords.2), Some(coords.3))
    } else {
        (None, None, None, None, None)
    }
}

/// Parse 8-arg form: ULx, ULy, LLx, LLy, URx, URy, LRx, LRy.
fn parse_tex_coord_8_args(args_vec: &[Value]) -> (Option<[f32; 8]>, Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
    let ul_x = value_to_f32(&args_vec[0], 0.0);
    let ul_y = value_to_f32(&args_vec[1], 0.0);
    let ll_x = value_to_f32(&args_vec[2], 0.0);
    let ll_y = value_to_f32(&args_vec[3], 1.0);
    let ur_x = value_to_f32(&args_vec[4], 1.0);
    let ur_y = value_to_f32(&args_vec[5], 0.0);
    let lr_x = value_to_f32(&args_vec[6], 1.0);
    let lr_y = value_to_f32(&args_vec[7], 1.0);
    let raw_quad = Some([ul_x, ul_y, ll_x, ll_y, ur_x, ur_y, lr_x, lr_y]);
    // Compute axis-aligned bounding box from all 4 corners.
    let left = ul_x.min(ll_x).min(ur_x).min(lr_x);
    let right = ul_x.max(ll_x).max(ur_x).max(lr_x);
    let top = ul_y.min(ll_y).min(ur_y).min(lr_y);
    let bottom = ul_y.max(ll_y).max(ur_y).max(lr_y);
    (raw_quad, Some(left), Some(right), Some(top), Some(bottom))
}

/// Convert a Lua Value to f32, with a default if it's not a number.
fn value_to_f32(value: &Value, default: f32) -> f32 {
    match value {
        Value::Number(n) => *n as f32,
        Value::Integer(n) => *n as f32,
        _ => default,
    }
}

/// Remap texture coordinates relative to atlas sub-region if active,
/// otherwise return them as-is.
fn remap_tex_coords(
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

/// AddMaskTexture, RemoveMaskTexture, GetNumMaskTextures, GetMaskTexture.
fn add_mask_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("AddMaskTexture", lua.create_function(|lua, (ud, mask): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let mask_id = extract_frame_id(&mask);
        if let Some(mask_id) = mask_id {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                if !frame.mask_textures.contains(&mask_id) {
                    frame.mask_textures.push(mask_id);
                }
            }
        }
        Ok(())
    })?)?;

    methods.set("RemoveMaskTexture", lua.create_function(|lua, (ud, mask): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let mask_id = extract_frame_id(&mask);
        if let Some(mask_id) = mask_id {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.mask_textures.retain(|&mid| mid != mask_id);
            }
        }
        Ok(())
    })?)?;

    methods.set("GetNumMaskTextures", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map_or(0, |f| f.mask_textures.len()))
    })?)?;

    methods.set("GetMaskTexture", lua.create_function(|_, (_ud, _index): (LightUserData, i32)| Ok(Value::Nil))?)?;

    Ok(())
}

/// SetRotation, GetRotation - rotate texture UVs around center.
fn add_rotation_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetRotation", lua.create_function(|lua, (ud, radians): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.rotation = radians as f32;
        }
        Ok(())
    })?)?;

    methods.set("GetRotation", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state
            .widgets
            .get(id)
            .map(|f| f.rotation as f64)
            .unwrap_or(0.0))
    })?)?;

    Ok(())
}

/// SetGradient, SetDrawLayer, GetDrawLayer.
fn add_draw_layer_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetGradient", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    methods.set("SetDrawLayer", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        use crate::widget::DrawLayer;
        let id = lud_to_id(ud);
        let args_vec: Vec<Value> = args.into_iter().collect();
        if let Some(Value::String(s)) = args_vec.first() {
            let layer_str = s.to_string_lossy();
            if let Some(layer) = DrawLayer::from_str(&layer_str) {
                let state_rc = get_sim_state(lua);
                let mut state = state_rc.borrow_mut();
                if let Some(frame) = state.widgets.get_mut_visual(id) {
                    frame.draw_layer = layer;
                    // Second arg is sublevel (default 0, range -8..7)
                    if let Some(sub_val) = args_vec.get(1) {
                        frame.draw_sub_layer = match sub_val {
                            Value::Integer(n) => *n as i32,
                            Value::Number(n) => *n as i32,
                            _ => 0,
                        };
                    }
                }
            }
        }
        Ok(())
    })?)?;

    methods.set("GetDrawLayer", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(f) = state.widgets.get(id) {
            Ok((f.draw_layer.as_str().to_string(), f.draw_sub_layer))
        } else {
            Ok(("ARTWORK".to_string(), 0i32))
        }
    })?)?;

    Ok(())
}

/// SetVisuals - used by StatusBar spark textures in UnitFrame.
fn add_visual_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetVisuals", lua.create_function(|_, (_ud, _info): (LightUserData, Value)| Ok(()))?)?;
    Ok(())
}
