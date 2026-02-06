//! Template application from the XML template registry.
//!
//! This module provides functionality to apply XML templates from the registry
//! when CreateFrame is called with a template name.

use crate::loader::helpers::generate_set_point_code;
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
        FrameElement::Model(f) => Some((f, "Model")),
        FrameElement::ModelScene(f) => Some((f, "ModelScene")),
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

    let mut all_child_names = Vec::new();
    for entry in &chain {
        let child_names = apply_single_template(lua, frame_name, entry);
        all_child_names.extend(child_names);
    }

    // Fire OnLoad for all child frames created during template application.
    // This is deferred until after ALL templates so that child OnLoad handlers
    // can access KeyValues from any template in the chain (not just earlier ones).
    // Example: Controller's OnLoad calls parent:InitButton() which needs atlasName
    // from BigRedThreeSliceButtonTemplate, a later template in the chain.
    for child_name in &all_child_names {
        fire_on_load(lua, child_name);
    }

    // NOTE: Do NOT fire parent OnLoad here. For XML-created frames, xml_frame.rs
    // fires OnLoad after all layers/children/textures are processed. For Lua
    // CreateFrame() calls, WoW does not fire OnLoad at all.
}

/// Apply a single template's children to a frame.
/// Returns names of child frames created (for deferred OnLoad).
fn apply_single_template(lua: &Lua, frame_name: &str, entry: &TemplateEntry) -> Vec<String> {
    let template = &entry.frame;

    apply_template_size(lua, template, frame_name);
    apply_template_anchors(lua, template, frame_name);
    apply_template_set_all_points(lua, template, frame_name);

    // Apply mixin from template (must be before scripts)
    apply_mixin(lua, &template.combined_mixin(), frame_name);

    apply_key_values(lua, template.key_values(), frame_name);
    apply_layers(lua, template, frame_name);

    // Create ThumbTexture for sliders
    if let Some(thumb) = template.thumb_texture() {
        create_thumb_texture_from_template(lua, thumb, frame_name);
    }

    apply_button_textures(lua, template, frame_name);

    // Create child Frames from template
    let child_names = create_child_frames(lua, template, frame_name);

    // Apply scripts from template
    if let Some(scripts) = template.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }

    child_names
}

