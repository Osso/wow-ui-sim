//! Texture-related methods: SetTexture, SetAtlas, SetTexCoord, etc.

use super::FrameHandle;
use crate::widget::{Frame, WidgetType};
use mlua::{UserDataMethods, Value};

/// Add texture-related methods to FrameHandle UserData.
pub fn add_texture_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_texture_path_methods(methods);
    add_tiling_methods(methods);
    add_blend_and_desaturation_methods(methods);
    add_atlas_methods(methods);
    add_pixel_grid_methods(methods);
    add_nine_slice_methods(methods);
    add_vertex_color_methods(methods);
    add_tex_coord_methods(methods);
    add_mask_methods(methods);
    add_draw_layer_methods(methods);
    add_visual_methods(methods);
}

/// SetTexture, GetTexture, SetColorTexture.
fn add_texture_path_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTexture", |_, this, path: Option<String>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.texture = path;
        }
        Ok(())
    });

    methods.add_method("GetTexture", |_, this, ()| {
        let state = this.state.borrow();
        let texture = state
            .widgets
            .get(this.id)
            .and_then(|f| f.texture.clone());
        Ok(texture)
    });

    methods.add_method(
        "SetColorTexture",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.color_texture =
                    Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
                // Clear file texture when setting color texture
                frame.texture = None;
            }
            Ok(())
        },
    );
}

/// SetHorizTile, GetHorizTile, SetVertTile, GetVertTile.
fn add_tiling_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetHorizTile", |_, this, tile: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.horiz_tile = tile;
        }
        Ok(())
    });

    methods.add_method("GetHorizTile", |_, this, ()| {
        let state = this.state.borrow();
        let tile = state
            .widgets
            .get(this.id)
            .map(|f| f.horiz_tile)
            .unwrap_or(false);
        Ok(tile)
    });

    methods.add_method("SetVertTile", |_, this, tile: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.vert_tile = tile;
        }
        Ok(())
    });

    methods.add_method("GetVertTile", |_, this, ()| {
        let state = this.state.borrow();
        let tile = state
            .widgets
            .get(this.id)
            .map(|f| f.vert_tile)
            .unwrap_or(false);
        Ok(tile)
    });
}

/// SetBlendMode, GetBlendMode, SetDesaturated, IsDesaturated.
fn add_blend_and_desaturation_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetBlendMode", |_, this, mode: Option<String>| {
        let blend = match mode.as_deref() {
            Some("ADD") => crate::render::BlendMode::Additive,
            _ => crate::render::BlendMode::Alpha,
        };
        let mut state = this.state.borrow_mut();
        if let Some(f) = state.widgets.get_mut(this.id) {
            f.blend_mode = blend;
        }
        Ok(())
    });

    methods.add_method("GetBlendMode", |_, this, ()| {
        let state = this.state.borrow();
        Ok(match state.widgets.get(this.id).map(|f| f.blend_mode) {
            Some(crate::render::BlendMode::Additive) => "ADD",
            _ => "BLEND",
        })
    });

    methods.add_method("SetDesaturated", |_, _this, _desaturated: bool| {
        // Stub - desaturation is a rendering effect
        Ok(())
    });

    methods.add_method("IsDesaturated", |_, _this, ()| Ok(false));
}

/// SetAtlas, GetAtlas.
fn add_atlas_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords)
    methods.add_method("SetAtlas", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let atlas_name = args_vec.first().and_then(|v| match v {
            Value::String(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        });
        let use_atlas_size = args_vec
            .get(1)
            .map(|v| matches!(v, Value::Boolean(true)))
            .unwrap_or(false);

        if let Some(name) = atlas_name {
            let lookup = crate::atlas::get_atlas_info(&name);
            // When the only match is a -2x fallback, prefer a nine-slice kit
            let prefer_nine_slice = lookup.as_ref().is_some_and(|l| l.is_2x_fallback);
            let ns_info = if lookup.is_none() || prefer_nine_slice {
                crate::atlas::get_nine_slice_atlas_info(&name)
            } else {
                None
            };

            if let Some(ns_info) = ns_info {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.nine_slice_atlas = Some(ns_info);
                    frame.atlas = Some(name);
                    frame.texture = None;
                    frame.tex_coords = None;
                }
            } else if let Some(lookup) = lookup {
                let atlas_info = lookup.info;
                let mut state = this.state.borrow_mut();
                let parent_info = find_parent_key(&state.widgets, this.id);
                apply_atlas_to_frame(&mut state.widgets, this.id, atlas_info, &name, &lookup, use_atlas_size);
                propagate_atlas_to_button(&mut state.widgets, parent_info, atlas_info);
            } else {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.atlas = Some(name);
                }
            }
        }
        Ok(())
    });

    // GetAtlas() - Get current atlas name
    // NOTE: Mixins can override GetAtlas with a Lua function. Since mlua's
    // add_method takes priority over __index, we check for Lua overrides
    // in __frame_fields before using the default.
    methods.add_method("GetAtlas", |lua, this, ()| {
        if let Some(result) = call_lua_override(lua, this, "GetAtlas")? {
            return Ok(result);
        }
        let state = this.state.borrow();
        let atlas = state.widgets.get(this.id).and_then(|f| f.atlas.clone());
        match atlas {
            Some(name) => Ok(Value::String(lua.create_string(&name)?)),
            None => Ok(Value::Nil),
        }
    });
}

