//! Template application from the XML template registry.
//!
//! This module provides functionality to apply XML templates from the registry
//! when CreateFrame is called with a template name.

use crate::xml::{get_template_chain, FrameElement, FrameXml, TemplateEntry};
use mlua::Lua;

/// Extract the FrameXml and widget type string from a FrameElement.
fn frame_element_type(element: &FrameElement) -> Option<(&FrameXml, &'static str)> {
    match element {
        FrameElement::Frame(f) => Some((f, "Frame")),
        FrameElement::Button(f) | FrameElement::ItemButton(f) => Some((f, "Button")),
        FrameElement::CheckButton(f) => Some((f, "CheckButton")),
        FrameElement::EditBox(f) | FrameElement::EventEditBox(f) => Some((f, "EditBox")),
        FrameElement::ScrollFrame(f) => Some((f, "ScrollFrame")),
        FrameElement::Slider(f) => Some((f, "Slider")),
        FrameElement::StatusBar(f) => Some((f, "StatusBar")),
        FrameElement::EventFrame(f) => Some((f, "Frame")),
        FrameElement::EventButton(f) => Some((f, "Button")),
        FrameElement::DropdownButton(f) | FrameElement::DropDownToggleButton(f) => {
            Some((f, "Button"))
        }
        FrameElement::Cooldown(f) => Some((f, "Cooldown")),
        FrameElement::GameTooltip(f) => Some((f, "GameTooltip")),
        FrameElement::Model(f) | FrameElement::ModelScene(f) => Some((f, "Frame")),
        _ => None,
    }
}

/// Apply templates from the registry to a frame.
///
/// This generates Lua code to create child frames, textures, and fontstrings
/// defined in the template chain (including inherited templates).
pub fn apply_templates_from_registry(lua: &Lua, frame_name: &str, template_names: &str) {
    let chain = get_template_chain(template_names);
    if chain.is_empty() {
        return;
    }

    for entry in &chain {
        apply_single_template(lua, frame_name, entry);
    }

    // Fire OnLoad after all templates are applied
    fire_on_load(lua, frame_name);
}

/// Apply a single template's children to a frame.
fn apply_single_template(lua: &Lua, frame_name: &str, entry: &TemplateEntry) {
    let template = &entry.frame;

    // Apply size from template if defined
    if let Some(size) = template.size() {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            let code = format!(
                r#"
                local frame = {}
                if frame and frame:GetWidth() == 0 and frame:GetHeight() == 0 then
                    frame:SetSize({}, {})
                end
                "#,
                frame_name, w, h
            );
            let _ = lua.load(&code).exec();
        }
    }

    // Apply mixin from template (must be before scripts)
    apply_mixin(lua, &template.mixin, frame_name);

    // Apply KeyValues from template
    if let Some(key_values) = template.key_values() {
        for kv in &key_values.values {
            let value = match kv.value_type.as_deref() {
                Some("number") => kv.value.clone(),
                Some("boolean") => kv.value.to_lowercase(),
                _ => format!("\"{}\"", escape_lua_string(&kv.value)),
            };
            let code = format!(
                r#"
                local frame = {}
                if frame then frame.{} = {} end
                "#,
                frame_name, kv.key, value
            );
            let _ = lua.load(&code).exec();
        }
    }

    // Create textures from Layers
    for layers in template.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");

            for texture in layer.textures() {
                create_texture_from_template(lua, texture, frame_name, draw_layer);
            }

            for fontstring in layer.font_strings() {
                create_fontstring_from_template(lua, fontstring, frame_name, draw_layer);
            }
        }
    }

    // Create ThumbTexture for sliders
    if let Some(thumb) = template.thumb_texture() {
        create_thumb_texture_from_template(lua, thumb, frame_name);
    }

    // Create button textures (NormalTexture, PushedTexture, etc.)
    if let Some(tex) = template.normal_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Normal", "SetNormalTexture");
    }
    if let Some(tex) = template.pushed_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Pushed", "SetPushedTexture");
    }
    if let Some(tex) = template.disabled_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Disabled", "SetDisabledTexture");
    }
    if let Some(tex) = template.highlight_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Highlight", "SetHighlightTexture");
    }
    if let Some(tex) = template.checked_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Checked", "SetCheckedTexture");
    }
    if let Some(tex) = template.disabled_checked_texture() {
        create_button_texture_from_template(
            lua,
            tex,
            frame_name,
            "DisabledChecked",
            "SetDisabledCheckedTexture",
        );
    }

    // Create child Frames from template
    create_child_frames(lua, template, frame_name);

    // Apply scripts from template
    if let Some(scripts) = template.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }
}