/// Apply size from template if frame has no size yet.
fn apply_template_size(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let Some(size) = template.size() else { return };
    let (width, height) = get_size_values(size);
    let (Some(w), Some(h)) = (width, height) else {
        return;
    };
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

/// Apply anchors from template (only if frame has no anchors yet).
fn apply_template_anchors(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let Some(anchors) = template.anchors() else {
        return;
    };

    let mut code = format!(
        r#"
        local frame = {}
        if frame and frame.GetNumPoints and frame:GetNumPoints() == 0 then
        "#,
        frame_name
    );
    for anchor in &anchors.anchors {
        let point = &anchor.point;
        let relative_point = anchor.relative_point.as_deref().unwrap_or(point.as_str());
        let (x, y) = anchor_offset(anchor);
        let rel_str = anchor_relative_to(anchor, frame_name);
        code.push_str(&format!(
            "                frame:SetPoint(\"{}\", {}, \"{}\", {}, {})\n",
            point, rel_str, relative_point, x, y
        ));
    }
    code.push_str("            end\n");
    let _ = lua.load(&code).exec();
}

/// Extract offset (x, y) from an anchor element.
fn anchor_offset(anchor: &crate::xml::AnchorXml) -> (f32, f32) {
    if let Some(offset) = &anchor.offset {
        if let Some(abs) = &offset.abs_dimension {
            return (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0));
        }
        return (0.0, 0.0);
    }
    (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
}

/// Get the Lua expression for an anchor's relativeTo.
fn anchor_relative_to(anchor: &crate::xml::AnchorXml, frame_name: &str) -> String {
    match anchor.relative_to.as_deref() {
        Some("$parent") => format!("{}:GetParent()", frame_name),
        Some(rel) => rel.to_string(),
        None => "nil".to_string(),
    }
}

/// Apply setAllPoints from template (only if frame has no anchors yet).
fn apply_template_set_all_points(lua: &Lua, template: &FrameXml, frame_name: &str) {
    if template.set_all_points != Some(true) {
        return;
    }
    let code = format!(
        r#"
        local frame = {}
        if frame and frame.GetNumPoints and frame:GetNumPoints() == 0 then
            frame:SetAllPoints(true)
        end
        "#,
        frame_name
    );
    let _ = lua.load(&code).exec();
}

/// Apply KeyValues to a frame from template or inline XML.
fn apply_key_values(
    lua: &Lua,
    key_values: Option<&crate::xml::KeyValuesXml>,
    frame_name: &str,
) {
    let Some(key_values) = key_values else { return };
    for kv in &key_values.values {
        let value = format_key_value(&kv.value, kv.value_type.as_deref());
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

/// Format a KeyValue value based on its type for Lua emission.
fn format_key_value(value: &str, value_type: Option<&str>) -> String {
    match value_type {
        Some("number") => value.to_string(),
        Some("boolean") => value.to_lowercase(),
        Some("global") => format!("_G[\"{}\"]", escape_lua_string(value)),
        _ => format!("\"{}\"", escape_lua_string(value)),
    }
}

/// Create textures and fontstrings from template layers.
fn apply_layers(lua: &Lua, template: &FrameXml, frame_name: &str) {
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
}

/// Apply standard button textures (Normal, Pushed, Disabled, Highlight, Checked).
fn apply_button_textures(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let texture_specs: &[(&str, &str, Option<&crate::xml::TextureXml>)] = &[
        ("Normal", "SetNormalTexture", template.normal_texture()),
        ("Pushed", "SetPushedTexture", template.pushed_texture()),
        ("Disabled", "SetDisabledTexture", template.disabled_texture()),
        ("Highlight", "SetHighlightTexture", template.highlight_texture()),
        ("Checked", "SetCheckedTexture", template.checked_texture()),
        (
            "DisabledChecked",
            "SetDisabledCheckedTexture",
            template.disabled_checked_texture(),
        ),
    ];
    for &(parent_key, setter, tex_opt) in texture_specs {
        if let Some(tex) = tex_opt {
            create_button_texture_from_template(lua, tex, frame_name, parent_key, setter);
        }
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
/// Returns the names of created child frames (for deferred OnLoad).
fn create_child_frames(lua: &Lua, frame: &FrameXml, parent_name: &str) -> Vec<String> {
    let mut child_names = Vec::new();
    let Some(frames) = frame.frames() else {
        return child_names;
    };
    for child in &frames.elements {
        let Some((child_frame, child_type)) = frame_element_type(child) else {
            continue;
        };
        let name = create_child_frame_from_template(lua, child_frame, child_type, parent_name);
        child_names.push(name);
    }
    child_names
}

/// Create a texture from template XML.
fn create_texture_from_template(
    lua: &Lua,
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    draw_layer: &str,
) {
    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local tex = parent:CreateTexture("{}", "{}")
        "#,
        parent_name, child_name, draw_layer,
    );

    append_texture_properties(&mut code, texture, "tex");
    append_anchors_and_parent_key(&mut code, &texture.anchors, texture.set_all_points, &texture.parent_key, "tex", "parent", parent_name);

    // Register as global if named
    if texture.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", child_name));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();
}

/// Append texture-specific property setters (size, file, atlas, color) to Lua code.
fn append_texture_properties(code: &mut String, texture: &crate::xml::TextureXml, var: &str) {
    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            {}:SetSize({}, {})\n", var, w, h));
        }
    }
    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            {}:SetTexture(\"{}\")\n",
            var,
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &texture.atlas {
        code.push_str(&format!(
            "            {}:SetAtlas(\"{}\")\n",
            var,
            escape_lua_string(atlas)
        ));
    }
    if let Some(color) = &texture.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            {}:SetColorTexture({}, {}, {}, {})\n",
            var, r, g, b, a
        ));
    }
}

/// Append anchors, setAllPoints, and parentKey assignment to Lua code.
fn append_anchors_and_parent_key(
    code: &mut String,
    anchors: &Option<crate::xml::AnchorsXml>,
    set_all_points: Option<bool>,
    parent_key: &Option<String>,
    var: &str,
    parent_var: &str,
    parent_name: &str,
) {
    if let Some(anchors) = anchors {
        code.push_str(&generate_set_point_code(anchors, var, parent_var, parent_name, "nil"));
    }
    if set_all_points == Some(true) {
        code.push_str(&format!("            {}:SetAllPoints(true)\n", var));
    }
    if let Some(parent_key) = parent_key {
        code.push_str(&format!("            {}.{} = {}\n", parent_var, parent_key, var));
    }
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

    append_fontstring_size_and_text(&mut code, fontstring);
    append_fontstring_justify_and_color(&mut code, fontstring);
    append_fontstring_shadow(&mut code, fontstring);
    append_anchors_and_parent_key(
        &mut code,
        &fontstring.anchors,
        fontstring.set_all_points,
        &fontstring.parent_key,
        "fs",
        "parent",
        parent_name,
    );
    append_fontstring_wrap_and_lines(&mut code, fontstring);

    // Register as global if named
    if fontstring.name.is_some() {
        code.push_str(&format!("            _G[\"{}\"] = fs\n", child_name));
    }

    code.push_str("        end\n");
    let _ = lua.load(&code).exec();
}

