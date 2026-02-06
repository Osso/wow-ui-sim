//! Frame creation from XML definitions.

use crate::lua_api::WowLuaEnv;

use super::button::{apply_button_text, apply_button_textures};
use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_anchors_code, generate_animation_group_code, generate_scripts_code, get_size_values, rand_id};
use super::xml_fontstring::create_fontstring_from_xml;
use super::xml_texture::create_texture_from_xml;

/// Create a frame from XML definition.
/// Returns the name of the created frame (or None if skipped).
pub fn create_frame_from_xml(
    env: &WowLuaEnv,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
) -> Result<Option<String>, LoadError> {
    // Register virtual frames (templates) in the template registry
    if frame.is_virtual == Some(true) {
        if let Some(ref name) = frame.name {
            crate::xml::register_template(name, widget_type, frame.clone());
        }
        return Ok(None);
    }

    let name = match resolve_frame_name(frame, parent_override) {
        Some(n) => n,
        None => return Ok(None),
    };

    let parent = parent_override
        .or(frame.parent.as_deref())
        .unwrap_or("UIParent");
    let inherits = frame.inherits.as_deref().unwrap_or("");

    let mut lua_code = build_create_frame_code(widget_type, &name, parent, inherits);

    append_parent_key_code(&mut lua_code, frame, parent);
    append_mixins_code(&mut lua_code, frame, inherits);
    append_size_code(&mut lua_code, frame, inherits);
    append_anchors_code(&mut lua_code, frame, inherits, parent);
    append_hidden_code(&mut lua_code, frame, inherits);
    append_enable_mouse_code(&mut lua_code, frame, inherits);
    append_set_all_points_code(&mut lua_code, frame, inherits);
    append_key_values_code(&mut lua_code, frame, inherits);
    append_scripts_code(&mut lua_code, frame);

    // Execute the creation code
    // NOTE: CreateFrame with an inherits parameter already calls apply_templates_from_registry
    // which creates template children (frames, textures, fontstrings, button textures).
    // Do NOT call instantiate_template_children here - that would duplicate everything.
    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create frame {}: {}", name, e))
    })?;

    // Child frames first: they may be referenced by layer children via relativeKey
    create_child_frames(env, frame, &name)?;
    create_layer_children(env, frame, &name)?;
    apply_animation_groups(env, frame, &name, inherits)?;

    apply_button_textures(env, frame, &name)?;
    apply_button_text(env, frame, &name, inherits)?;

    fire_lifecycle_scripts(env, &name);

    Ok(Some(name))
}

/// Resolve the frame name, applying `$parent` substitution and generating anonymous names.
/// Returns `None` if the frame should be skipped (anonymous top-level frame).
fn resolve_frame_name(frame: &crate::xml::FrameXml, parent_override: Option<&str>) -> Option<String> {
    match &frame.name {
        Some(n) => {
            if let Some(parent_name) = parent_override {
                Some(n.replace("$parent", parent_name))
            } else {
                Some(n.clone())
            }
        }
        None => {
            if parent_override.is_some() {
                Some(format!("__anon_{}", rand_id()))
            } else {
                None // Anonymous top-level frames are templates
            }
        }
    }
}

/// Build the initial `CreateFrame(...)` Lua code.
fn build_create_frame_code(widget_type: &str, name: &str, parent: &str, inherits: &str) -> String {
    let inherits_arg = if inherits.is_empty() {
        "nil".to_string()
    } else {
        format!("\"{}\"", inherits)
    };
    format!(
        r#"
        local frame = CreateFrame("{}", "{}", {}, {})
        "#,
        widget_type, name, parent, inherits_arg
    )
}

/// Append parentKey assignment so sibling frames can reference this frame.
fn append_parent_key_code(lua_code: &mut String, frame: &crate::xml::FrameXml, parent: &str) {
    if let Some(parent_key) = &frame.parent_key {
        lua_code.push_str(&format!(
            r#"
        {}.{} = frame
        "#,
            parent, parent_key
        ));
    }
}

