//! Template application from the XML template registry.
//!
//! This module provides functionality to apply XML templates from the registry
//! when CreateFrame is called with a template name.

mod elements;

use crate::loader::helpers::{generate_animation_group_code, generate_set_point_code};
use crate::xml::{get_template_chain, FrameElement, FrameXml, TemplateEntry};
use mlua::Lua;

/// Extract the FrameXml and widget type string from a FrameElement.
fn frame_element_type(element: &FrameElement) -> Option<(&FrameXml, &'static str)> {
    match element {
        FrameElement::Frame(f) => Some((f, "Frame")),
        FrameElement::Button(f)
        | FrameElement::DropdownButton(f)
        | FrameElement::DropDownToggleButton(f)
        | FrameElement::EventButton(f) => Some((f, "Button")),
        FrameElement::ItemButton(f) => Some((f, "ItemButton")),
        FrameElement::CheckButton(f) => Some((f, "CheckButton")),
        FrameElement::EditBox(f)
        | FrameElement::EventEditBox(f) => Some((f, "EditBox")),
        FrameElement::ScrollFrame(f)
        | FrameElement::EventScrollFrame(f) => Some((f, "ScrollFrame")),
        FrameElement::Slider(f) => Some((f, "Slider")),
        FrameElement::StatusBar(f) => Some((f, "StatusBar")),
        FrameElement::Cooldown(f) => Some((f, "Cooldown")),
        FrameElement::GameTooltip(f) => Some((f, "GameTooltip")),
        FrameElement::ColorSelect(f) => Some((f, "ColorSelect")),
        FrameElement::Model(f)
        | FrameElement::DressUpModel(f) => Some((f, "Model")),
        FrameElement::ModelScene(f) => Some((f, "ModelScene")),
        FrameElement::PlayerModel(f)
        | FrameElement::CinematicModel(f) => Some((f, "PlayerModel")),
        FrameElement::MessageFrame(f)
        | FrameElement::ScrollingMessageFrame(f) => Some((f, "MessageFrame")),
        FrameElement::SimpleHTML(f) => Some((f, "SimpleHTML")),
        FrameElement::EventFrame(f)
        | FrameElement::TaxiRouteFrame(f)
        | FrameElement::ModelFFX(f)
        | FrameElement::TabardModel(f)
        | FrameElement::UiCamera(f)
        | FrameElement::UnitPositionFrame(f)
        | FrameElement::OffScreenFrame(f)
        | FrameElement::Checkout(f)
        | FrameElement::FogOfWarFrame(f)
        | FrameElement::QuestPOIFrame(f)
        | FrameElement::ArchaeologyDigSiteFrame(f)
        | FrameElement::ScenarioPOIFrame(f)
        | FrameElement::UIThemeContainerFrame(f)
        | FrameElement::ContainedAlertFrame(f)
        | FrameElement::MapScene(f)
        | FrameElement::Line(f)
        | FrameElement::Browser(f)
        | FrameElement::Minimap(f)
        | FrameElement::MovieFrame(f)
        | FrameElement::WorldFrame(f) => Some((f, "Frame")),
        FrameElement::ScopedModifier(_) => None,
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
    // This is deferred until after ALL templates in the chain are applied,
    // because child OnLoad handlers may depend on KeyValues from later templates.
    for child_name in &all_child_names {
        fire_on_load(lua, child_name);
    }
}

/// Apply a single template entry to a frame, returning names of created children.
fn apply_single_template(lua: &Lua, frame_name: &str, entry: &TemplateEntry) -> Vec<String> {
    let template = &entry.frame;

    // Apply mixin (must be before children and scripts)
    apply_mixin(lua, &template.combined_mixin(), frame_name);

    // Apply size from template
    apply_template_size(lua, template, frame_name);

    // Apply anchors from template
    apply_template_anchors(lua, template, frame_name);

    // Apply SetAllPoints from template
    apply_template_set_all_points(lua, template, frame_name);

    // Apply key values from template
    if let Some(key_values) = template.key_values() {
        apply_key_values(lua, key_values, frame_name);
    }

    // Apply layers (textures and fontstrings)
    apply_layers(lua, template, frame_name);

    // Apply button textures (NormalTexture, PushedTexture, etc.)
    apply_button_textures(lua, template, frame_name);

    // Apply StatusBar BarTexture
    if let Some(bar) = template.bar_texture() {
        elements::create_bar_texture_from_template(lua, bar, frame_name);
    }

    // Apply Slider ThumbTexture
    if let Some(thumb) = template.thumb_texture() {
        elements::create_thumb_texture_from_template(lua, thumb, frame_name);
    }

    // Apply ButtonText and EditBox FontString
    apply_button_text(lua, template, frame_name);
    apply_editbox_fontstring(lua, template, frame_name);

    // Create child frames defined in the template
    let mut child_names = create_child_frames(lua, template, frame_name);

    // Create ScrollChild children
    if let Some(scroll_child) = template.scroll_child() {
        child_names.extend(create_scroll_child_frames(lua, &scroll_child.children, frame_name));
    }

    // Apply scripts from template (after children, so OnLoad can reference them)
    if let Some(scripts) = template.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }

    child_names
}