/// Apply mixin(s) to a frame. The mixin attribute can be comma-separated.
fn apply_mixin(lua: &Lua, mixin: &Option<String>, frame_name: &str) {
    let Some(mixin_str) = mixin else { return };
    if mixin_str.is_empty() {
        return;
    }

    // Build a Mixin() call with all mixin names
    let mixin_args: Vec<&str> = mixin_str.split(',').map(|s| s.trim()).collect();
    let args = mixin_args.join(", ");
    let code = format!(
        "do local f = {} if f then Mixin(f, {}) end end",
        frame_name, args
    );
    let _ = lua.load(&code).exec();
}

/// Fire OnLoad script on a frame after it's fully configured.
fn fire_on_load(lua: &Lua, frame_name: &str) {
    let code = format!(
        r#"
        local frame = {}
        if frame then
            local handler = frame:GetScript("OnLoad")
            if handler then
                handler(frame)
            elseif type(frame.OnLoad) == "function" then
                frame:OnLoad()
            end
        end
        "#,
        frame_name
    );
    let _ = lua.load(&code).exec();
}

/// Create child frames from a FrameXml's `<Frames>` section.
fn create_child_frames(lua: &Lua, frame: &FrameXml, parent_name: &str) {
    let Some(frames) = frame.frames() else {
        return;
    };
    for child in &frames.elements {
        let Some((child_frame, child_type)) = frame_element_type(child) else {
            continue;
        };
        create_child_frame_from_template(lua, child_frame, child_type, parent_name);
    }
}

/// Create a texture from template XML.
fn create_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
) {
    // Generate child name
    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    // Create the texture
    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local tex = parent:CreateTexture("{}", "{}")
        "#,
        parent_name, child_name, draw_layer,
    );

    // Apply size
    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            tex:SetSize({}, {})\n", w, h));
        }
    }

    // Apply texture file
    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            tex:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }

    // Apply atlas
    if let Some(atlas) = &texture.atlas {
        code.push_str(&format!(
            "            tex:SetAtlas(\"{}\")\n",
            escape_lua_string(atlas)
        ));
    }

    // Apply color
    if let Some(color) = &texture.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            tex:SetColorTexture({}, {}, {}, {})\n",
            r, g, b, a
        ));
    }

    // Apply anchors
    if let Some(anchors) = &texture.anchors {
        code.push_str(&generate_anchors_code_for_child(anchors, parent_name, "tex"));
    }

    // Apply setAllPoints
    if texture.set_all_points == Some(true) {
        code.push_str("            tex:SetAllPoints(true)\n");
    }

    // Apply parentKey
    if let Some(parent_key) = &texture.parent_key {
        code.push_str(&format!("            parent.{} = tex\n", parent_key));
    }

    // Register as global if named
    if texture.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", child_name));
    }

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();
}