/// Append size and text setters for a fontstring.
fn append_fontstring_size_and_text(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(size) = fs.size.last() {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            fs:SetSize({}, {})\n", w, h));
        }
    }
    if let Some(text_key) = &fs.text {
        let resolved = crate::global_strings::get_global_string(text_key)
            .unwrap_or(text_key.as_str());
        code.push_str(&format!(
            "            fs:SetText(\"{}\")\n",
            escape_lua_string(resolved)
        ));
    }
}

/// Append justification and text color for a fontstring.
fn append_fontstring_justify_and_color(code: &mut String, fs: &crate::xml::FontStringXml) {
    if let Some(justify_h) = &fs.justify_h {
        code.push_str(&format!("            fs:SetJustifyH(\"{}\")\n", justify_h));
    }
    if let Some(justify_v) = &fs.justify_v {
        code.push_str(&format!("            fs:SetJustifyV(\"{}\")\n", justify_v));
    }
    if let Some(color) = &fs.color {
        let r = color.r.unwrap_or(1.0);
        let g = color.g.unwrap_or(1.0);
        let b = color.b.unwrap_or(1.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            fs:SetTextColor({}, {}, {}, {})\n",
            r, g, b, a
        ));
    }
}

/// Append shadow offset and color for a fontstring.
fn append_fontstring_shadow(code: &mut String, fs: &crate::xml::FontStringXml) {
    let Some(shadow) = &fs.shadow else { return };
    if let Some(offset) = &shadow.offset {
        let x = offset.x();
        let y = offset.y();
        code.push_str(&format!("            fs:SetShadowOffset({}, {})\n", x, y));
    }
    if let Some(color) = &shadow.color {
        let r = color.r.unwrap_or(0.0);
        let g = color.g.unwrap_or(0.0);
        let b = color.b.unwrap_or(0.0);
        let a = color.a.unwrap_or(1.0);
        code.push_str(&format!(
            "            fs:SetShadowColor({}, {}, {}, {})\n",
            r, g, b, a
        ));
    }
}

/// Append wordWrap and maxLines for a fontstring.
fn append_fontstring_wrap_and_lines(code: &mut String, fs: &crate::xml::FontStringXml) {
    if fs.word_wrap == Some(false) {
        code.push_str("            fs:SetWordWrap(false)\n");
    }
    if let Some(max_lines) = fs.max_lines {
        if max_lines > 0 {
            code.push_str(&format!("            fs:SetMaxLines({})\n", max_lines));
        }
    }
}

/// Create a child frame from template XML.
fn create_child_frame_from_template(
    lua: &Lua,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_name: &str,
) -> String {
    let child_name = frame
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__frame_{}", rand_id()));

    let code = build_create_child_code(frame, widget_type, parent_name, &child_name);
    let _ = lua.load(&code).exec();

    // Apply inline content from the child FrameXml (layers, key values, scripts, etc.)
    // This handles elements defined directly on the child XML, not via inherits.
    apply_inline_frame_content(lua, frame, &child_name);

    // NOTE: Do NOT fire OnLoad here. Child OnLoad is deferred until after ALL templates
    // in the chain are applied by apply_templates_from_registry. This is required because
    // child OnLoad handlers (e.g. ButtonControllerMixin:OnLoad -> InitButton) may depend on
    // KeyValues from later templates in the chain (e.g. atlasName from BigRedThreeSliceButtonTemplate).

    child_name
}

/// Build Lua code to create a child frame with size, anchors, visibility, and parentKey.
fn build_create_child_code(
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_name: &str,
    child_name: &str,
) -> String {
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

    if let Some(size) = frame.size() {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            child:SetSize({}, {})\n", w, h));
        }
    }
    if let Some(anchors) = frame.anchors() {
        code.push_str(&generate_set_point_code(anchors, "child", "parent", parent_name, "nil"));
    }
    if frame.set_all_points == Some(true) {
        code.push_str("            child:SetAllPoints(true)\n");
    }
    if frame.hidden == Some(true) {
        code.push_str("            child:Hide()\n");
    }
    if let Some(parent_key) = &frame.parent_key {
        code.push_str(&format!("            parent.{} = child\n", parent_key));
    }
    code.push_str(&format!("            _G[\"{}\"] = child\n", child_name));
    code.push_str("        end\n");
    code
}

