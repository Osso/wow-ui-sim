//! Texture creation from XML definitions.

use crate::lua_api::WowLuaEnv;
use crate::xml::{collect_texture_mixins, register_texture_template};

use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_animation_group_code, generate_set_point_code, get_size_values, resolve_child_name};

/// Generate Lua code for texture source (file or atlas) and size.
fn generate_texture_source_code(texture: &crate::xml::TextureXml) -> String {
    let mut code = String::new();

    if let Some(file) = &texture.file {
        code.push_str(&format!(
            r#"
        tex:SetTexture("{}")
        "#,
            escape_lua_string(file)
        ));
    }

    if let Some(atlas) = &texture.atlas {
        let use_atlas_size = texture.use_atlas_size.unwrap_or(false);
        code.push_str(&format!(
            r#"
        tex:SetAtlas("{}", {})
        "#,
            escape_lua_string(atlas),
            use_atlas_size
        ));
    }

    if let Some(size) = &texture.size {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            code.push_str(&format!(
                r#"
        tex:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    code
}

/// Generate Lua code for texture visual properties (color, tiling, parentKey).
fn generate_texture_visual_code(texture: &crate::xml::TextureXml) -> String {
    let mut code = String::new();

    if let Some(color) = &texture.color {
        code.push_str(&format!(
            r#"
        tex:SetVertexColor({}, {}, {}, {})
        "#,
            color.r.unwrap_or(1.0),
            color.g.unwrap_or(1.0),
            color.b.unwrap_or(1.0),
            color.a.unwrap_or(1.0)
        ));
    }

    if texture.horiz_tile == Some(true) {
        code.push_str("\n        tex:SetHorizTile(true)\n        ");
    }

    if texture.vert_tile == Some(true) {
        code.push_str("\n        tex:SetVertTile(true)\n        ");
    }

    if texture.set_all_points == Some(true) {
        code.push_str("\n        tex:SetAllPoints(true)\n        ");
    }

    if let Some(key) = &texture.parent_key {
        code.push_str(&format!(
            r#"
        parent.{} = tex
        "#,
            key
        ));
    }

    if let Some(parent_array) = &texture.parent_array {
        code.push_str(&format!(
            r#"
        parent.{parent_array} = parent.{parent_array} or {{}}
        table.insert(parent.{parent_array}, tex)
        "#,
        ));
    }

    code
}

/// Generate Lua Mixin() calls for texture mixins (from inherits and direct mixin attr).
fn generate_mixin_code(texture: &crate::xml::TextureXml) -> String {
    let mixins = collect_texture_mixins(texture);
    if mixins.is_empty() {
        return String::new();
    }
    let mut code = String::new();
    for m in &mixins {
        code.push_str(&format!(
            "\n        if {} then Mixin(tex, {}) end\n        ",
            m, m
        ));
    }
    code
}

/// Create a texture from XML definition.
pub fn create_texture_from_xml(
    env: &WowLuaEnv,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
) -> Result<(), LoadError> {
    if texture.is_virtual == Some(true) {
        // Register virtual textures as templates for mixin resolution
        if let Some(ref name) = texture.name {
            register_texture_template(name, texture.clone());
        }
        return Ok(());
    }

    let tex_name = resolve_child_name(texture.name.as_deref(), parent_name, "__tex_");

    let mut lua_code = format!(
        r#"
        local parent = {}
        local tex = parent:CreateTexture("{}", "{}")
        "#,
        parent_name, tex_name, draw_layer
    );

    // Apply mixins from inherited templates and direct mixin attribute
    lua_code.push_str(&generate_mixin_code(texture));

    lua_code.push_str(&generate_texture_source_code(texture));
    lua_code.push_str(&generate_texture_visual_code(texture));

    if let Some(anchors) = &texture.anchors {
        lua_code.push_str(&generate_set_point_code(
            anchors,
            "tex",
            "parent",
            parent_name,
            "parent",
        ));
    } else if texture.set_all_points != Some(true) {
        // WoW implicitly applies SetAllPoints to textures with no anchors
        lua_code.push_str("\n        tex:SetAllPoints(true)\n        ");
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create texture {} on {}: {}",
            tex_name, parent_name, e
        ))
    })?;

    // Process animation groups on the texture (e.g. AutoCastOverlayTemplate Shine rotation)
    if let Some(anims) = &texture.animations {
        let mut anim_code = format!("local frame = {}\n", tex_name);
        for anim_group_xml in &anims.animations {
            if anim_group_xml.is_virtual == Some(true) {
                continue;
            }
            anim_code.push_str(&generate_animation_group_code(anim_group_xml, "frame"));
        }
        env.exec(&anim_code).ok();
    }

    Ok(())
}
