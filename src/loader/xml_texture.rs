//! Texture creation from XML definitions.

use crate::lua_api::WowLuaEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_set_point_code, get_size_values, resolve_child_name};

/// Generate Lua code for texture-specific properties (file, atlas, color, tiling, etc).
fn generate_texture_properties_code(texture: &crate::xml::TextureXml) -> String {
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
        code.push_str(
            r#"
        tex:SetHorizTile(true)
        "#,
        );
    }

    if texture.vert_tile == Some(true) {
        code.push_str(
            r#"
        tex:SetVertTile(true)
        "#,
        );
    }

    if texture.set_all_points == Some(true) {
        code.push_str(
            r#"
        tex:SetAllPoints(true)
        "#,
        );
    }

    if let Some(key) = &texture.parent_key {
        code.push_str(&format!(
            r#"
        parent.{} = tex
        "#,
            key
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

    lua_code.push_str(&generate_texture_properties_code(texture));

    if let Some(anchors) = &texture.anchors {
        lua_code.push_str(&generate_set_point_code(
            anchors,
            "tex",
            "parent",
            parent_name,
            "parent",
        ));
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create texture {} on {}: {}",
            tex_name, parent_name, e
        ))
    })
}
