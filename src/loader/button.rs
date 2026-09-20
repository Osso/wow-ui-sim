//! Button texture and text application from XML.

use crate::lua_api::LoaderEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_set_point_code, get_size_values, lua_global_ref};

/// Generate the setter portion of button texture Lua code (atlas or file path).
fn generate_texture_setter_code(
    button_ref_name: &str,
    method: &str,
    texture: &crate::xml::TextureXml,
) -> String {
    let getter = method.replace("Set", "Get");
    if let Some(atlas) = &texture.atlas {
        format!(
            "do\n    local tex = {}:{}()\n    if tex then tex:SetAtlas(\"{}\") end\nend\n",
            lua_global_ref(button_ref_name),
            getter,
            escape_lua_string(atlas)
        )
    } else if let Some(file) = &texture.file {
        format!(
            "{}:{}(\"{}\")\n",
            lua_global_ref(button_ref_name),
            method,
            escape_lua_string(file)
        )
    } else {
        String::new()
    }
}

/// Generate the global registration snippet for a named button texture.
fn generate_texture_global_snippet(
    button_name: &str,
    button_ref_name: &str,
    method: &str,
    texture: &crate::xml::TextureXml,
) -> String {
    let getter = method.replace("Set", "Get");
    texture
        .name
        .as_ref()
        .map(|n| {
            let resolved = n.replace("$parent", button_name);
            format!(
                "do local t = {}:{}() if t then _G[\"{}\"] = t end end\n",
                lua_global_ref(button_ref_name),
                getter,
                escape_lua_string(&resolved),
            )
        })
        .unwrap_or_default()
}

fn generate_texture_layout_snippet(
    button_name: &str,
    button_ref_name: &str,
    method: &str,
    texture: &crate::xml::TextureXml,
) -> String {
    let getter = method.replace("Set", "Get");
    let mut code = format!(
        "do\n    local parent = {button_ref}\n    local tex = parent and parent:{getter}()\n    if tex then\n",
        button_ref = lua_global_ref(button_ref_name),
        getter = getter,
    );

    if let Some(size) = &texture.size {
        let (x, y) = get_size_values(size);
        match (x, y) {
            (Some(x), Some(y)) => code.push_str(&format!("        tex:SetSize({x}, {y})\n")),
            (Some(x), None) => code.push_str(&format!("        tex:SetWidth({x})\n")),
            (None, Some(y)) => code.push_str(&format!("        tex:SetHeight({y})\n")),
            (None, None) => {}
        }
    }

    if let Some(anchors) = &texture.anchors {
        code.push_str("        tex:ClearAllPoints()\n");
        code.push_str(&generate_set_point_code(
            anchors,
            "tex",
            "parent",
            button_name,
            "parent",
        ));
    } else if texture.set_all_points == Some(true) {
        code.push_str("        tex:SetAllPoints(true)\n");
    }

    code.push_str(&generate_texture_tex_coords_snippet(texture));
    if let Some(animations) = &texture.animations {
        for group in &animations.animations {
            if group.is_virtual != Some(true) {
                code.push_str(&super::helpers_anim::generate_animation_group_code(
                    group, "tex",
                ));
            }
        }
    }

    code.push_str("    end\nend\n");
    code
}

fn generate_texture_tex_coords_snippet(texture: &crate::xml::TextureXml) -> String {
    let Some(tex_coords) = &texture.tex_coords else {
        return String::new();
    };

    let left = tex_coords.left.unwrap_or(0.0);
    let right = tex_coords.right.unwrap_or(1.0);
    let top = tex_coords.top.unwrap_or(0.0);
    let bottom = tex_coords.bottom.unwrap_or(1.0);
    format!("        tex:SetTexCoord({left}, {right}, {top}, {bottom})\n")
}