/// Apply size from a template to a frame.
fn apply_template_size(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let Some(size) = template.size() else { return };
    let (width, height) = get_size_values(size);
    let (Some(w), Some(h)) = (width, height) else { return };
    let code = format!(
        "do local f = {} if f then f:SetSize({}, {}) end end",
        frame_name, w, h
    );
    let _ = lua.load(&code).exec();
}

/// Apply anchors from a template to a frame.
fn apply_template_anchors(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let Some(anchors) = template.anchors() else { return };
    for anchor in &anchors.anchors {
        let (offset_x, offset_y) = anchor_offset(anchor);
        let relative_to = anchor_relative_to(anchor, frame_name);
        let point = anchor.point.as_str();
        let relative_point = anchor.relative_point.as_deref().unwrap_or(point);
        let code = format!(
            "do local f = {} if f then f:SetPoint(\"{}\", {}, \"{}\", {}, {}) end end",
            frame_name, point, relative_to, relative_point, offset_x, offset_y
        );
        let _ = lua.load(&code).exec();
    }
}

/// Extract offset values from an anchor.
fn anchor_offset(anchor: &crate::xml::AnchorXml) -> (f32, f32) {
    if let Some(offset) = &anchor.offset {
        let abs = offset.abs_dimension.as_ref();
        (
            abs.and_then(|d| d.x).unwrap_or(0.0),
            abs.and_then(|d| d.y).unwrap_or(0.0),
        )
    } else {
        (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
    }
}

/// Determine the relativeTo target for an anchor.
fn anchor_relative_to(anchor: &crate::xml::AnchorXml, frame_name: &str) -> String {
    match &anchor.relative_to {
        Some(rel) if rel == "$parent" => {
            format!("({} and {}:GetParent())", frame_name, frame_name)
        }
        Some(rel) => rel.clone(),
        None => format!("({} and {}:GetParent())", frame_name, frame_name),
    }
}

/// Apply SetAllPoints from a template to a frame.
fn apply_template_set_all_points(lua: &Lua, template: &FrameXml, frame_name: &str) {
    if template.set_all_points != Some(true) {
        return;
    }
    let code = format!(
        "do local f = {} if f then f:SetAllPoints(true) end end",
        frame_name
    );
    let _ = lua.load(&code).exec();
}

/// Apply key values from a template to a frame.
fn apply_key_values(
    lua: &Lua,
    key_values: &crate::xml::KeyValuesXml,
    frame_name: &str,
) {
    for kv in &key_values.values {
        let value = format_key_value(&kv.value, kv.value_type.as_deref());
        let code = format!(
            "do local f = {} if f then f.{} = {} end end",
            frame_name, kv.key, value
        );
        let _ = lua.load(&code).exec();
    }
}

/// Format a key value for Lua assignment.
fn format_key_value(value: &str, value_type: Option<&str>) -> String {
    match value_type {
        Some("number") => value.to_string(),
        Some("boolean") => value.to_lowercase(),
        Some("global") => value.to_string(),
        _ => format!("\"{}\"", escape_lua_string(value)),
    }
}

/// Apply layers (textures and fontstrings) from a template.
fn apply_layers(lua: &Lua, template: &FrameXml, frame_name: &str) {
    for layers in template.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");
            for (texture, is_mask) in layer.textures() {
                elements::create_texture_from_template(lua, texture, frame_name, draw_layer, is_mask);
            }
            for fontstring in layer.font_strings() {
                elements::create_fontstring_from_template(lua, fontstring, frame_name, draw_layer);
            }
        }
    }
}

/// Apply button textures (NormalTexture, PushedTexture, etc.) from a template.
fn apply_button_textures(lua: &Lua, template: &FrameXml, frame_name: &str) {
    let texture_specs: &[(&str, &str, Option<&crate::xml::TextureXml>)] = &[
        ("Normal", "SetNormalTexture", template.normal_texture()),
        ("Pushed", "SetPushedTexture", template.pushed_texture()),
        ("Disabled", "SetDisabledTexture", template.disabled_texture()),
        ("Highlight", "SetHighlightTexture", template.highlight_texture()),
        ("Checked", "SetCheckedTexture", template.checked_texture()),
        ("DisabledChecked", "SetDisabledCheckedTexture", template.disabled_checked_texture()),
    ];
    for &(parent_key, setter, tex_opt) in texture_specs {
        if let Some(tex) = tex_opt {
            elements::create_button_texture_from_template(lua, tex, frame_name, parent_key, setter);
        }
    }
}

