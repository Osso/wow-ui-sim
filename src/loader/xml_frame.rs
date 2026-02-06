//! Frame creation from XML definitions.

use crate::lua_api::WowLuaEnv;

use super::button::{apply_button_text, apply_button_textures};
use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_anchors_code, generate_scripts_code, get_size_values, rand_id};
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

    // Need a name to create a global frame (unless we have a parent override for anonymous children)
    let name = match &frame.name {
        Some(n) => {
            // Replace $parent with actual parent name if present
            if let Some(parent_name) = parent_override {
                n.replace("$parent", parent_name)
            } else {
                n.clone()
            }
        }
        None => {
            if parent_override.is_some() {
                // Anonymous child frame - generate temp name
                format!("__anon_{}", rand_id())
            } else {
                return Ok(None); // Anonymous top-level frames are templates
            }
        }
    };

    // Build the Lua code to create and configure the frame
    let parent = parent_override
        .or(frame.parent.as_deref())
        .unwrap_or("UIParent");
    let inherits = frame.inherits.as_deref().unwrap_or("");

    // Create the frame
    let mut lua_code = format!(
        r#"
        local frame = CreateFrame("{}", "{}", {}, {})
        "#,
        widget_type,
        name,
        parent,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Set parentKey immediately after frame creation, BEFORE anchors are set.
    // This ensures sibling frames can reference this frame via $parent.ChildKey in their anchors.
    if let Some(parent_key) = &frame.parent_key {
        lua_code.push_str(&format!(
            r#"
        {}.{} = frame
        "#,
            parent, parent_key
        ));
    }

    // Collect mixins from both direct attribute and inherited templates
    let mut all_mixins: Vec<String> = Vec::new();

    // First, collect mixins from inherited templates (base mixins first)
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if let Some(mixin) = &template_entry.frame.mixin {
                for m in mixin.split(',').map(|s| s.trim()) {
                    if !m.is_empty() && !all_mixins.contains(&m.to_string()) {
                        all_mixins.push(m.to_string());
                    }
                }
            }
        }
    }

    // Then add direct mixins (override templates)
    if let Some(mixin) = &frame.mixin {
        for m in mixin.split(',').map(|s| s.trim()) {
            if !m.is_empty() && !all_mixins.contains(&m.to_string()) {
                all_mixins.push(m.to_string());
            }
        }
    }

    // Apply all mixins
    for m in &all_mixins {
        lua_code.push_str(&format!(
            r#"
        if {} then Mixin(frame, {}) end
        "#,
            m, m
        ));
    }

    // Set size - inherit from templates (base to derived), then frame itself overrides
    let mut final_width: Option<f32> = None;
    let mut final_height: Option<f32> = None;

    // First collect sizes from inherited templates (most derived wins)
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if let Some(size) = template_entry.frame.size() {
                let (x, y) = get_size_values(size);
                if let Some(x) = x {
                    final_width = Some(x);
                }
                if let Some(y) = y {
                    final_height = Some(y);
                }
            }
        }
    }

    // Frame's own size overrides template sizes
    if let Some(size) = frame.size() {
        let (x, y) = get_size_values(size);
        if let Some(x) = x {
            final_width = Some(x);
        }
        if let Some(y) = y {
            final_height = Some(y);
        }
    }

    // Apply the final size
    if let (Some(w), Some(h)) = (final_width, final_height) {
        lua_code.push_str(&format!(
            r#"
        frame:SetSize({}, {})
        "#,
            w, h
        ));
    }

    // Set anchors - inherit from templates if frame doesn't define its own
    // Frame's own anchors override template anchors (WoW behavior)
    if let Some(anchors) = frame.anchors() {
        lua_code.push_str(&generate_anchors_code(anchors, parent));
    } else if !inherits.is_empty() {
        // No direct anchors - check templates
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in template_chain.iter().rev() {
            // Most derived template with anchors wins
            if let Some(anchors) = template_entry.frame.anchors() {
                lua_code.push_str(&generate_anchors_code(anchors, parent));
                break;
            }
        }
    }

    // Set hidden state
    if frame.hidden == Some(true) {
        lua_code.push_str(
            r#"
        frame:Hide()
        "#,
        );
    }

    // Set enableMouse (also check inherited templates)
    let mut enable_mouse = frame.enable_mouse;
    if enable_mouse.is_none() && !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
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

    // Handle setAllPoints from inherited templates first
    let mut has_set_all_points = false;
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if template_entry.frame.set_all_points == Some(true) {
                has_set_all_points = true;
                break;
            }
        }
    }

    // Direct attribute overrides template
    if frame.set_all_points == Some(true) {
        has_set_all_points = true;
    }

    // Apply setAllPoints if set
    if has_set_all_points {
        lua_code.push_str(
            r#"
        frame:SetAllPoints(true)
        "#,
        );
    }

    // Handle KeyValues from inherited templates first (so they can be overridden)
    if !inherits.is_empty() {
        let template_chain = crate::xml::get_template_chain(inherits);
        for template_entry in &template_chain {
            if let Some(key_values) = template_entry.frame.key_values() {
                for kv in &key_values.values {
                    let value = match kv.value_type.as_deref() {
                        Some("number") => kv.value.clone(),
                        Some("boolean") => kv.value.to_lowercase(),
                        Some("global") => format!("_G[\"{}\"]", escape_lua_string(&kv.value)),
                        _ => format!("\"{}\"", escape_lua_string(&kv.value)),
                    };
                    lua_code.push_str(&format!(
                        r#"
        frame.{} = {}
        "#,
                        kv.key, value
                    ));
                }
            }
        }
    }

    // Handle KeyValues from the frame itself (can override template values)
    if let Some(key_values) = frame.key_values() {
        for kv in &key_values.values {
            let value = match kv.value_type.as_deref() {
                Some("number") => kv.value.clone(),
                Some("boolean") => kv.value.to_lowercase(),
                Some("global") => format!("_G[\"{}\"]", escape_lua_string(&kv.value)),
                _ => format!("\"{}\"", escape_lua_string(&kv.value)),
            };
            lua_code.push_str(&format!(
                r#"
        frame.{} = {}
        "#,
                kv.key, value
            ));
        }
    }

    // Handle Scripts
    if let Some(scripts) = frame.scripts() {
        lua_code.push_str(&generate_scripts_code(scripts));
    }

    // Execute the creation code
    // NOTE: CreateFrame with an inherits parameter already calls apply_templates_from_registry
    // which creates template children (frames, textures, fontstrings, button textures).
    // Do NOT call instantiate_template_children here - that would duplicate everything.
    env.exec(&lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create frame {}: {}", name, e))
    })?;

    // Handle Layers (textures and fontstrings)
    for layers in frame.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");

            // Create textures
            for texture in layer.textures() {
                create_texture_from_xml(env, texture, &name, draw_layer)?;
            }

            // Create fontstrings
            for fontstring in layer.font_strings() {
                create_fontstring_from_xml(env, fontstring, &name, draw_layer)?;
            }
        }
    }

    // Handle child Frames recursively
    if let Some(frames) = frame.frames() {
        for child in &frames.elements {
            let (child_frame, child_type) = match child {
                crate::xml::FrameElement::Frame(f) => (f, "Frame"),
                crate::xml::FrameElement::Button(f) | crate::xml::FrameElement::ItemButton(f) => (f, "Button"),
                crate::xml::FrameElement::CheckButton(f) => (f, "CheckButton"),
                crate::xml::FrameElement::EditBox(f) | crate::xml::FrameElement::EventEditBox(f) => (f, "EditBox"),
                crate::xml::FrameElement::ScrollFrame(f) => (f, "ScrollFrame"),
                crate::xml::FrameElement::Slider(f) => (f, "Slider"),
                crate::xml::FrameElement::StatusBar(f) => (f, "StatusBar"),
                crate::xml::FrameElement::EventFrame(f) => (f, "Frame"), // EventFrame is just a Frame
                crate::xml::FrameElement::EventButton(f) => (f, "Button"), // EventButton is just a Button
                crate::xml::FrameElement::DropdownButton(f) | crate::xml::FrameElement::DropDownToggleButton(f) => (f, "Button"), // Dropdown buttons
                crate::xml::FrameElement::Cooldown(f) => (f, "Cooldown"),
                crate::xml::FrameElement::GameTooltip(f) => (f, "GameTooltip"),
                crate::xml::FrameElement::Model(f) | crate::xml::FrameElement::ModelScene(f) => (f, "Frame"), // Model frames
                _ => continue, // Skip unsupported types for now
            };
            let child_name = create_frame_from_xml(env, child_frame, child_type, Some(&name))?;

            // Handle parentKey for child frames (works for both named and anonymous frames)
            // The Lua assignment triggers __newindex which syncs to Rust children_keys
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
    }

    // Apply button textures from this frame's XML (NormalTexture, PushedTexture, etc.)
    apply_button_textures(env, frame, &name)?;

    // Apply button text if the text attribute is set
    apply_button_text(env, frame, &name, inherits)?;

    // Fire OnLoad script after frame is fully configured
    // In WoW, OnLoad fires at the end of frame creation from XML
    // Templates often use method="OnLoad" which calls self:OnLoad()
    let onload_code = format!(
        r#"
        local frame = {}
        local handler = frame:GetScript("OnLoad")
        if handler then
            handler(frame)
        elseif type(frame.OnLoad) == "function" then
            -- Call mixin OnLoad method if no script handler but method exists
            frame:OnLoad()
        end
        "#,
        name
    );
    env.exec(&onload_code).ok(); // Ignore errors (OnLoad might not be set)

    // Fire OnShow for visible frames after OnLoad
    // In WoW, OnShow fires when a frame becomes visible, including at creation if visible
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
    env.exec(&onshow_code).ok(); // Ignore errors (OnShow might not be set)

    Ok(Some(name))
}