/// Apply inline content from a FrameXml to an already-created frame.
///
/// This handles layers (textures, fontstrings), key values, scripts, child frames,
/// button textures, and thumb textures that are defined directly in the XML element
/// rather than through an inherited template.
fn apply_inline_frame_content(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    // Apply mixin (must be before scripts)
    apply_mixin(lua, &frame.combined_mixin(), frame_name);

    apply_inline_key_values(lua, frame, frame_name);
    apply_layers(lua, frame, frame_name);

    // Create ThumbTexture
    if let Some(thumb) = frame.thumb_texture() {
        create_thumb_texture_from_template(lua, thumb, frame_name);
    }

    apply_inline_button_textures(lua, frame, frame_name);

    // Create nested child frames
    create_child_frames(lua, frame, frame_name);

    // Apply scripts (after children so OnLoad can reference them)
    if let Some(scripts) = frame.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }
}

/// Apply KeyValues from inline frame content.
fn apply_inline_key_values(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let Some(key_values) = frame.key_values() else {
        return;
    };
    for kv in &key_values.values {
        let value = format_key_value(&kv.value, kv.value_type.as_deref());
        let code = format!(
            "do local f = {} if f then f.{} = {} end end",
            frame_name, kv.key, value
        );
        let _ = lua.load(&code).exec();
    }
}

/// Apply button textures from inline frame content.
fn apply_inline_button_textures(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let texture_specs: &[(&str, &str, Option<&crate::xml::TextureXml>)] = &[
        ("Normal", "SetNormalTexture", frame.normal_texture()),
        ("Pushed", "SetPushedTexture", frame.pushed_texture()),
        ("Disabled", "SetDisabledTexture", frame.disabled_texture()),
        ("Highlight", "SetHighlightTexture", frame.highlight_texture()),
    ];
    for &(parent_key, setter, tex_opt) in texture_specs {
        if let Some(tex) = tex_opt {
            create_button_texture_from_template(lua, tex, frame_name, parent_key, setter);
        }
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

    if let Some(size) = &thumb.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            thumb:SetSize({}, {})\n", w, h));
        }
    }
    if let Some(file) = &thumb.file {
        code.push_str(&format!(
            "            thumb:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }

    code.push_str("            parent:SetThumbTexture(thumb)\n");
    if let Some(parent_key) = &thumb.parent_key {
        code.push_str(&format!("            parent.{} = thumb\n", parent_key));
    } else {
        code.push_str("            parent.ThumbTexture = thumb\n");
    }

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
    let default_parent_key = format!("{}Texture", parent_key);
    let actual_parent_key = texture
        .parent_key
        .as_deref()
        .unwrap_or(&default_parent_key);

    let child_name = texture
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name));

    let tex_name = child_name
        .clone()
        .unwrap_or_else(|| format!("__tex_{}", rand_id()));

    let code = build_button_texture_code(
        texture,
        parent_name,
        setter_method,
        actual_parent_key,
        &tex_name,
        child_name.is_some(),
    );
    let _ = lua.load(&code).exec();
}

/// Build Lua code to create or reuse a button texture with properties applied.
fn build_button_texture_code(
    texture: &crate::xml::TextureXml,
    parent_name: &str,
    setter_method: &str,
    actual_parent_key: &str,
    tex_name: &str,
    is_named: bool,
) -> String {
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

    if let Some(size) = &texture.size {
        let (width, height) = get_size_values(size);
        if let (Some(w), Some(h)) = (width, height) {
            code.push_str(&format!("            tex:SetSize({}, {})\n", w, h));
        }
    }

    // Register parentKey and call setter method
    code.push_str(&format!("            parent.{} = tex\n", actual_parent_key));
    code.push_str(&format!("            parent:{}(tex)\n", setter_method));

    if let Some(file) = &texture.file {
        code.push_str(&format!(
            "            tex:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &texture.atlas {
        let use_atlas_size = texture.use_atlas_size.unwrap_or(false);
        code.push_str(&format!(
            "            tex:SetAtlas(\"{}\", {})\n",
            escape_lua_string(atlas),
            use_atlas_size
        ));
    }

    if is_named {
        code.push_str(&format!("            _G[\"{}\"] = tex\n", tex_name));
    }

    code.push_str("        end\n");
    code
}

/// Apply scripts from template.
fn apply_scripts_from_template(lua: &Lua, scripts: &crate::xml::ScriptsXml, frame_name: &str) {
    use crate::loader::helpers::apply_script_handlers;

    let handlers_code = apply_script_handlers("frame", &[
        ("OnLoad", scripts.on_load.last()),
        ("OnEvent", scripts.on_event.last()),
        ("OnUpdate", scripts.on_update.last()),
        ("OnClick", scripts.on_click.last()),
        ("OnShow", scripts.on_show.last()),
        ("OnHide", scripts.on_hide.last()),
    ]);

    if !handlers_code.is_empty() {
        let code = format!(
            "\n        local frame = {frame_name}\n        if frame then\n        {handlers_code}\n        end\n"
        );
        let _ = lua.load(&code).exec();
    }
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
