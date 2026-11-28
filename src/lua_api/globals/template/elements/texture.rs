mod codegen;

use crate::loader::chunk_cache;
use crate::lua_api::SimState;
use crate::lua_api::frame::methods::{apply_atlas_to_frame, register_child_widget};
use crate::lua_api::frame::{frame_ref, sync_child_to_lua};
use crate::render::BlendMode;
use crate::widget::{Color, Frame, Gradient, WidgetType};
use mlua::Lua;
use std::cell::RefCell;
use std::rc::Rc;

use super::super::{get_size_values, rand_id};
pub(crate) use codegen::{
    append_anchors_and_parent_refs, append_key_values, append_texture_properties,
    apply_deferred_mask_atlases, apply_texture_animations, build_template_texture_lua,
};

pub(crate) struct AnchorParentContext<'a> {
    pub parent_key: &'a Option<String>,
    pub parent_array: &'a Option<String>,
    pub var: &'a str,
    pub parent_var: &'a str,
    pub parent_name: &'a str,
}

pub(crate) struct DirectTextureCreateContext<'a> {
    pub texture: &'a crate::xml::TextureXml,
    pub parent_name: &'a str,
    pub subst_parent: &'a str,
    pub draw_layer: &'a str,
    pub is_mask: bool,
    pub is_line: bool,
}

struct DirectTextureFinalizeContext<'a> {
    texture: &'a crate::xml::TextureXml,
    parent_id: u64,
    child_id: u64,
    parent_name: &'a str,
    is_mask: bool,
    is_line: bool,
}

/// Create a texture from template XML.
///
/// `parent_name` is the actual Lua frame name (for parent reference).
/// `subst_parent` is the name used for `$parent` substitution in child names
/// (propagated through anonymous frames to the nearest named ancestor).
pub(crate) fn create_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    subst_parent: &str,
    draw_layer: &str,
    is_mask: bool,
    is_line: bool,
) {
    let resolved = crate::xml::resolve_texture_inheritance(texture);
    let child_name = resolved
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    #[cfg(test)]
    super::super::test_counters::record_texture_create();

    let code = build_template_texture_lua(
        &resolved,
        parent_name,
        &child_name,
        draw_layer,
        is_mask,
        is_line,
    );
    if let Err(e) = chunk_cache::exec(lua, &code, "template-elements") {
        eprintln!(
            "[create_texture] failed for '{}' on '{}': {}",
            child_name, parent_name, e
        );
    }
    apply_texture_animations(lua, &resolved, &child_name);
}

pub(crate) fn create_texture_from_template_direct(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    ctx: DirectTextureCreateContext<'_>,
) -> mlua::Result<()> {
    let resolved = crate::xml::resolve_texture_inheritance(ctx.texture);
    let finalize_ctx = prepare_direct_texture_child(lua, state, &resolved, &ctx)?;
    finalize_direct_texture_child(lua, state, finalize_ctx)
}

fn prepare_direct_texture_child<'a>(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    texture: &'a crate::xml::TextureXml,
    ctx: &DirectTextureCreateContext<'a>,
) -> mlua::Result<DirectTextureFinalizeContext<'a>> {
    ensure_direct_texture_supported(texture)?;
    let parent_id = direct_texture_parent_id(state, ctx.parent_name)?;
    let child_name = direct_texture_child_name(texture, ctx.subst_parent);
    let child_id = register_direct_texture_child(
        lua,
        texture,
        parent_id,
        &child_name,
        ctx.draw_layer,
        ctx.is_mask,
        ctx.is_line,
    )?;
    Ok(DirectTextureFinalizeContext {
        texture,
        parent_id,
        child_id,
        parent_name: ctx.parent_name,
        is_mask: ctx.is_mask,
        is_line: ctx.is_line,
    })
}

fn direct_texture_parent_id(state: &Rc<RefCell<SimState>>, parent_name: &str) -> mlua::Result<u64> {
    resolve_state_frame_id(state, parent_name).ok_or_else(|| {
        mlua::Error::runtime(format!(
            "missing parent '{}' for direct texture create",
            parent_name
        ))
    })
}