/// Collect mixins from inherited templates and the frame itself, then append Mixin() calls.
fn append_mixins_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut all_mixins: Vec<String> = Vec::new();

    // Collect from inherited templates (base mixins first)
    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            collect_mixins_from_attr(&mut all_mixins, template_entry.frame.combined_mixin().as_deref());
        }
    }

    // Direct mixins (override templates)
    collect_mixins_from_attr(&mut all_mixins, frame.combined_mixin().as_deref());

    for m in &all_mixins {
        lua_code.push_str(&format!(
            r#"
        if {} then Mixin(frame, {}) end
        "#,
            m, m
        ));
    }
}

/// Parse a comma-separated mixin attribute and append unique entries.
fn collect_mixins_from_attr(all_mixins: &mut Vec<String>, mixin_attr: Option<&str>) {
    if let Some(mixin) = mixin_attr {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() && !all_mixins.contains(&m.to_string()) {
                all_mixins.push(m.to_string());
            }
        }
    }
}

/// Resolve size from templates (base to derived) then the frame itself, and append SetSize.
fn append_size_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut final_width: Option<f32> = None;
    let mut final_height: Option<f32> = None;

    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            apply_size_from_xml(&mut final_width, &mut final_height, template_entry.frame.size());
        }
    }

    apply_size_from_xml(&mut final_width, &mut final_height, frame.size());

    if let (Some(w), Some(h)) = (final_width, final_height) {
        lua_code.push_str(&format!(
            r#"
        frame:SetSize({}, {})
        "#,
            w, h
        ));
    }
}

/// Update width/height from a SizeXml if present.
fn apply_size_from_xml(width: &mut Option<f32>, height: &mut Option<f32>, size: Option<&crate::xml::SizeXml>) {
    if let Some(size) = size {
        let (x, y) = get_size_values(size);
        if let Some(x) = x {
            *width = Some(x);
        }
        if let Some(y) = y {
            *height = Some(y);
        }
    }
}

/// Append anchor SetPoint calls from the frame or inherited templates.
fn append_anchors_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str, parent: &str) {
    if let Some(anchors) = frame.anchors() {
        lua_code.push_str(&generate_anchors_code(anchors, parent));
    } else if !inherits.is_empty() {
        // No direct anchors - most derived template with anchors wins
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in template_chain.iter().rev() {
            if let Some(anchors) = template_entry.frame.anchors() {
                lua_code.push_str(&generate_anchors_code(anchors, parent));
                break;
            }
        }
    }
}

/// Append `frame:Hide()` if the frame is marked hidden (directly or via template).
fn append_hidden_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut hidden = frame.hidden;
    if hidden.is_none() && !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if let Some(h) = template_entry.frame.hidden {
                hidden = Some(h);
                break;
            }
        }
    }
    if hidden == Some(true) {
        lua_code.push_str(
            r#"
        frame:Hide()
        "#,
        );
    }
}

/// Resolve enableMouse from the frame and templates, then append EnableMouse call.
fn append_enable_mouse_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut enable_mouse = frame.enable_mouse;
    if enable_mouse.is_none() && !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if let Some(em) = template_entry.frame.enable_mouse {
                enable_mouse = Some(em);
            }
        }
    }
    if let Some(enabled) = enable_mouse {
        lua_code.push_str(&format!(
            r#"
        frame:EnableMouse({})
        "#,
            if enabled { "true" } else { "false" }
        ));
    }
}

/// Resolve setAllPoints from templates and frame, then append SetAllPoints call.
fn append_set_all_points_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut has_set_all_points = false;

    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if template_entry.frame.set_all_points == Some(true) {
                has_set_all_points = true;
                break;
            }
        }
    }

    if frame.set_all_points == Some(true) {
        has_set_all_points = true;
    }

    if has_set_all_points {
        lua_code.push_str(
            r#"
        frame:SetAllPoints(true)
        "#,
        );
    }
}

