//! Extra frame helpers: animations, bar textures, action bar init.

use crate::lua_api::LoaderEnv;

use super::error::LoadError;
use super::helpers::{
    escape_lua_string, generate_set_point_code, lua_frame_ref_by_id, lua_global_ref,
    lua_table_field_ref, rand_id,
};
use super::helpers_anim::generate_animation_group_code;

/// Apply animation groups from the frame and its inherited templates.
pub(crate) fn apply_animation_groups(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    frame_id: u64,
    inherits: &str,
) -> Result<(), LoadError> {
    if let Some(anims) = frame.animations() {
        exec_animation_groups(env, anims, frame_id);
    }
    if !inherits.is_empty() {
        for template_entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(anims) = template_entry.frame.animations() {
                exec_animation_groups(env, anims, frame_id);
            }
        }
    }
    Ok(())
}

/// Generate and execute Lua code for a set of animation groups on a frame.
fn exec_animation_groups(env: &LoaderEnv<'_>, anims: &crate::xml::AnimationsXml, frame_id: u64) {
    let mut anim_code = format!(
        r#"
            local frame = {}
            "#,
        lua_frame_ref_by_id(frame_id)
    );
    for anim_group_xml in &anims.animations {
        if anim_group_xml.is_virtual == Some(true) {
            if let Some(ref name) = anim_group_xml.name {
                crate::xml::register_anim_group_template(name, anim_group_xml.clone());
            }
            continue;
        }
        anim_code.push_str(&generate_animation_group_code(anim_group_xml, "frame"));
    }
    if let Err(e) = env.exec(&anim_code) {
        eprintln!("[AnimSetup] error: {}", e);
    }
}

/// Create the bar texture for a StatusBar from its inline `<BarTexture>` XML element.
pub(crate) fn apply_bar_texture(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    let Some(bar) = resolve_bar_texture(frame, inherits) else {
        return Ok(());
    };
    let bar_name = resolved_texture_name(&bar, name, "__bar_");

    let parent_ref = lua_global_ref(name);
    let mut code = build_bar_texture_header(&parent_ref, &bar_name);
    append_bar_texture_properties(&mut code, &bar);
    code.push_str("            parent:SetStatusBarTexture(bar)\n");
    let parent_key = bar.parent_key.as_deref().unwrap_or("Bar");
    code.push_str(&format!("            parent.{} = bar\n", parent_key));
    if bar.name.is_some() {
        code.push_str(&format!(
            "            _G[\"{}\"] = bar\n",
            escape_lua_string(&bar_name)
        ));
    }
    code.push_str("        end\n");
    env.exec(&code)
        .map_err(|e| LoadError::Lua(format!("Failed to create bar texture on {}: {}", name, e)))?;
    Ok(())
}

/// Create and apply the thumb texture declared by a Slider's `<ThumbTexture>`.
pub(crate) fn apply_thumb_texture(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    let Some(thumb) = resolve_thumb_texture(frame, inherits) else {
        return Ok(());
    };
    let resolved = crate::xml::resolve_texture_inheritance(&thumb);
    let thumb_name = resolved_texture_name(&resolved, name, "__thumb_");
    let parent_ref = lua_global_ref(name);
    let mut code = build_thumb_texture_header(&parent_ref, &thumb_name);
    append_texture_common_properties(&mut code, &resolved, name);
    append_thumb_texture_binding(&mut code, &resolved, &thumb_name);
    code.push_str("        end\n");
    env.exec(&code).map_err(|e| {
        LoadError::Lua(format!("Failed to create thumb texture on {}: {}", name, e))
    })?;
    Ok(())
}

fn resolve_bar_texture(
    frame: &crate::xml::FrameXml,
    inherits: &str,
) -> Option<crate::xml::TextureXml> {
    if let Some(bar) = frame.bar_texture() {
        return Some(bar.clone());
    }

    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| entry.frame.bar_texture().cloned())
}

fn resolve_thumb_texture(
    frame: &crate::xml::FrameXml,
    inherits: &str,
) -> Option<crate::xml::TextureXml> {
    if let Some(thumb) = frame.thumb_texture() {
        return Some(thumb.clone());
    }

    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| entry.frame.thumb_texture().cloned())
}

fn resolved_texture_name(
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    anonymous_prefix: &str,
) -> String {
    texture
        .name
        .as_ref()
        .map(|name| name.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("{}{}", anonymous_prefix, rand_id()))
}

fn build_bar_texture_header(parent_ref: &str, bar_name: &str) -> String {
    format!(
        r#"
        local parent = {parent_ref}
        if parent and parent.SetStatusBarTexture then
            local bar = parent:CreateTexture("{}", "ARTWORK")
        "#,
        escape_lua_string(bar_name),
    )
}

fn build_thumb_texture_header(parent_ref: &str, thumb_name: &str) -> String {
    format!(
        r#"
        local parent = {parent_ref}
        if parent then
            local thumb = parent:GetThumbTexture()
            if not thumb then
                thumb = parent:CreateTexture("{}", "ARTWORK")
                parent:SetThumbTexture(thumb)
            end
        "#,
        escape_lua_string(thumb_name),
    )
}