/// Check __frame_fields for a Lua override of a method and call it if present.
fn call_lua_override(
    lua: &mlua::Lua,
    this: &FrameHandle,
    method_name: &str,
) -> mlua::Result<Option<Value>> {
    if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields")
        && let Ok(frame_fields) = fields_table.get::<mlua::Table>(this.id)
            && let Ok(Value::Function(f)) = frame_fields.get::<Value>(method_name) {
                let ud = lua.create_userdata(FrameHandle {
                    id: this.id,
                    state: std::rc::Rc::clone(&this.state),
                })?;
                return Ok(Some(f.call::<Value>(ud)?));
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
    if let Some(frame) = widgets.get_mut(frame_id) {
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
        if use_atlas_size {
            frame.width = lookup.width() as f32;
            frame.height = lookup.height() as f32;
        }
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
    let Some(parent) = widgets.get_mut(parent_id) else {
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
fn add_pixel_grid_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetSnapToPixelGrid", |_, _this, _snap: bool| Ok(()));

    methods.add_method("IsSnappingToPixelGrid", |_, _this, ()| Ok(false));

    methods.add_method("SetTexelSnappingBias", |_, _this, _bias: f32| Ok(()));

    methods.add_method("GetTexelSnappingBias", |_, _this, ()| Ok(0.0_f32));
}

/// SetTextureSliceMargins, GetTextureSliceMargins, SetTextureSliceMode,
/// GetTextureSliceMode, ClearTextureSlice.
fn add_nine_slice_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetTextureSliceMargins",
        |_, _this, (_left, _right, _top, _bottom): (f32, f32, f32, f32)| Ok(()),
    );

    methods.add_method("GetTextureSliceMargins", |_, _this, ()| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });

    methods.add_method("SetTextureSliceMode", |_, _this, _mode: i32| Ok(()));

    methods.add_method("GetTextureSliceMode", |_, _this, ()| Ok(0i32));

    methods.add_method("ClearTextureSlice", |_, _this, ()| Ok(()));
}

/// SetVertexColor, GetVertexColor, SetCenterColor.
fn add_vertex_color_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetVertexColor",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let new_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
            let already_set = this.state.borrow().widgets.get(this.id)
                .and_then(|f| f.vertex_color.as_ref())
                .map(|c| c.r == new_color.r && c.g == new_color.g && c.b == new_color.b && c.a == new_color.a)
                .unwrap_or(false);
            if !already_set {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.vertex_color = Some(new_color);
                }
            }
            Ok(())
        },
    );

    methods.add_method("GetVertexColor", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(color) = &frame.vertex_color {
                return Ok((color.r, color.g, color.b, color.a));
            }
        Ok((1.0f32, 1.0f32, 1.0f32, 1.0f32)) // Default white
    });

    // SetCenterColor(r, g, b, a) - for NineSlice frames (sets center fill color)
    methods.add_method("SetCenterColor", |_, _this, _args: mlua::MultiValue| Ok(()));
}

/// GetTexCoord, SetTexCoord - with atlas-relative coordinate remapping.
fn add_tex_coord_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetTexCoord() - returns UL.x, UL.y, LL.x, LL.y, UR.x, UR.y, LR.x, LR.y (8 values)
    methods.add_method("GetTexCoord", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some((left, right, top, bottom)) = frame.tex_coords {
                // Return 8 values: UL, LL, UR, LR corners
                return Ok((left, top, left, bottom, right, top, right, bottom));
            }
        // Default: full texture
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 1.0_f32))
    });

    methods.add_method("SetTexCoord", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        if args_vec.len() >= 4 {
            let left = value_to_f32(&args_vec[0], 0.0);
            let right = value_to_f32(&args_vec[1], 1.0);
            let top = value_to_f32(&args_vec[2], 0.0);
            let bottom = value_to_f32(&args_vec[3], 1.0);

            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.tex_coords =
                    Some(remap_tex_coords(frame.atlas_tex_coords, left, right, top, bottom));
            }
        }
        Ok(())
    });
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
fn add_mask_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("AddMaskTexture", |_, _this, _mask: Value| Ok(()));

    methods.add_method("RemoveMaskTexture", |_, _this, _mask: Value| Ok(()));

    methods.add_method("GetNumMaskTextures", |_, _this, ()| Ok(0));

    methods.add_method("GetMaskTexture", |_, _this, _index: i32| Ok(Value::Nil));
}

/// SetGradient, SetDrawLayer, GetDrawLayer.
fn add_draw_layer_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetGradient", |_, _this, _args: mlua::MultiValue| Ok(()));

    methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| Ok(()));

    methods.add_method("GetDrawLayer", |_, _this, ()| {
        Ok(("ARTWORK", 0i32))
    });
}

/// SetVisuals - used by StatusBar spark textures in UnitFrame.
fn add_visual_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetVisuals", |_, _this, _info: Value| Ok(()));
}