/// Generate the per-button field assignment for a button texture parentKey.
///
/// WoW exposes custom parentKeys on button texture slots as fields on the button
/// itself (for example `<NormalTexture parentKey="texture">` gives
/// `button.texture`). The normal button slot (`NormalTexture`, `HighlightTexture`,
/// etc.) still exists via the getter/setter methods; this only mirrors the extra
/// Lua field expected by Blizzard code.
fn generate_texture_parent_key_snippet(
    button_ref_name: &str,
    method: &str,
    texture: &crate::xml::TextureXml,
) -> String {
    let Some(parent_key) = texture.parent_key.as_deref() else {
        return String::new();
    };
    let getter = method.replace("Set", "Get");
    format!(
        "do local b = {button_ref} local t = b and b:{getter}() if b and t then b[\"{parent_key}\"] = t end end\n",
        button_ref = lua_global_ref(button_ref_name),
        getter = getter,
        parent_key = escape_lua_string(parent_key),
    )
}

/// Generate Lua code for a single button texture (atlas or file path),
/// and register the texture as a global if it has a `$parent`-prefixed name.
fn generate_button_texture_code(
    button_name: &str,
    button_ref_name: &str,
    method: &str,
    texture: &crate::xml::TextureXml,
) -> String {
    let mut code = generate_texture_setter_code(button_ref_name, method, texture);
    code.push_str(&generate_texture_layout_snippet(
        button_name,
        button_ref_name,
        method,
        texture,
    ));
    code.push_str(&generate_texture_parent_key_snippet(
        button_ref_name,
        method,
        texture,
    ));
    code.push_str(&generate_texture_global_snippet(
        button_name,
        button_ref_name,
        method,
        texture,
    ));
    code
}

/// Apply button textures (NormalTexture, PushedTexture, etc.) from a FrameXml to a button.
///
/// First ensures texture children exist for ALL XML texture slots (atlas, file, or empty),
/// then generates Lua code to set atlas/file on them. This ordering is critical: Lua code
/// calls `Get*Texture()` which returns nil if the child doesn't exist yet.
pub fn apply_button_textures_with_ref(
    env: &LoaderEnv<'_>,
    frame_xml: &crate::xml::FrameXml,
    button_name: &str,
    button_ref_name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    let texture_slots = button_texture_slots(frame_xml, inherits);

    // Create texture children for ALL slots BEFORE running Lua code.
    // The Lua atlas path calls Get*Texture() which needs the child to exist.
    ensure_button_texture_children(env, &texture_slots, button_name);
    apply_button_texture_lua(env, &texture_slots, button_ref_name, button_name)
}

type SlotAccessor = fn(&crate::xml::FrameXml) -> Option<&crate::xml::TextureXml>;

const BUTTON_TEXTURE_SLOT_DEFS: [(&str, &str, SlotAccessor); 6] = [
    (
        "SetNormalTexture",
        "NormalTexture",
        crate::xml::FrameXml::normal_texture,
    ),
    (
        "SetPushedTexture",
        "PushedTexture",
        crate::xml::FrameXml::pushed_texture,
    ),
    (
        "SetHighlightTexture",
        "HighlightTexture",
        crate::xml::FrameXml::highlight_texture,
    ),
    (
        "SetDisabledTexture",
        "DisabledTexture",
        crate::xml::FrameXml::disabled_texture,
    ),
    (
        "SetCheckedTexture",
        "CheckedTexture",
        crate::xml::FrameXml::checked_texture,
    ),
    (
        "SetDisabledCheckedTexture",
        "DisabledCheckedTexture",
        crate::xml::FrameXml::disabled_checked_texture,
    ),
];

fn button_texture_slots(
    frame_xml: &crate::xml::FrameXml,
    inherits: &str,
) -> [(&'static str, &'static str, Option<crate::xml::TextureXml>); 6] {
    BUTTON_TEXTURE_SLOT_DEFS.map(|(method, parent_key, accessor)| {
        (
            method,
            parent_key,
            resolve_button_texture_slot(frame_xml, inherits, accessor),
        )
    })
}

