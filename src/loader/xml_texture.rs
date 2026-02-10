//! Texture creation from XML definitions.

use crate::lua_api::LoaderEnv;
use crate::xml::{collect_texture_mixins, register_texture_template};

use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_set_point_code, get_size_values, lua_global_ref, resolve_child_name};
use super::helpers_anim::generate_animation_group_code;

/// Generate Lua code for texture source (file or atlas) and size.
///
/// `is_mask`: MaskTextures default to `useAtlasSize=true` when not explicit,
/// matching WoW behavior where masks auto-size from their atlas.  This matters
/// because the mask frame must be larger than the icon so the icon samples only
/// the opaque center of the mask texture.
fn generate_texture_source_code(texture: &crate::xml::TextureXml, is_mask: bool) -> String {
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
        let default_use_atlas_size = is_mask;
        let use_atlas_size = texture.use_atlas_size.unwrap_or(default_use_atlas_size);
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
        match (x, y) {
            (Some(x), Some(y)) => {
                code.push_str(&format!("\n        tex:SetSize({}, {})\n        ", x, y));
            }
            (Some(x), None) => {
                code.push_str(&format!("\n        tex:SetWidth({})\n        ", x));
            }
            (None, Some(y)) => {
                code.push_str(&format!("\n        tex:SetHeight({})\n        ", y));
            }
            _ => {}
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
    env: &LoaderEnv<'_>,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
    is_mask: bool,
) -> Result<(), LoadError> {
    if texture.is_virtual == Some(true) {
        if let Some(ref name) = texture.name {
            register_texture_template(name, texture.clone());
        }
        return Ok(());
    }

    let tex_name = resolve_child_name(texture.name.as_deref(), parent_name, "__tex_");
    let lua_code = build_texture_lua(&tex_name, texture, parent_name, draw_layer, is_mask);

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to create texture {} on {}: {}",
            tex_name, parent_name, e
        ))
    })?;

    apply_texture_animations_xml(env, texture, &tex_name);
    Ok(())
}

/// Build the Lua code string that creates and configures a texture.
fn build_texture_lua(
    tex_name: &str,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
    is_mask: bool,
) -> String {
    let create_method = if is_mask { "CreateMaskTexture" } else { "CreateTexture" };
    let mut code = format!(
        r#"
        local parent = {}
        local tex = parent:{}("{}", "{}")
        "#,
        lua_global_ref(parent_name), create_method, tex_name, draw_layer
    );
    code.push_str(&generate_mixin_code(texture));
    code.push_str(&generate_texture_source_code(texture, is_mask));
    code.push_str(&generate_texture_visual_code(texture));
    append_texture_anchors(&mut code, texture, parent_name);
    if texture.hidden == Some(true) {
        code.push_str("\n        tex:Hide()\n        ");
    }
    if let Some(a) = texture.alpha {
        code.push_str(&format!("\n        tex:SetAlpha({})\n        ", a));
    }
    if let Some(ref mode) = texture.alpha_mode {
        code.push_str(&format!("\n        tex:SetBlendMode(\"{}\")\n        ", mode));
    }
    // Wire up MaskedTextures: call AddMaskTexture on each referenced sibling.
    if is_mask {
        if let Some(ref masked) = texture.masked_textures {
            for entry in &masked.entries {
                if let Some(ref key) = entry.child_key {
                    code.push_str(&format!(
                        r#"
        if parent.{key} then parent.{key}:AddMaskTexture(tex) end
        "#,
                    ));
                }
            }
        }
    }
    code
}

/// Append anchor or SetAllPoints code for a texture.
fn append_texture_anchors(code: &mut String, texture: &crate::xml::TextureXml, parent_name: &str) {
    if let Some(anchors) = &texture.anchors {
        code.push_str(&generate_set_point_code(anchors, "tex", "parent", parent_name, "parent"));
    } else if texture.set_all_points != Some(true) {
        code.push_str("\n        tex:SetAllPoints(true)\n        ");
    }
}

/// Process animation groups on a texture created from XML.
fn apply_texture_animations_xml(env: &LoaderEnv<'_>, texture: &crate::xml::TextureXml, tex_name: &str) {
    let Some(anims) = &texture.animations else { return };
    let mut anim_code = format!("local frame = {}\n", lua_global_ref(tex_name));
    for anim_group_xml in &anims.animations {
        if anim_group_xml.is_virtual == Some(true) {
            if let Some(ref name) = anim_group_xml.name {
                crate::xml::register_anim_group_template(name, anim_group_xml.clone());
            }
            continue;
        }
        anim_code.push_str(&generate_animation_group_code(anim_group_xml, "frame"));
    }
    env.exec(&anim_code).ok();
}