/// Apply mixin to a frame.
fn apply_mixin(lua: &Lua, mixin: &Option<String>, frame_name: &str) {
    let Some(mixin) = mixin else { return };
    let mut parts = Vec::new();
    for name in mixin.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(format!("if {} then Mixin(f, {}) end", name, name));
    }
    if parts.is_empty() {
        return;
    }
    // WoW's C++ engine pre-initializes certain fields that OnLoad methods
    // expect to exist before they run. Simulate this for ActionBarMixin.
    let mut post_init = String::new();
    for name in mixin.split(',').map(str::trim) {
        if name == "ActionBarMixin" {
            post_init.push_str("f.actionButtons = f.actionButtons or {} ");
            post_init.push_str("f.shownButtonContainers = f.shownButtonContainers or {} ");
        }
        if name == "EditModeSystemMixin" {
            // Pre-initialize "Base" method aliases that OnSystemLoad normally sets up.
            // Frames with inherit="prepend" OnLoad handlers (e.g. StanceBar) may call
            // methods depending on these aliases before OnSystemLoad runs.
            post_init.push_str("f.SetScaleBase = f.SetScale ");
            post_init.push_str("f.SetPointBase = f.SetPoint ");
            post_init.push_str("f.ClearAllPointsBase = f.ClearAllPoints ");
            post_init.push_str("f.SetShownBase = f.SetShown ");
            post_init.push_str("f.ShowBase = f.Show ");
            post_init.push_str("f.HideBase = f.Hide ");
            post_init.push_str("f.IsShownBase = f.IsShown ");
        }
    }
    let code = format!(
        "do local f = {} if f then {} {} end end",
        frame_name,
        parts.join(" "),
        post_init,
    );
    let _ = lua.load(&code).exec();
}

/// Fire OnLoad on a created child frame.
///
/// Checks both `GetScript("OnLoad")` (set via SetScript in XML) and `frame.OnLoad`
/// (set via mixin), matching the behavior of `fire_lifecycle_scripts` in xml_frame.rs.
fn fire_on_load(lua: &Lua, frame_name: &str) {
    let code = format!(
        r#"
        local frame = {0}
        if frame then
            local handler = frame:GetScript("OnLoad")
            if handler then
                local ok, err = pcall(handler, frame)
                if not ok then
                    print("[fire_on_load] {0} error: " .. tostring(err))
                end
            elseif type(frame.OnLoad) == "function" then
                local ok, err = pcall(frame.OnLoad, frame)
                if not ok then
                    print("[fire_on_load] {0} error: " .. tostring(err))
                end
            end
        end
        "#,
        frame_name
    );
    let _ = lua.load(&code).exec();
}

/// Create child frames from template XML.
fn create_child_frames(lua: &Lua, frame: &FrameXml, parent_name: &str) -> Vec<String> {
    let mut all_names = Vec::new();
    for child in frame.all_frame_elements() {
        let Some((child_frame, child_type)) = frame_element_type(child) else {
            continue;
        };
        let names = create_child_frame_from_template(lua, child_frame, child_type, parent_name);
        all_names.extend(names);
    }
    all_names
}

/// Create a child frame from template XML.
/// Returns names of this frame AND all nested descendants (for deferred OnLoad).
fn create_child_frame_from_template(
    lua: &Lua,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_name: &str,
) -> Vec<String> {
    let child_name = frame
        .name
        .as_ref()
        .map(|n| n.replace("$parent", parent_name))
        .unwrap_or_else(|| format!("__tpl_{}", rand_id()));

    let code = build_create_child_code(frame, widget_type, parent_name, &child_name);
    if let Err(e) = lua.load(&code).exec() {
        eprintln!(
            "[template] Failed to create child '{}' (type={}) under '{}': {}",
            child_name, widget_type, parent_name, e
        );
    }

    let nested_names = apply_inline_frame_content(lua, frame, &child_name);

    let mut all_names = nested_names;
    all_names.push(child_name);
    all_names
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

    append_child_size_and_anchors(&mut code, frame, parent_name);
    append_child_parent_refs(&mut code, frame);
    code.push_str(&format!("            _G[\"{}\"] = child\n", child_name));
    code.push_str("        end\n");
    code
}

/// Append size, anchors, setAllPoints, and hidden to child frame code.
fn append_child_size_and_anchors(code: &mut String, frame: &FrameXml, parent_name: &str) {
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
}