fn resolve_button_texture_slot(
    frame_xml: &crate::xml::FrameXml,
    inherits: &str,
    slot: fn(&crate::xml::FrameXml) -> Option<&crate::xml::TextureXml>,
) -> Option<crate::xml::TextureXml> {
    if let Some(texture) = slot(frame_xml) {
        return Some(crate::xml::resolve_texture_inheritance(texture));
    }

    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| slot(&entry.frame).cloned())
        .map(|texture| crate::xml::resolve_texture_inheritance(&texture))
}

fn apply_button_texture_lua(
    env: &LoaderEnv<'_>,
    texture_slots: &[(&'static str, &'static str, Option<crate::xml::TextureXml>); 6],
    button_ref_name: &str,
    button_name: &str,
) -> Result<(), LoadError> {
    let lua_code: String = texture_slots
        .iter()
        .filter_map(|(method, _, tex)| {
            tex.as_ref().map(|texture| {
                generate_button_texture_code(button_name, button_ref_name, method, texture)
            })
        })
        .collect();

    if !lua_code.is_empty() {
        env.exec(&lua_code).map_err(|e| {
            LoadError::Lua(format!(
                "Failed to apply button textures to {}: {}",
                button_name, e
            ))
        })?;
    }

    Ok(())
}

/// Ensure texture children exist for all button XML texture slots.
/// Creates empty texture children for any slot that has an XML element,
/// regardless of whether it has atlas/file attributes.
fn ensure_button_texture_children(
    env: &LoaderEnv<'_>,
    slots: &[(&'static str, &'static str, Option<crate::xml::TextureXml>); 6],
    button_name: &str,
) {
    use crate::lua_api::frame::methods::methods_helpers::get_or_create_button_texture;
    let button_id = env.state().borrow().widgets.get_id_by_name(button_name);
    let Some(button_id) = button_id else { return };
    for (_, parent_key, tex_opt) in slots {
        if tex_opt.is_none() {
            continue;
        }
        let mut state = env.state().borrow_mut();
        get_or_create_button_texture(&mut state, button_id, parent_key);
    }
}

/// Resolve the text key for a button: frame attribute takes priority, then inherited templates.
fn resolve_button_text_key(frame_xml: &crate::xml::FrameXml, inherits: &str) -> Option<String> {
    if let Some(t) = &frame_xml.text {
        return Some(t.clone());
    }
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        return template_chain
            .iter()
            .find_map(|entry| entry.frame.text.clone());
    }
    None
}

fn resolve_button_text_child<'a>(
    frame_xml: &'a crate::xml::FrameXml,
    inherits: &str,
) -> Option<crate::xml::FontStringXml> {
    if let Some(button_text) = frame_xml.button_text() {
        return Some(button_text.clone());
    }

    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| entry.frame.button_text().cloned())
}

/// Generate Lua code to set text on a button and its Text fontstring child.
fn generate_set_text_code(button_ref_name: &str, text_key: &str) -> String {
    format!(
        "local frame = {ref}\nif frame then\n\
         local text = _G[\"{key}\"] or \"{key}\"\n\
         if frame.SetText then frame:SetText(text) end\n\
         if frame.Text and frame.Text.SetText then frame.Text:SetText(text) end\n\
         end\n",
        ref = lua_global_ref(button_ref_name),
        key = escape_lua_string(text_key),
    )
}

/// Resolve a button font ref slot (NormalFont/HighlightFont/DisabledFont) from a frame
/// or its inherited template chain.
fn resolve_button_font_slot(
    frame_xml: &crate::xml::FrameXml,
    inherits: &str,
    accessor: fn(&crate::xml::FrameXml) -> Option<&crate::xml::FontRefXml>,
) -> Option<crate::xml::FontRefXml> {
    if let Some(font_ref) = accessor(frame_xml) {
        return Some(font_ref.clone());
    }
    crate::xml::get_template_chain(inherits)
        .iter()
        .rev()
        .find_map(|entry| accessor(&entry.frame).cloned())
}