/// Create a fontstring from template XML.
fn create_fontstring_from_template(
    lua: &Lua,
    fontstring: &crate::xml::FontStringXml,
    parent_name: &str,
    draw_layer: &str,
) {
    let child_name = fontstring
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__fs_{}", rand_id()));

    let inherits = fontstring.inherits.as_deref().unwrap_or("");

    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local fs = parent:CreateFontString("{}", "{}", {})
        "#,
        parent_name,
        child_name,
        draw_layer,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Apply size
    if let Some(size) = &fontstring.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            fs:SetSize({}, {})\n", w, h));
        }
    }

    // Apply text (resolve localization key via global strings)
    if let Some(text_key) = &fontstring.text {
        let resolved = crate::global_strings::get_global_string(text_key)
            .unwrap_or(text_key.as_str());
        code.push_str(&format!(
            "            fs:SetText(\"{}\")\n",
            escape_lua_string(resolved)
        ));
    }

    // Apply justifyH
    if let Some(justify_h) = &fontstring.justify_h {
        code.push_str(&format!("            fs:SetJustifyH(\"{}\")\n", justify_h));
    }

    // Apply justifyV
    if let Some(justify_v) = &fontstring.justify_v {
        code.push_str(&format!("            fs:SetJustifyV(\"{}\")\n", justify_v));
    }

    // Apply anchors
    if let Some(anchors) = &fontstring.anchors {
        code.push_str(&generate_anchors_code_for_child(anchors, parent_name, "fs"));
    }

    // Apply parentKey
    if let Some(parent_key) = &fontstring.parent_key {
        code.push_str(&format!("            parent.{} = fs\n", parent_key));
    }

    // Register as global if named
    if fontstring.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = fs\n", child_name));
    }

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();
}

/// Create a child frame from template XML.
fn create_child_frame_from_template(
    lua: &Lua,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_name: &str,
) {
    let child_name = frame
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__frame_{}", rand_id()));

    let inherits = frame.inherits.as_deref().unwrap_or("");

    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local child = CreateFrame("{}", "{}", parent, {})
        "#,
        parent_name,
        widget_type,
        child_name,
        if inherits.is_empty() {
            "nil".to_string()
        } else {
            format!("\"{}\"", inherits)
        }
    );

    // Apply size
    if let Some(size) = frame.size() {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            child:SetSize({}, {})\n", w, h));
        }
    }

    // Apply anchors
    if let Some(anchors) = frame.anchors() {
        code.push_str(&generate_anchors_code_for_child(
            anchors,
            parent_name,
            "child",
        ));
    }

    // Apply setAllPoints
    if frame.set_all_points == Some(true) {
        code.push_str("            child:SetAllPoints(true)\n");
    }

    // Apply hidden state
    if frame.hidden == Some(true) {
        code.push_str("            child:Hide()\n");
    }

    // Apply parentKey
    if let Some(parent_key) = &frame.parent_key {
        code.push_str(&format!("            parent.{} = child\n", parent_key));
    }

    // Register as global
    code.push_str(&format!("            _G[\"{}\"] = child\n", child_name));

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();

    // Apply inline content from the child FrameXml (layers, key values, scripts, etc.)
    // This handles elements defined directly on the child XML, not via inherits.
    apply_inline_frame_content(lua, frame, &child_name);

    // Fire OnLoad after all content (inherited + inline) is applied.
    // This must be after apply_inline_frame_content so KeyValues like
    // normalTexture are available when OnLoad runs.
    fire_on_load(lua, &child_name);
}