fn append_bar_texture_properties(code: &mut String, bar: &crate::xml::TextureXml) {
    append_texture_source_properties(code, "bar", bar);
    append_texture_color_property(code, "bar", bar);
}

fn append_texture_common_properties(
    code: &mut String,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
) {
    append_texture_source_properties(code, "thumb", texture);
    append_texture_size_property(code, "thumb", texture);
    append_texture_anchor_property(code, "thumb", texture, parent_name);
    append_texture_tex_coords_property(code, "thumb", texture);
    append_texture_color_property(code, "thumb", texture);
    if texture.hidden == Some(true) {
        code.push_str("            thumb:Hide()\n");
    }
}

fn append_thumb_texture_binding(
    code: &mut String,
    texture: &crate::xml::TextureXml,
    thumb_name: &str,
) {
    if texture.name.is_some() {
        let escaped_thumb_name = escape_lua_string(thumb_name);
        code.push_str(&format!(
            "            _G[\"{}\"] = thumb\n",
            escaped_thumb_name
        ));
        code.push_str(&format!(
            "            parent:SetThumbTexture(_G[\"{}\"])\n",
            escaped_thumb_name
        ));
    } else {
        code.push_str("            parent:SetThumbTexture(thumb)\n");
    }
    code.push_str("            parent.ThumbTexture = thumb\n");
    if let Some(parent_key) = texture.parent_key.as_deref() {
        let parent_field = lua_table_field_ref("parent", parent_key);
        code.push_str(&format!("            {parent_field} = thumb\n"));
    }
}

fn append_texture_source_properties(
    code: &mut String,
    var_name: &str,
    texture: &crate::xml::TextureXml,
) {
    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            {var_name}:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &texture.atlas {
        code.push_str(&format!(
            "            {var_name}:SetAtlas(\"{}\")\n",
            escape_lua_string(atlas)
        ));
    }
}

fn append_texture_color_property(
    code: &mut String,
    var_name: &str,
    texture: &crate::xml::TextureXml,
) {
    if let Some(color) = &texture.color {
        code.push_str(&format!(
            "            {var_name}:SetColorTexture({}, {}, {}, {})\n",
            color.r.unwrap_or(1.0),
            color.g.unwrap_or(1.0),
            color.b.unwrap_or(1.0),
            color.a.unwrap_or(1.0)
        ));
    }
}

fn append_texture_size_property(
    code: &mut String,
    var_name: &str,
    texture: &crate::xml::TextureXml,
) {
    let Some(size) = &texture.size else {
        return;
    };
    let width = size
        .x
        .or_else(|| size.abs_dimension.as_ref().and_then(|dim| dim.x));
    let height = size
        .y
        .or_else(|| size.abs_dimension.as_ref().and_then(|dim| dim.y));
    match (width, height) {
        (Some(width), Some(height)) => code.push_str(&format!(
            "            {var_name}:SetSize({width}, {height})\n"
        )),
        (Some(width), None) => {
            code.push_str(&format!("            {var_name}:SetWidth({width})\n"))
        }
        (None, Some(height)) => {
            code.push_str(&format!("            {var_name}:SetHeight({height})\n"))
        }
        (None, None) => {}
    }
}

fn append_texture_anchor_property(
    code: &mut String,
    var_name: &str,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
) {
    if let Some(anchors) = &texture.anchors {
        code.push_str(&format!("            {var_name}:ClearAllPoints()\n"));
        code.push_str(&generate_set_point_code(
            anchors,
            var_name,
            "parent",
            parent_name,
            "parent",
        ));
    } else if texture.set_all_points == Some(true) {
        code.push_str(&format!("            {var_name}:SetAllPoints(true)\n"));
    }
}

fn append_texture_tex_coords_property(
    code: &mut String,
    var_name: &str,
    texture: &crate::xml::TextureXml,
) {
    let Some(coords) = &texture.tex_coords else {
        return;
    };
    if let (Some(left), Some(right), Some(top), Some(bottom)) =
        (coords.left, coords.right, coords.top, coords.bottom)
    {
        code.push_str(&format!(
            "            {var_name}:SetTexCoord({left}, {right}, {top}, {bottom})\n"
        ));
    }
}

/// Initialize tables expected by action bar OnLoad handlers.
/// Only runs Lua when the frame has a `numButtons` KeyValue (rare).
pub(crate) fn init_action_bar_tables(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    name: &str,
) {
    let has_num_buttons = frame
        .all_key_values()
        .any(|kv| kv.values.iter().any(|v| v.key == "numButtons"));
    if !has_num_buttons {
        return;
    }
    let code = format!(
        r#"do local f = {}
        if f and not f.actionButtons then
            f.actionButtons = {{}}
        end end"#,
        lua_global_ref(name)
    );
    let _ = env.exec(&code);
}