/// Append KeyValue assignments from templates and the frame itself.
fn append_key_values_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            append_key_values_from_xml(lua_code, template_entry.frame.key_values());
        }
    }
    append_key_values_from_xml(lua_code, frame.key_values());
}

/// Append `frame.key = value` assignments for a KeyValues block.
fn append_key_values_from_xml(lua_code: &mut String, key_values: Option<&crate::xml::KeyValuesXml>) {
    if let Some(key_values) = key_values {
        for kv in &key_values.values {
            let value = format_key_value_lua(&kv.value, kv.value_type.as_deref());
            lua_code.push_str(&format!(
                r#"
        frame.{} = {}
        "#,
                kv.key, value
            ));
        }
    }
}

/// Format a KeyValue's value as a Lua expression based on its type.
fn format_key_value_lua(value: &str, value_type: Option<&str>) -> String {
    match value_type {
        Some("number") => value.to_string(),
        Some("boolean") => value.to_lowercase(),
        Some("global") => format!("_G[\"{}\"]", escape_lua_string(value)),
        _ => format!("\"{}\"", escape_lua_string(value)),
    }
}

/// Append script handler registrations from the frame's Scripts element.
fn append_scripts_code(lua_code: &mut String, frame: &crate::xml::FrameXml) {
    if let Some(scripts) = frame.scripts() {
        lua_code.push_str(&generate_scripts_code(scripts));
    }
}

/// Create textures and fontstrings from the frame's Layers.
fn create_layer_children(env: &WowLuaEnv, frame: &crate::xml::FrameXml, name: &str) -> Result<(), LoadError> {
    for layers in frame.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");
            for texture in layer.textures() {
                create_texture_from_xml(env, texture, name, draw_layer)?;
            }
            for fontstring in layer.font_strings() {
                create_fontstring_from_xml(env, fontstring, name, draw_layer)?;
            }
        }
    }
    Ok(())
}

/// Map a FrameElement variant to its (FrameXml, widget_type) pair.
/// Returns None for unsupported element types.
fn frame_element_to_type(child: &crate::xml::FrameElement) -> Option<(&crate::xml::FrameXml, &'static str)> {
    match child {
        crate::xml::FrameElement::Frame(f) => Some((f, "Frame")),
        crate::xml::FrameElement::Button(f) | crate::xml::FrameElement::ItemButton(f) => Some((f, "Button")),
        crate::xml::FrameElement::CheckButton(f) => Some((f, "CheckButton")),
        crate::xml::FrameElement::EditBox(f) | crate::xml::FrameElement::EventEditBox(f) => Some((f, "EditBox")),
        crate::xml::FrameElement::ScrollFrame(f) => Some((f, "ScrollFrame")),
        crate::xml::FrameElement::Slider(f) => Some((f, "Slider")),
        crate::xml::FrameElement::StatusBar(f) => Some((f, "StatusBar")),
        crate::xml::FrameElement::EventFrame(f) => Some((f, "Frame")),
        crate::xml::FrameElement::EventButton(f) => Some((f, "Button")),
        crate::xml::FrameElement::DropdownButton(f) | crate::xml::FrameElement::DropDownToggleButton(f) => Some((f, "Button")),
        crate::xml::FrameElement::Cooldown(f) => Some((f, "Cooldown")),
        crate::xml::FrameElement::GameTooltip(f) => Some((f, "GameTooltip")),
        crate::xml::FrameElement::Model(f) => Some((f, "Model")),
        crate::xml::FrameElement::ModelScene(f) => Some((f, "ModelScene")),
        crate::xml::FrameElement::TaxiRouteFrame(f)
        | crate::xml::FrameElement::ModelFFX(f)
        | crate::xml::FrameElement::TabardModel(f)
        | crate::xml::FrameElement::UiCamera(f)
        | crate::xml::FrameElement::UnitPositionFrame(f)
        | crate::xml::FrameElement::OffScreenFrame(f)
        | crate::xml::FrameElement::Checkout(f)
        | crate::xml::FrameElement::FogOfWarFrame(f)
        | crate::xml::FrameElement::QuestPOIFrame(f)
        | crate::xml::FrameElement::ArchaeologyDigSiteFrame(f)
        | crate::xml::FrameElement::ScenarioPOIFrame(f)
        | crate::xml::FrameElement::UIThemeContainerFrame(f)
        | crate::xml::FrameElement::ContainedAlertFrame(f)
        | crate::xml::FrameElement::MapScene(f)
        | crate::xml::FrameElement::ScopedModifier(f)
        | crate::xml::FrameElement::Line(f) => Some((f, "Frame")),
        crate::xml::FrameElement::EventScrollFrame(f) => Some((f, "ScrollFrame")),
        _ => None,
    }
}