fn frame_normal_font(frame_xml: &crate::xml::FrameXml) -> Option<&crate::xml::FontRefXml> {
    frame_xml.children.iter().find_map(|c| match c {
        crate::xml::FrameChildElement::NormalFont(f) => Some(f),
        _ => None,
    })
}

fn frame_highlight_font(frame_xml: &crate::xml::FrameXml) -> Option<&crate::xml::FontRefXml> {
    frame_xml.children.iter().find_map(|c| match c {
        crate::xml::FrameChildElement::HighlightFont(f) => Some(f),
        _ => None,
    })
}

fn frame_disabled_font(frame_xml: &crate::xml::FrameXml) -> Option<&crate::xml::FontRefXml> {
    frame_xml.children.iter().find_map(|c| match c {
        crate::xml::FrameChildElement::DisabledFont(f) => Some(f),
        _ => None,
    })
}

fn font_ref_style(font_ref: &crate::xml::FontRefXml) -> Option<&str> {
    font_ref
        .style
        .as_deref()
        .or(font_ref.inherits.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn generate_button_font_setter(button_name: &str, method: &str, style: &str) -> String {
    format!(
        "do local fo = _G[\"{style}\"] if fo and {btn} and {btn}.{method} then {btn}:{method}(fo) end end\n",
        style = escape_lua_string(style),
        btn = lua_global_ref(button_name),
        method = method,
    )
}

/// Apply button font references (NormalFont/HighlightFont/DisabledFont) from XML.
///
/// `<NormalFont style="GameFontNormalSmall"/>` is parsed into FrameChildElement variants
/// but is otherwise inert; this turns those declarations into runtime
/// `Button:SetNormalFontObject(_G["GameFontNormalSmall"])` calls so the button text
/// child picks up the font/size/outline/color from the font object.
pub fn apply_button_fonts_with_ref(
    env: &LoaderEnv<'_>,
    frame_xml: &crate::xml::FrameXml,
    button_name: &str,
    button_ref_name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    let slots: [(
        &str,
        fn(&crate::xml::FrameXml) -> Option<&crate::xml::FontRefXml>,
    ); 3] = [
        ("SetNormalFontObject", frame_normal_font),
        ("SetHighlightFontObject", frame_highlight_font),
        ("SetDisabledFontObject", frame_disabled_font),
    ];

    let lua_code: String = slots
        .iter()
        .filter_map(|(method, accessor)| {
            let font_ref = resolve_button_font_slot(frame_xml, inherits, *accessor)?;
            let style = font_ref_style(&font_ref)?.to_string();
            Some(generate_button_font_setter(button_ref_name, method, &style))
        })
        .collect();

    if lua_code.is_empty() {
        return Ok(());
    }

    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!(
            "Failed to apply button fonts to {}: {}",
            button_name, e
        ))
    })?;
    Ok(())
}

/// Apply button text from the text attribute and ButtonText child element.
pub fn apply_button_text_with_ref(
    env: &LoaderEnv<'_>,
    frame_xml: &crate::xml::FrameXml,
    button_name: &str,
    button_ref_name: &str,
    inherits: &str,
) -> Result<(), LoadError> {
    if let Some(mut button_text) = resolve_button_text_child(frame_xml, inherits) {
        if button_text.parent_key.is_none() {
            button_text.parent_key = Some("Text".to_string());
        }
        super::xml_fontstring::create_fontstring_from_xml_with_ref(
            env,
            &button_text,
            button_name,
            button_ref_name,
            "ARTWORK",
            0,
        )?;
    }
    if let Some(text_key) = resolve_button_text_key(frame_xml, inherits) {
        env.exec(&generate_set_text_code(button_ref_name, &text_key))
            .ok();
    }
    Ok(())
}