/// Apply inline content from a FrameXml to an already-created frame.
///
/// This handles layers (textures, fontstrings), key values, scripts, child frames,
/// button textures, and thumb textures that are defined directly in the XML element
/// rather than through an inherited template.
fn apply_inline_frame_content(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    // Apply mixin (must be before scripts)
    apply_mixin(lua, &frame.mixin, frame_name);

    // Apply KeyValues
    if let Some(key_values) = frame.key_values() {
        for kv in &key_values.values {
            let value = match kv.value_type.as_deref() {
                Some("number") => kv.value.clone(),
                Some("boolean") => kv.value.to_lowercase(),
                _ => format!("\"{}\"", escape_lua_string(&kv.value)),
            };
            let code = format!(
                "do local f = {} if f then f.{} = {} end end",
                frame_name, kv.key, value
            );
            let _ = lua.load(&code).exec();
        }
    }

    // Create textures from Layers
    for layers in frame.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");
            for texture in layer.textures() {
                create_texture_from_template(lua, texture, frame_name, draw_layer);
            }
            for fontstring in layer.font_strings() {
                create_fontstring_from_template(lua, fontstring, frame_name, draw_layer);
            }
        }
    }

    // Create ThumbTexture
    if let Some(thumb) = frame.thumb_texture() {
        create_thumb_texture_from_template(lua, thumb, frame_name);
    }

    // Create button textures
    if let Some(tex) = frame.normal_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Normal", "SetNormalTexture");
    }
    if let Some(tex) = frame.pushed_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Pushed", "SetPushedTexture");
    }
    if let Some(tex) = frame.disabled_texture() {
        create_button_texture_from_template(lua, tex, frame_name, "Disabled", "SetDisabledTexture");
    }
    if let Some(tex) = frame.highlight_texture() {
        create_button_texture_from_template(
            lua,
            tex,
            frame_name,
            "Highlight",
            "SetHighlightTexture",
        );
    }

    // Create nested child frames
    create_child_frames(lua, frame, frame_name);

    // Apply scripts (after children so OnLoad can reference them)
    if let Some(scripts) = frame.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }
}

/// Create a thumb texture from template XML (for sliders).
fn create_thumb_texture_from_template(
    lua: &Lua,
    thumb: &crate::xml::TextureXml,
    parent_name: &str,
) {
    let child_name = thumb
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__thumb_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.SetThumbTexture then
            local thumb = parent:CreateTexture("{}", "ARTWORK")
        "#,
        parent_name, child_name,
    );

    // Apply size
    if let Some(size) = &thumb.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            thumb:SetSize({}, {})\n", w, h));
        }
    }

    // Apply texture file
    if let Some(file) = &thumb.file {
        code.push_str(&format!(
            "            thumb:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }

    // Set as thumb texture and parentKey
    code.push_str("            parent:SetThumbTexture(thumb)\n");
    if let Some(parent_key) = &thumb.parent_key {
        code.push_str(&format!("            parent.{} = thumb\n", parent_key));
    } else {
        code.push_str("            parent.ThumbTexture = thumb\n");
    }

    // Register as global if named
    if thumb.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = thumb\n", child_name));
    }

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();
}

/// Create a button texture from template XML (NormalTexture, PushedTexture, etc.).
///
/// Reuses existing default texture children (from create_widget_type_defaults) when
/// available, to avoid orphaning them. Orphaned 0x0 anchorless children would center
/// themselves in the parent frame, causing ghost rectangles.
fn create_button_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    parent_key: &str,
    setter_method: &str,
) {
    let actual_parent_key = texture
        .parent_key
        .as_deref()
        .unwrap_or(parent_key);

    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name));

    // Reuse existing default texture child if available (e.g., from create_widget_type_defaults),
    // otherwise create a new one. This prevents orphaned children that render as ghost elements.
    let tex_name = child_name
        .clone()
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent and parent.{} then
            local tex = parent.{}
            if tex == nil then
                tex = parent:CreateTexture("{}", "ARTWORK")
            end
        "#,
        parent_name, setter_method, actual_parent_key, tex_name,
    );

    // Apply size
    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            tex:SetSize({}, {})\n", w, h));
        }
    }

    // Register parentKey so children_keys is populated, then call setter
    // method which uses get_or_create_button_texture (finds existing child), then
    // SetAtlas can propagate to parent via the parent_key lookup
    code.push_str(&format!("            parent.{} = tex\n", actual_parent_key));
    code.push_str(&format!("            parent:{}(tex)\n", setter_method));

    // Apply texture file
    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            tex:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }

    // Apply atlas (after parentKey registration so SetAtlas can propagate to parent)
    if let Some(atlas) = &texture.atlas {
        code.push_str(&format!(
            "            tex:SetAtlas(\"{}\")\n",
            escape_lua_string(atlas)
        ));
    }

    // Register as global if named
    if child_name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", tex_name));
    }

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();
}

