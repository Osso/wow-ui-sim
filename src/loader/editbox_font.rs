//! The `<FontString>` element of an EditBox: the font, justification and
//! color of its text.
//!
//! `<EditBox><FontString inherits="ChatFontNormal" justifyH="LEFT"/></EditBox>`
//! describes the box's own text, not a child region. Blizzard's input box
//! templates set their text font this way (InputBoxVisualTemplate:
//! `<FontString inherits="ChatFontNormal"/>`), so a SearchBoxTemplate box
//! shows white ChatFontNormal text in the client.

use crate::lua_api::LoaderEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, lua_global_ref};

/// The EditBox's own `<FontString>` element, else the nearest one in its
/// template chain.
fn resolve_editbox_font_string(
    frame_xml: &crate::xml::FrameXml,
    inherits: &str,
) -> Option<crate::xml::FontStringXml> {
    if let Some(font_string) = frame_xml.font_string_child() {
        return Some(font_string.clone());
    }
    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| entry.frame.font_string_child().cloned())
}

/// Lua that applies `font_string` to the EditBox `box_ref`: the font object it
/// inherits, then the justification and color it names. The font object
/// brings no justification of its own here: a SearchBoxTemplate box, whose
/// element names only ChatFontNormal, stays "LEFT" in the client.
fn generate_editbox_font_code(box_ref: &str, font_string: &crate::xml::FontStringXml) -> String {
    let mut body = String::from("local h, v = box:GetJustifyH(), box:GetJustifyV()\n");
    if let Some(inherits) = font_string.inherits.as_deref() {
        for name in inherits
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            body.push_str(&format!(
                "local fo = _G[\"{}\"] if fo then box:SetFontObject(fo) end\n",
                escape_lua_string(name)
            ));
        }
    }
    let justify_h = font_string
        .justify_h
        .as_deref()
        .map_or("h".to_string(), |justify| {
            format!("\"{}\"", escape_lua_string(justify))
        });
    let justify_v = font_string
        .justify_v
        .as_deref()
        .map_or("v".to_string(), |justify| {
            format!("\"{}\"", escape_lua_string(justify))
        });
    body.push_str(&format!(
        "box:SetJustifyH({justify_h})\nbox:SetJustifyV({justify_v})\n"
    ));
    if let Some(color) = font_string.color.as_ref() {
        body.push_str(&format!(
            "box:SetTextColor({}, {}, {}, {})\n",
            color.r.unwrap_or(1.0),
            color.g.unwrap_or(1.0),
            color.b.unwrap_or(1.0),
            color.a.unwrap_or(1.0)
        ));
    }
    format!(
        "do local box = {box} if box and box.GetObjectType and box:GetObjectType() == \"EditBox\" then\n{body}end end\n",
        box = lua_global_ref(box_ref),
    )
}

/// Apply the EditBox `<FontString>` element of `frame_xml` (or of its
/// template chain `inherits`) to the frame `box_ref_name`.
pub fn apply_editbox_font_with_ref(
    env: &LoaderEnv<'_>,
    frame_xml: &crate::xml::FrameXml,
    box_name: &str,
    box_ref_name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    let Some(font_string) = resolve_editbox_font_string(frame_xml, inherits) else {
        return Ok(());
    };
    let code = generate_editbox_font_code(box_ref_name, &font_string);
    env.exec(&code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to apply the text font of EditBox {box_name}: {e}"
        ))
    })
}
