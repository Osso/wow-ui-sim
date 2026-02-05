//! Button texture and text application from XML.

use crate::lua_api::WowLuaEnv;

use super::error::LoadError;
use super::helpers::escape_lua_string;

/// Apply button textures (NormalTexture, PushedTexture, etc.) from a FrameXml to a button.
pub fn apply_button_textures(
    env: &WowLuaEnv,
    frame_xml: &crate::xml::FrameXml,
    button_name: &str,
) -> Result<(), LoadError> {
    let mut lua_code = String::new();

    // Helper to generate SetXxxTexture call
    let add_texture = |code: &mut String, method: &str, texture: &crate::xml::TextureXml| {
        if let Some(atlas) = &texture.atlas {
            // Use SetAtlas on the texture object
            code.push_str(&format!(
                r#"
        do
            local tex = {}:{}()
            if tex then tex:SetAtlas("{}") end
        end
        "#,
                button_name,
                method.replace("Set", "Get"),
                escape_lua_string(atlas)
            ));
        } else if let Some(file) = &texture.file {
            // Use SetTexture directly on the button
            code.push_str(&format!(
                r#"
        {}:{}("{}")
        "#,
                button_name,
                method,
                escape_lua_string(file)
            ));
        }
    };

    if let Some(tex) = frame_xml.normal_texture() {
        add_texture(&mut lua_code, "SetNormalTexture", tex);
    }
    if let Some(tex) = frame_xml.pushed_texture() {
        add_texture(&mut lua_code, "SetPushedTexture", tex);
    }
    if let Some(tex) = frame_xml.highlight_texture() {
        add_texture(&mut lua_code, "SetHighlightTexture", tex);
    }
    if let Some(tex) = frame_xml.disabled_texture() {
        add_texture(&mut lua_code, "SetDisabledTexture", tex);
    }

    if !lua_code.is_empty() {
        if let Err(e) = env.exec(&lua_code) {
            return Err(LoadError::Lua(format!(
                "Failed to apply button textures to {}: {}",
                button_name, e
            )));
        }
    }

    Ok(())
}

/// Apply button text from the text attribute on a button.
/// The text attribute is a localization key that gets resolved to actual text.
pub fn apply_button_text(
    env: &WowLuaEnv,
    frame_xml: &crate::xml::FrameXml,
    button_name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    // Check for text attribute on the frame itself first
    let text = if let Some(t) = &frame_xml.text {
        Some(t.clone())
    } else if !inherits.is_empty() {
        // Check inherited templates for text attribute
        let template_chain = crate::xml::get_template_chain(inherits);
        template_chain
            .iter()
            .find_map(|entry| entry.frame.text.clone())
    } else {
        None
    };

    if let Some(text_key) = text {
        // In WoW, text attribute is a localization key or literal text.
        // We try to resolve it via global lookup (e.g., CANCEL -> "Cancel").
        // If not found, use the literal value.
        // Set text on both the button AND its Text fontstring child to ensure rendering works.
        let lua_code = format!(
            r#"
            local frame = {}
            if frame then
                local text = _G["{}"] or "{}"
                if frame.SetText then
                    frame:SetText(text)
                end
                -- Also set text directly on the Text fontstring child
                if frame.Text and frame.Text.SetText then
                    frame.Text:SetText(text)
                end
            end
            "#,
            button_name,
            escape_lua_string(&text_key),
            escape_lua_string(&text_key)
        );
        env.exec(&lua_code).ok(); // Ignore errors (SetText might not exist)
    }

    Ok(())
}