/// Apply scripts from template.
fn apply_scripts_from_template(lua: &Lua, scripts: &crate::xml::ScriptsXml, frame_name: &str) {
    let mut code = format!(
        r#"
        local frame = {}
        if frame then
        "#,
        frame_name
    );

    // Helper to add script handler
    let add_handler = |code: &mut String, handler_name: &str, script: &crate::xml::ScriptBodyXml| {
        if let Some(func) = &script.function {
            if func.is_empty() {
                // Empty function means no-op (used to override parent scripts)
                return;
            }
            code.push_str(&format!(
                "            frame:SetScript(\"{}\", {})\n",
                handler_name, func
            ));
        } else if let Some(method) = &script.method {
            code.push_str(&format!(
                "            frame:SetScript(\"{}\", function(self, ...) self:{}(...) end)\n",
                handler_name, method
            ));
        } else if let Some(body) = &script.body {
            let body = body.trim();
            if !body.is_empty() {
                code.push_str(&format!(
                    "            frame:SetScript(\"{}\", function(self, ...)\n                {}\n            end)\n",
                    handler_name, body
                ));
            }
        }
    };

    if let Some(on_load) = scripts.on_load.last() {
        add_handler(&mut code, "OnLoad", on_load);
    }
    if let Some(on_event) = scripts.on_event.last() {
        add_handler(&mut code, "OnEvent", on_event);
    }
    if let Some(on_update) = scripts.on_update.last() {
        add_handler(&mut code, "OnUpdate", on_update);
    }
    if let Some(on_click) = scripts.on_click.last() {
        add_handler(&mut code, "OnClick", on_click);
    }
    if let Some(on_show) = scripts.on_show.last() {
        add_handler(&mut code, "OnShow", on_show);
    }
    if let Some(on_hide) = scripts.on_hide.last() {
        add_handler(&mut code, "OnHide", on_hide);
    }

    code.push_str("        end\n");

    let _ = lua.load(&code).exec();
}

/// Generate Lua code for anchors on a child element.
fn generate_anchors_code_for_child(
    anchors: &crate::xml::AnchorsXml,
    parent_name: &str,
    var_name: &str,
) -> String {
    let mut code = String::new();
    for anchor in &anchors.anchors {
        let point = &anchor.point;
        let relative_to = anchor.relative_to.as_deref();
        let relative_key = anchor.relative_key.as_deref();
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

        let rel_str = if let Some(key) = relative_key {
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
                if expr.is_empty() {
                    "parent".to_string()
                } else {
                    expr
                }
            } else {
                key.to_string()
            }
        } else {
            match relative_to {
                Some("$parent") => "parent".to_string(),
                Some(rel) if rel.contains("$parent") || rel.contains("$Parent") => rel
                    .replace("$parent", parent_name)
                    .replace("$Parent", parent_name),
                Some(rel) => rel.to_string(),
                None => "nil".to_string(),
            }
        };

        code.push_str(&format!(
            "            {}:SetPoint(\"{}\", {}, \"{}\", {}, {})\n",
            var_name, point, rel_str, relative_point, x, y
        ));
    }
    code
}

/// Get size values from a SizeXml.
fn get_size_values(size: &crate::xml::SizeXml) -> (Option<f32>, Option<f32>) {
    if size.x.is_some() || size.y.is_some() {
        (size.x, size.y)
    } else if let Some(abs) = &size.abs_dimension {
        (abs.x, abs.y)
    } else {
        (None, None)
    }
}

/// Escape a string for use in Lua code.
fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Generate a simple random ID.
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}
