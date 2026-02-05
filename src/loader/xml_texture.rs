//! Texture creation from XML definitions.

use crate::lua_api::WowLuaEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, get_size_values, rand_id};

/// Create a texture from XML definition.
pub fn create_texture_from_xml(
    env: &WowLuaEnv,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
) -> Result<(), LoadError> {
    // Skip virtual textures
    if texture.is_virtual == Some(true) {
        return Ok(());
    }

    let tex_name = texture
        .name
        .clone()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let mut lua_code = format!(
        r#"
        local parent = {}
        local tex = parent:CreateTexture("{}", "{}")
        "#,
        parent_name, tex_name, draw_layer
    );

    // Set texture file
    if let Some(file) = &texture.file {
        lua_code.push_str(&format!(
            r#"
        tex:SetTexture("{}")
        "#,
            escape_lua_string(file)
        ));
    }

    // Set atlas
    if let Some(atlas) = &texture.atlas {
        let use_atlas_size = texture.use_atlas_size.unwrap_or(false);
        lua_code.push_str(&format!(
            r#"
        tex:SetAtlas("{}", {})
        "#,
            escape_lua_string(atlas),
            use_atlas_size
        ));
    }

    // Set size
    if let Some(size) = &texture.size {
        let (x, y) = get_size_values(size);
        if let (Some(x), Some(y)) = (x, y) {
            lua_code.push_str(&format!(
                r#"
        tex:SetSize({}, {})
        "#,
                x, y
            ));
        }
    }

    // Set anchors
    if let Some(anchors) = &texture.anchors {
        for anchor in &anchors.anchors {
            let point = &anchor.point;
            let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());

            let (x, y) = if let Some(offset) = &anchor.offset {
                if let Some(abs) = &offset.abs_dimension {
                    (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
                } else {
                    (0.0, 0.0)
                }
            } else {
                (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
            };

            // Handle relativeKey first, then relativeTo
            let rel = if let Some(key) = anchor.relative_key.as_deref() {
                // Handle $parent chains like "$parent.$parent.ScrollFrame"
                if key.contains("$parent") || key.contains("$Parent") {
                    let parts: Vec<&str> = key.split('.').collect();
                    let mut expr = String::new();
                    for part in parts {
                        if part == "$parent" || part == "$Parent" {
                            if expr.is_empty() {
                                expr = "parent".to_string();
                            } else {
                                expr = format!("{}:GetParent()", expr);
                            }
                        } else if !part.is_empty() {
                            expr = format!("{}[\"{}\"]", expr, part);
                        }
                    }
                    if expr.is_empty() { "parent".to_string() } else { expr }
                } else {
                    key.to_string()
                }
            } else {
                match anchor.relative_to.as_deref() {
                    Some("$parent") | None => "parent".to_string(),
                    Some(r) if r.contains("$parent") || r.contains("$Parent") => {
                        // Replace any $parent/$Parent pattern (e.g., $parent_Sibling, $parentColorSwatch)
                        r.replace("$parent", parent_name).replace("$Parent", parent_name)
                    }
                    Some(r) => r.to_string(),
                }
            };

            lua_code.push_str(&format!(
                r#"
        tex:SetPoint("{}", {}, "{}", {}, {})
        "#,
                point, rel, relative_point, x, y
            ));
        }
    }

    // Set color if specified
    if let Some(color) = &texture.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        lua_code.push_str(&format!(
            r#"
        tex:SetVertexColor({}, {}, {}, {})
        "#,
            r, g, b, a
        ));
    }

    // Set horizontal tiling
    if texture.horiz_tile == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetHorizTile(true)
        "#,
        );
    }

    // Set vertical tiling
    if texture.vert_tile == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetVertTile(true)
        "#,
        );
    }

    // Set all points if specified
    if texture.set_all_points == Some(true) {
        lua_code.push_str(
            r#"
        tex:SetAllPoints(true)
        "#,
        );
    }

    // Set parentKey if specified
    if let Some(key) = &texture.parent_key {
        lua_code.push_str(&format!(
            r#"
        parent.{} = tex
        "#,
            key
        ));
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create texture {} on {}: {}",
            tex_name, parent_name, e
        ))
    })
}