fn direct_texture_child_name(texture: &crate::xml::TextureXml, subst_parent: &str) -> String {
    texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()))
}

fn register_direct_texture_child(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_id: u64,
    child_name: &str,
    draw_layer: &str,
    is_mask: bool,
    is_line: bool,
) -> mlua::Result<u64> {
    let widget_type = if is_line {
        WidgetType::Line
    } else {
        WidgetType::Texture
    };
    let mut child = Frame::new(widget_type, Some(child_name.to_string()), Some(parent_id));
    if let Some(layer) = crate::widget::DrawLayer::from_str(draw_layer) {
        child.draw_layer = layer;
    }
    if let Some(parent_key) = texture.parent_key.as_ref() {
        child.parent_key = Some(parent_key.clone());
    }
    if is_mask {
        child.is_mask = true;
        child.object_type_name = Some("MaskTexture".to_string());
    }
    let child_id = child.id;
    register_child_widget(lua, parent_id, child, &Some(child_name.to_string()))?;
    Ok(child_id)
}

fn finalize_direct_texture_child(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    ctx: DirectTextureFinalizeContext<'_>,
) -> mlua::Result<()> {
    sync_region_parent_refs(
        lua,
        state,
        ctx.parent_id,
        ctx.child_id,
        ctx.texture.parent_key.as_deref(),
        ctx.texture.parent_array.as_deref(),
    )?;
    apply_texture_visuals_direct(state, ctx.child_id, ctx.texture, ctx.is_mask, ctx.is_line);
    apply_region_layout(
        state,
        ctx.child_id,
        &ctx.texture.anchors,
        ctx.texture.set_all_points,
        ctx.parent_name,
        ctx.texture.anchors.is_none() && ctx.texture.set_all_points != Some(true),
    );
    apply_region_visibility(state, ctx.child_id, ctx.texture.hidden, ctx.texture.alpha);
    apply_mask_wiring_direct(state, ctx.parent_id, ctx.child_id, ctx.texture)?;
    Ok(())
}

pub(crate) fn resolve_state_frame_id(
    state: &Rc<RefCell<SimState>>,
    frame_name: &str,
) -> Option<u64> {
    state
        .borrow()
        .widgets
        .get_id_by_name(frame_name)
        .or_else(|| {
            frame_name
                .strip_prefix("__frame_")
                .and_then(|suffix| suffix.parse::<u64>().ok())
        })
}

pub(crate) fn sync_region_parent_refs(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    parent_id: u64,
    child_id: u64,
    parent_key: Option<&str>,
    parent_array: Option<&str>,
) -> mlua::Result<()> {
    if let Some(parent_key) = parent_key {
        {
            let mut sim = state.borrow_mut();
            if let Some(parent) = sim.widgets.get_mut_visual(parent_id) {
                parent
                    .children_keys
                    .insert(parent_key.to_string(), child_id);
            }
        }
        sync_child_to_lua(lua, parent_id, parent_key, child_id)?;
    }

    let Some(parent_array) = parent_array else {
        return Ok(());
    };
    let parent_val = frame_ref(lua, parent_id)?;
    let child_val = frame_ref(lua, child_id)?;
    let mlua::Value::UserData(parent_ud) = parent_val else {
        return Ok(());
    };
    let fields: mlua::Table = parent_ud.user_value()?;
    let array = match fields.raw_get::<mlua::Value>(parent_array)? {
        mlua::Value::Table(existing) => existing,
        _ => {
            let created = lua.create_table()?;
            fields.raw_set(parent_array, created.clone())?;
            created
        }
    };
    let next_index = array.raw_len() + 1;
    array.raw_set(next_index, child_val)?;
    Ok(())
}

pub(crate) fn apply_region_layout(
    state: &Rc<RefCell<SimState>>,
    region_id: u64,
    anchors: &Option<crate::xml::AnchorsXml>,
    set_all_points: Option<bool>,
    parent_name: &str,
    default_fill_parent: bool,
) {
    if let Some(frame) = state.borrow_mut().widgets.get_mut_visual(region_id) {
        frame.clear_all_points();
    }

    if let Some(anchors) = anchors {
        let mut sim = state.borrow_mut();
        for anchor in &anchors.anchors {
            super::super::direct::set_single_anchor(&mut sim, region_id, anchor, parent_name);
        }
    }
    if set_all_points == Some(true) || default_fill_parent {
        set_region_all_points(state, region_id);
    }
}

