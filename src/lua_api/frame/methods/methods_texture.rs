//! Texture-related methods: SetTexture, SetAtlas, SetTexCoord, etc.

use super::FrameHandle;
use crate::widget::WidgetType;
use mlua::{UserDataMethods, Value};

/// Add texture-related methods to FrameHandle UserData.
pub fn add_texture_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetTexture(path) - for Texture widgets
    methods.add_method("SetTexture", |_, this, path: Option<String>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.texture = path;
        }
        Ok(())
    });

    // GetTexture() - for Texture widgets
    methods.add_method("GetTexture", |_, this, ()| {
        let state = this.state.borrow();
        let texture = state
            .widgets
            .get(this.id)
            .and_then(|f| f.texture.clone());
        Ok(texture)
    });

    // SetHorizTile(tile) - Enable/disable horizontal tiling
    methods.add_method("SetHorizTile", |_, this, tile: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.horiz_tile = tile;
        }
        Ok(())
    });

    // GetHorizTile() - Check if horizontal tiling is enabled
    methods.add_method("GetHorizTile", |_, this, ()| {
        let state = this.state.borrow();
        let tile = state
            .widgets
            .get(this.id)
            .map(|f| f.horiz_tile)
            .unwrap_or(false);
        Ok(tile)
    });

    // SetVertTile(tile) - Enable/disable vertical tiling
    methods.add_method("SetVertTile", |_, this, tile: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.vert_tile = tile;
        }
        Ok(())
    });

    // GetVertTile() - Check if vertical tiling is enabled
    methods.add_method("GetVertTile", |_, this, ()| {
        let state = this.state.borrow();
        let tile = state
            .widgets
            .get(this.id)
            .map(|f| f.vert_tile)
            .unwrap_or(false);
        Ok(tile)
    });

    // SetBlendMode(blendMode) - Set texture blend mode (ADD, ALPHAKEY, BLEND, DISABLE, MOD)
    methods.add_method("SetBlendMode", |_, _this, _mode: Option<String>| {
        // Stub - blend mode is a rendering hint
        Ok(())
    });

    // GetBlendMode() - Get texture blend mode
    methods.add_method("GetBlendMode", |_, _this, ()| {
        Ok("BLEND") // Default blend mode
    });

    // SetDesaturated(desaturation) - Set texture desaturation
    methods.add_method("SetDesaturated", |_, _this, _desaturated: bool| {
        // Stub - desaturation is a rendering effect
        Ok(())
    });

    // IsDesaturated() - Check if texture is desaturated
    methods.add_method("IsDesaturated", |_, _this, ()| Ok(false));

    // SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords) - Set texture from atlas
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
            // Look up atlas info
            if let Some(lookup) = crate::atlas::get_atlas_info(&name) {
                let atlas_info = lookup.info;
                let mut state = this.state.borrow_mut();

                // Get parent info - find which children_key this frame is registered as
                let parent_info: Option<(u64, Option<String>)> =
                    state.widgets.get(this.id).and_then(|f| {
                        f.parent_id.and_then(|pid| {
                            state.widgets.get(pid).map(|parent| {
                                // Find which key in parent's children_keys maps to this.id
                                let key = parent
                                    .children_keys
                                    .iter()
                                    .find(|(_, child_id)| **child_id == this.id)
                                    .map(|(k, _)| k.clone());
                                (pid, key)
                            })
                        })
                    });

                if let Some(frame) = state.widgets.get_mut(this.id) {
                    // Set texture file from atlas
                    frame.texture = Some(atlas_info.file.to_string());
                    // Set atlas base texture coordinates (the sub-region on the file)
                    let atlas_uvs = (
                        atlas_info.left_tex_coord,
                        atlas_info.right_tex_coord,
                        atlas_info.top_tex_coord,
                        atlas_info.bottom_tex_coord,
                    );
                    frame.atlas_tex_coords = Some(atlas_uvs);
                    // Set rendering tex_coords to the full atlas sub-region
                    frame.tex_coords = Some(atlas_uvs);
                    // Set tiling flags
                    frame.horiz_tile = atlas_info.tiles_horizontally;
                    frame.vert_tile = atlas_info.tiles_vertically;
                    // Store atlas name
                    frame.atlas = Some(name.clone());
                    // Optionally set size from atlas (use logical dimensions)
                    if use_atlas_size {
                        frame.width = lookup.width() as f32;
                        frame.height = lookup.height() as f32;
                    }
                }

                // If this texture is a child of a button (NormalTexture, PushedTexture, etc.),
                // also update the parent button's texture field and tex_coords so rendering picks it up
                if let Some((parent_id, parent_key_opt)) = parent_info {
                    if let Some(parent) = state.widgets.get_mut(parent_id as u64) {
                        if matches!(
                            parent.widget_type,
                            WidgetType::Button | WidgetType::CheckButton
                        ) {
                            let texture_path = atlas_info.file.to_string();
                            let tex_coords = (
                                atlas_info.left_tex_coord,
                                atlas_info.right_tex_coord,
                                atlas_info.top_tex_coord,
                                atlas_info.bottom_tex_coord,
                            );
                            if let Some(parent_key) = parent_key_opt {
                                match parent_key.as_str() {
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
                        }
                    }
                }
            } else {
                // Unknown atlas - just store the name
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.atlas = Some(name);
                }
            }
        }
        Ok(())
    });

    // GetAtlas() - Get current atlas name
    // NOTE: Mixins (e.g., MinimalScrollBarStepperScriptsMixin) can override GetAtlas
    // with a Lua function. Since mlua's add_method takes priority over __index,
    // we check for Lua overrides in __frame_fields before using the default.
    methods.add_method("GetAtlas", |lua, this, ()| {
        // Check for Lua override (from Mixin)
        if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
            if let Ok(frame_fields) = fields_table.get::<mlua::Table>(this.id) {
                if let Ok(Value::Function(f)) = frame_fields.get::<Value>("GetAtlas") {
                    // Create userdata for self parameter
                    let ud = lua.create_userdata(FrameHandle {
                        id: this.id,
                        state: std::rc::Rc::clone(&this.state),
                    })?;
                    return f.call::<Value>(ud);
                }
            }
        }
        // Default: return atlas from Rust widget state
        let state = this.state.borrow();
        let atlas = state.widgets.get(this.id).and_then(|f| f.atlas.clone());
        match atlas {
            Some(name) => Ok(Value::String(lua.create_string(&name)?)),
            None => Ok(Value::Nil),
        }
    });

    // SetSnapToPixelGrid(snap) - Set whether texture snaps to pixel grid
    methods.add_method("SetSnapToPixelGrid", |_, _this, _snap: bool| {
        // No-op for now, just store the state if needed
        Ok(())
    });

    // IsSnappingToPixelGrid() - Get whether texture snaps to pixel grid
    methods.add_method("IsSnappingToPixelGrid", |_, _this, ()| Ok(false));

    // SetTexelSnappingBias(bias) - Set texel snapping bias for pixel-perfect rendering
    methods.add_method("SetTexelSnappingBias", |_, _this, _bias: f32| {
        // No-op - this controls sub-pixel texture positioning
        Ok(())
    });

    // GetTexelSnappingBias() - Get texel snapping bias
    methods.add_method("GetTexelSnappingBias", |_, _this, ()| Ok(0.0_f32));

    // SetTextureSliceMargins(left, right, top, bottom) - Set 9-slice margins
    methods.add_method(
        "SetTextureSliceMargins",
        |_, _this, (_left, _right, _top, _bottom): (f32, f32, f32, f32)| Ok(()),
    );

    // GetTextureSliceMargins() - Get 9-slice margins
    methods.add_method("GetTextureSliceMargins", |_, _this, ()| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });

    // SetTextureSliceMode(mode) - Set 9-slice mode
    methods.add_method("SetTextureSliceMode", |_, _this, _mode: i32| Ok(()));

    // GetTextureSliceMode() - Get 9-slice mode
    methods.add_method("GetTextureSliceMode", |_, _this, ()| Ok(0i32));

    // ClearTextureSlice() - Clear 9-slice configuration
    methods.add_method("ClearTextureSlice", |_, _this, ()| Ok(()));

    // SetVertexColor(r, g, b, a) - for Texture widgets
    methods.add_method(
        "SetVertexColor",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.vertex_color = Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
            }
            Ok(())
        },
    );

    // GetVertexColor() - get vertex color for Texture widgets
    methods.add_method("GetVertexColor", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(color) = &frame.vertex_color {
                return Ok((color.r, color.g, color.b, color.a));
            }
        }
        Ok((1.0f32, 1.0f32, 1.0f32, 1.0f32)) // Default white
    });

    // SetCenterColor(r, g, b, a) - for NineSlice frames (sets center fill color)
    methods.add_method("SetCenterColor", |_, _this, _args: mlua::MultiValue| {
        // NineSlice center color - just stub for now
        Ok(())
    });

    // SetTexCoord(left, right, top, bottom) - for Texture widgets
    // When atlas is active, coords are relative to the atlas sub-region (0-1 maps to atlas bounds)
    // Can also be called with 8 values for corner-based coords
    methods.add_method("SetTexCoord", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        if args_vec.len() >= 4 {
            let left = match &args_vec[0] {
                Value::Number(n) => *n as f32,
                Value::Integer(n) => *n as f32,
                _ => 0.0,
            };
            let right = match &args_vec[1] {
                Value::Number(n) => *n as f32,
                Value::Integer(n) => *n as f32,
                _ => 1.0,
            };
            let top = match &args_vec[2] {
                Value::Number(n) => *n as f32,
                Value::Integer(n) => *n as f32,
                _ => 0.0,
            };
            let bottom = match &args_vec[3] {
                Value::Number(n) => *n as f32,
                Value::Integer(n) => *n as f32,
                _ => 1.0,
            };

            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                // When atlas is active, remap coords relative to the atlas sub-region
                if let Some((al, ar, at, ab)) = frame.atlas_tex_coords {
                    let aw = ar - al;
                    let ah = ab - at;
                    frame.tex_coords = Some((
                        al + left * aw,
                        al + right * aw,
                        at + top * ah,
                        at + bottom * ah,
                    ));
                } else {
                    frame.tex_coords = Some((left, right, top, bottom));
                }
            }
        }
        Ok(())
    });

    // AddMaskTexture(mask) - add a mask texture to this texture
    methods.add_method("AddMaskTexture", |_, _this, _mask: Value| {
        // Mask textures control alpha blending on parent texture
        Ok(())
    });

    // RemoveMaskTexture(mask) - remove a mask texture from this texture
    methods.add_method("RemoveMaskTexture", |_, _this, _mask: Value| Ok(()));

    // GetNumMaskTextures() - get number of mask textures
    methods.add_method("GetNumMaskTextures", |_, _this, ()| Ok(0));

    // GetMaskTexture(index) - get mask texture by index
    methods.add_method("GetMaskTexture", |_, _this, _index: i32| Ok(Value::Nil));

    // SetColorTexture(r, g, b, a) - for Texture widgets
    methods.add_method(
        "SetColorTexture",
        |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.color_texture = Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
                // Clear file texture when setting color texture
                frame.texture = None;
            }
            Ok(())
        },
    );

    // SetGradient(orientation, minColor, maxColor) - set gradient on texture
    methods.add_method("SetGradient", |_, _this, _args: mlua::MultiValue| Ok(()));

    // SetDrawLayer(layer, sublayer) - set draw layer for texture/fontstring
    methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| Ok(()));

    // GetDrawLayer() - get draw layer for texture/fontstring
    methods.add_method("GetDrawLayer", |_, _this, ()| {
        // Returns: layer, sublayer
        Ok(("ARTWORK", 0i32))
    });
}