/// Append parentKey and parentArray assignment (with template chain resolution).
fn append_child_parent_refs(code: &mut String, frame: &FrameXml) {
    if let Some(parent_key) = &resolve_inherited_field(frame, |f| f.parent_key.as_ref()) {
        code.push_str(&format!("            parent.{} = child\n", parent_key));
    }
    if let Some(parent_array) = &resolve_inherited_field(frame, |f| f.parent_array.as_ref()) {
        code.push_str(&format!(
            "            parent.{parent_array} = parent.{parent_array} or {{}}\n\
             table.insert(parent.{parent_array}, child)\n"
        ));
    }
}

/// Resolve a field from a frame or its inherited template chain.
fn resolve_inherited_field(
    frame: &FrameXml,
    getter: impl Fn(&FrameXml) -> Option<&String>,
) -> Option<String> {
    if let Some(val) = getter(frame) {
        return Some(val.clone());
    }
    let inherits = frame.inherits.as_deref().unwrap_or("");
    if inherits.is_empty() {
        return None;
    }
    for entry in &get_template_chain(inherits) {
        if let Some(val) = getter(&entry.frame) {
            return Some(val.clone());
        }
    }
    None
}

/// Create child frames from a ScrollChild element.
fn create_scroll_child_frames(
    lua: &Lua,
    children: &[FrameElement],
    parent_name: &str,
) -> Vec<String> {
    let mut all_names = Vec::new();
    for child in children {
        let Some((child_frame, child_type)) = frame_element_type(child) else {
            continue;
        };
        let names = create_child_frame_from_template(lua, child_frame, child_type, parent_name);
        all_names.extend(names);
    }
    all_names
}

/// Apply inline content from a FrameXml to an already-created frame.
fn apply_inline_frame_content(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) -> Vec<String> {
    apply_mixin(lua, &frame.combined_mixin(), frame_name);
    apply_inline_key_values(lua, frame, frame_name);
    apply_layers(lua, frame, frame_name);

    if let Some(thumb) = frame.thumb_texture() {
        elements::create_thumb_texture_from_template(lua, thumb, frame_name);
    }
    if let Some(bar) = frame.bar_texture() {
        elements::create_bar_texture_from_template(lua, bar, frame_name);
    }

    apply_inline_button_textures(lua, frame, frame_name);
    apply_button_text(lua, frame, frame_name);
    apply_editbox_fontstring(lua, frame, frame_name);
    apply_animation_groups(lua, frame, frame_name);

    let mut nested_names = create_child_frames(lua, frame, frame_name);
    if let Some(scroll_child) = frame.scroll_child() {
        nested_names.extend(create_scroll_child_frames(lua, &scroll_child.children, frame_name));
    }

    if let Some(scripts) = frame.scripts() {
        apply_scripts_from_template(lua, scripts, frame_name);
    }

    nested_names
}

/// Apply animation groups from a FrameXml to an already-created frame.
fn apply_animation_groups(lua: &Lua, frame: &FrameXml, frame_name: &str) {
    let Some(anims) = frame.animations() else { return };
    let mut code = format!("local frame = {}\n", frame_name);
    for group in &anims.animations {
        if group.is_virtual == Some(true) {
            continue;
        }
        code.push_str(&generate_animation_group_code(group, "frame"));
    }
    let _ = lua.load(&code).exec();
}

/// Apply KeyValues from inline frame content.
fn apply_inline_key_values(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let Some(key_values) = frame.key_values() else { return };
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
        ("Checked", "SetCheckedTexture", frame.checked_texture()),
        ("DisabledChecked", "SetDisabledCheckedTexture", frame.disabled_checked_texture()),
    ];
    for &(parent_key, setter, tex_opt) in texture_specs {
        if let Some(tex) = tex_opt {
            elements::create_button_texture_from_template(lua, tex, frame_name, parent_key, setter);
        }
    }
}

/// Create ButtonText fontstring from template.
fn apply_button_text(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let Some(fs) = frame.button_text() else { return };
    elements::create_fontstring_from_template(lua, fs, frame_name, "OVERLAY");
    // Assign to parent key (fontstring creation already does this if parentKey is set,
    // but ButtonText defaults to "Text" when no parentKey is specified)
    if fs.parent_key.is_none() {
        let code = format!(
            "do local p = {} if p then \
             local n = p:GetNumRegions() \
             if n > 0 then p.Text = select(n, p:GetRegions()) end \
             end end",
            frame_name
        );
        let _ = lua.load(&code).exec();
    }
}

/// Create EditBox FontString child from template.
fn apply_editbox_fontstring(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let Some(fs) = frame.font_string_child() else { return };
    elements::create_fontstring_from_template(lua, fs, frame_name, "OVERLAY");
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

/// Generate a unique ID (delegates to shared atomic counter).
fn rand_id() -> u64 {
    crate::loader::helpers::rand_id()
}