pub(crate) fn apply_region_visibility(
    state: &Rc<RefCell<SimState>>,
    region_id: u64,
    hidden: Option<bool>,
    alpha: Option<f32>,
) {
    if hidden == Some(true) {
        state.borrow_mut().set_frame_visible(region_id, false);
    }
    if let Some(alpha) = alpha {
        super::super::direct::set_alpha(state, region_id, alpha);
    }
}

pub(crate) fn apply_texture_visuals_direct(
    state: &Rc<RefCell<SimState>>,
    texture_id: u64,
    texture: &crate::xml::TextureXml,
    is_mask: bool,
    is_line: bool,
) {
    {
        let mut sim = state.borrow_mut();
        if let Some(frame) = sim.widgets.get_mut_visual(texture_id) {
            apply_texture_size_direct(frame, texture);
            if is_line && let Some(thickness) = texture.thickness {
                frame.line_thickness = thickness;
            }
            apply_texture_source_direct(frame, texture, is_mask);
            apply_texture_color_direct(frame, texture);
            apply_texture_gradient_direct(frame, texture.gradient.as_ref());
            if texture.horiz_tile == Some(true) {
                frame.horiz_tile = true;
            }
            if texture.vert_tile == Some(true) {
                frame.vert_tile = true;
            }
            if let Some(mode) = texture.effective_blend_mode() {
                frame.alpha_mode = Some(mode.to_string());
                frame.blend_mode = BlendMode::from_token(&mode.to_ascii_uppercase());
            }
        }
    }

    if let Some(atlas_name) = texture.atlas.as_deref()
        && let Some(lookup) = crate::atlas::get_render_atlas_info(atlas_name)
    {
        let mut sim = state.borrow_mut();
        apply_atlas_to_frame(
            &mut sim.widgets,
            texture_id,
            lookup.info,
            atlas_name,
            &lookup,
            texture.use_atlas_size.unwrap_or(is_mask),
        );
    }
}

fn ensure_direct_texture_supported(texture: &crate::xml::TextureXml) -> mlua::Result<()> {
    if !crate::xml::collect_texture_mixins(texture).is_empty() {
        return Err(mlua::Error::runtime(
            "direct texture create does not support texture mixins yet",
        ));
    }
    if texture.animations.is_some() {
        return Err(mlua::Error::runtime(
            "direct texture create does not support texture animations yet",
        ));
    }
    if texture
        .key_values
        .as_ref()
        .is_some_and(|values| !values.values.is_empty())
    {
        return Err(mlua::Error::runtime(
            "direct texture create does not support texture key values yet",
        ));
    }
    if texture
        .color
        .as_ref()
        .is_some_and(|color| color.color.is_some())
    {
        return Err(mlua::Error::runtime(
            "direct texture create does not support named color refs yet",
        ));
    }
    Ok(())
}

fn apply_texture_size_direct(frame: &mut Frame, texture: &crate::xml::TextureXml) {
    let Some(size) = texture.size.as_ref() else {
        return;
    };
    let (width, height) = get_size_values(size);
    match (width, height) {
        (Some(width), Some(height)) => frame.set_size(width, height),
        (Some(width), None) => frame.width = width,
        (None, Some(height)) => frame.height = height,
        (None, None) => {}
    }
}

fn apply_texture_source_direct(frame: &mut Frame, texture: &crate::xml::TextureXml, is_mask: bool) {
    if let Some(file) = texture.file.as_ref() {
        frame.texture = Some(file.clone());
    }
    if let Some(atlas_name) = texture.atlas.as_ref()
        && crate::atlas::get_render_atlas_info(atlas_name).is_none()
    {
        frame.atlas = Some(atlas_name.clone());
        if texture.use_atlas_size.unwrap_or(is_mask) {
            frame.width = 0.0;
            frame.height = 0.0;
        }
    }
    if let Some(tc) = texture.tex_coords.as_ref() {
        frame.tex_coords = Some((
            tc.left.unwrap_or(0.0),
            tc.right.unwrap_or(1.0),
            tc.top.unwrap_or(0.0),
            tc.bottom.unwrap_or(1.0),
        ));
    }
}