/// Recursively create child frames and assign parentKey references.
fn create_child_frames(env: &WowLuaEnv, frame: &crate::xml::FrameXml, name: &str) -> Result<(), LoadError> {
    let frames = match frame.frames() {
        Some(f) => f,
        None => return Ok(()),
    };

    for child in &frames.elements {
        let (child_frame, child_type) = match frame_element_to_type(child) {
            Some(pair) => pair,
            None => continue,
        };
        let child_name = create_frame_from_xml(env, child_frame, child_type, Some(name))?;

        // Assign parentKey so the parent can reference the child.
        // The Lua assignment triggers __newindex which syncs to Rust children_keys.
        if let (Some(actual_child_name), Some(parent_key)) =
            (child_name.clone(), &child_frame.parent_key)
        {
            let lua_code = format!(
                r#"
                    {}.{} = {}
                    "#,
                name, parent_key, actual_child_name
            );
            env.exec(&lua_code).ok(); // Ignore errors (parent might not exist yet)
        }
    }
    Ok(())
}

/// Apply animation groups from the frame and its inherited templates.
fn apply_animation_groups(env: &WowLuaEnv, frame: &crate::xml::FrameXml, name: &str, inherits: &str) -> Result<(), LoadError> {
    if let Some(anims) = frame.animations() {
        exec_animation_groups(env, anims, name);
    }

    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if let Some(anims) = template_entry.frame.animations() {
                exec_animation_groups(env, anims, name);
            }
        }
    }
    Ok(())
}

/// Generate and execute Lua code for a set of animation groups on a frame.
fn exec_animation_groups(env: &WowLuaEnv, anims: &crate::xml::AnimationsXml, name: &str) {
    let mut anim_code = format!(
        r#"
            local frame = {}
            "#,
        name
    );
    for anim_group_xml in &anims.animations {
        if anim_group_xml.is_virtual == Some(true) {
            continue;
        }
        anim_code.push_str(&generate_animation_group_code(anim_group_xml, "frame"));
    }
    env.exec(&anim_code).ok();
}

/// Fire OnLoad and OnShow lifecycle scripts after the frame is fully configured.
fn fire_lifecycle_scripts(env: &WowLuaEnv, name: &str) {
    // In WoW, OnLoad fires at the end of frame creation from XML.
    // Templates often use method="OnLoad" which calls self:OnLoad().
    let onload_code = format!(
        r#"
        local frame = {}
        local handler = frame:GetScript("OnLoad")
        if handler then
            handler(frame)
        elseif type(frame.OnLoad) == "function" then
            frame:OnLoad()
        end
        "#,
        name
    );
    env.exec(&onload_code).ok();

    // In WoW, OnShow fires when a frame becomes visible, including at creation if visible.
    let onshow_code = format!(
        r#"
        local frame = {}
        if frame:IsVisible() then
            local handler = frame:GetScript("OnShow")
            if handler then
                handler(frame)
            elseif type(frame.OnShow) == "function" then
                frame:OnShow()
            end
        end
        "#,
        name
    );
    env.exec(&onshow_code).ok();
}