fn apply_texture_color_direct(frame: &mut Frame, texture: &crate::xml::TextureXml) {
    let Some(color) = texture.color.as_ref() else {
        return;
    };
    let rgba = Color::new(
        color.r.unwrap_or(1.0),
        color.g.unwrap_or(1.0),
        color.b.unwrap_or(1.0),
        color.a.unwrap_or(1.0),
    );
    if texture.file.is_some() || texture.atlas.is_some() {
        frame.vertex_color = Some(rgba);
    } else {
        frame.color_texture = Some(rgba);
        frame.texture = None;
        frame.texture_file_data_id = None;
    }
}

fn apply_texture_gradient_direct(frame: &mut Frame, gradient: Option<&crate::xml::GradientXml>) {
    let Some(gradient) = gradient else {
        return;
    };
    let min = gradient.min_color.as_ref();
    let max = gradient.max_color.as_ref();
    frame.gradient = Some(Gradient {
        vertical: gradient
            .orientation
            .as_deref()
            .unwrap_or("VERTICAL")
            .eq_ignore_ascii_case("VERTICAL"),
        min_color: Color::new(
            min.and_then(|c| c.r).unwrap_or(0.0),
            min.and_then(|c| c.g).unwrap_or(0.0),
            min.and_then(|c| c.b).unwrap_or(0.0),
            min.and_then(|c| c.a).unwrap_or(1.0),
        ),
        max_color: Color::new(
            max.and_then(|c| c.r).unwrap_or(0.0),
            max.and_then(|c| c.g).unwrap_or(0.0),
            max.and_then(|c| c.b).unwrap_or(0.0),
            max.and_then(|c| c.a).unwrap_or(1.0),
        ),
    });
}

fn set_region_all_points(state: &Rc<RefCell<SimState>>, region_id: u64) {
    let parent_id = state
        .borrow()
        .widgets
        .get(region_id)
        .and_then(|frame| frame.parent_id);
    let mut sim = state.borrow_mut();
    if let Some(frame) = sim.widgets.get_mut_visual(region_id) {
        frame.clear_all_points();
        frame.set_point(
            crate::widget::AnchorPoint::TopLeft,
            parent_id.map(|id| id as usize),
            crate::widget::AnchorPoint::TopLeft,
            0.0,
            0.0,
        );
        frame.set_point(
            crate::widget::AnchorPoint::BottomRight,
            parent_id.map(|id| id as usize),
            crate::widget::AnchorPoint::BottomRight,
            0.0,
            0.0,
        );
    }
    sim.widgets.mark_rect_dirty(region_id);
}

fn apply_mask_wiring_direct(
    state: &Rc<RefCell<SimState>>,
    parent_id: u64,
    mask_id: u64,
    texture: &crate::xml::TextureXml,
) -> mlua::Result<()> {
    let Some(masked) = texture.masked_textures.as_ref() else {
        return Ok(());
    };
    let mut sim = state.borrow_mut();
    for entry in &masked.entries {
        let Some(key) = entry.child_key.as_deref() else {
            continue;
        };
        let target_id = resolve_mask_target_id(&sim, parent_id, key).ok_or_else(|| {
            mlua::Error::runtime(format!(
                "direct texture create could not resolve mask childKey '{}'",
                key
            ))
        })?;
        let Some(frame) = sim.widgets.get_mut_visual(target_id) else {
            continue;
        };
        ensure_mask_texture(frame, mask_id);
    }
    Ok(())
}

fn resolve_mask_target_id(state: &SimState, parent_id: u64, key: &str) -> Option<u64> {
    let mut current_id = parent_id;
    for segment in key.split('.') {
        let frame = state.widgets.get(current_id)?;
        current_id = *frame.children_keys.get(segment)?;
    }
    Some(current_id)
}

fn ensure_mask_texture(frame: &mut Frame, mask_id: u64) {
    frame
        .mask_textures
        .extend((!frame.mask_textures.contains(&mask_id)).then_some(mask_id));
}
