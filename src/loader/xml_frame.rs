//! Frame creation from XML definitions.

use crate::lua_api::LoaderEnv;

use super::button::{apply_button_text, apply_button_textures};
use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_anchors_code, generate_animation_group_code, generate_scripts_code, get_size_values, rand_id};
use super::xml_fontstring::create_fontstring_from_xml;
use super::xml_texture::create_texture_from_xml;

/// Create a frame from XML definition.
/// Returns the name of the created frame (or None if skipped).
pub fn create_frame_from_xml(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
) -> Result<Option<String>, LoadError> {
    // Register virtual/intrinsic frames (templates) in the template registry
    if frame.is_virtual == Some(true) || frame.intrinsic == Some(true) {
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
    append_alpha_code(&mut lua_code, frame, inherits);
    append_enable_mouse_code(&mut lua_code, frame, inherits);
    append_set_all_points_code(&mut lua_code, frame, inherits);
    append_key_values_code(&mut lua_code, frame, inherits);
    append_xml_attributes_code(&mut lua_code, frame);
    append_id_code(&mut lua_code, frame);
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

    init_action_bar_tables(env, &name);

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
    // Engine-root frames (e.g. UIParent) are pre-created without a parent.
    // When XML defines them, name == default parent, which would self-parent.
    // Reuse the existing engine frame instead.
    if name == parent {
        return format!(
            r#"
        local frame = _G["{name}"]
        "#,
        );
    }
    format!(
        r#"
        local frame = CreateFrame("{widget_type}", "{name}", {parent}, {inherits_arg})
        "#,
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
    append_parent_array_code(lua_code, frame, parent);
}

/// Append parentArray insertion from the frame or its inherited templates.
fn append_parent_array_code(lua_code: &mut String, frame: &crate::xml::FrameXml, parent: &str) {
    // Check the frame itself first
    if let Some(parent_array) = &frame.parent_array {
        lua_code.push_str(&format!(
            "\n        {parent}.{parent_array} = {parent}.{parent_array} or {{}}\n        \
             table.insert({parent}.{parent_array}, frame)\n        ",
        ));
        return;
    }
    // Check inherited templates
    let inherits = frame.inherits.as_deref().unwrap_or("");
    if !inherits.is_empty() {
        for entry in &crate::xml::get_template_chain(inherits) {
            if let Some(parent_array) = &entry.frame.parent_array {
                lua_code.push_str(&format!(
                    "\n        {parent}.{parent_array} = {parent}.{parent_array} or {{}}\n        \
                     table.insert({parent}.{parent_array}, frame)\n        ",
                ));
                return;
            }
        }
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

/// Append `frame:SetAlpha(val)` if the frame has an alpha attribute (directly or via template).
fn append_alpha_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut alpha = frame.alpha;
    if alpha.is_none() && !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if let Some(a) = template_entry.frame.alpha {
                alpha = Some(a);
                break;
            }
        }
    }
    if let Some(a) = alpha {
        lua_code.push_str(&format!(
            r#"
        frame:SetAlpha({})
        "#,
            a
        ));
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
        Some("global") if !value.is_empty() => value.to_string(),
        Some("global") => "nil".to_string(),
        _ => format!("\"{}\"", escape_lua_string(value)),
    }
}

/// Append SetAttribute calls for `<Attributes>` XML elements.
fn append_xml_attributes_code(lua_code: &mut String, frame: &crate::xml::FrameXml) {
    if let Some(attrs) = frame.xml_attributes() {
        for attr in &attrs.entries {
            let value = match attr.attr_type.as_deref() {
                Some("number") => attr.value.as_deref().unwrap_or("0").to_string(),
                Some("boolean") => attr.value.as_deref().unwrap_or("false").to_lowercase(),
                Some("nil") => "nil".to_string(),
                _ => format!(
                    "\"{}\"",
                    escape_lua_string(attr.value.as_deref().unwrap_or(""))
                ),
            };
            lua_code.push_str(&format!(
                "\n        frame:SetAttribute(\"{}\", {})",
                escape_lua_string(&attr.name),
                value
            ));
        }
    }
}

/// Append SetID call if the frame has an `id` XML attribute.
fn append_id_code(lua_code: &mut String, frame: &crate::xml::FrameXml) {
    if let Some(id) = frame.xml_id {
        lua_code.push_str(&format!("\n        frame:SetID({})\n        ", id));
    }
}

/// Append script handler registrations from the frame's Scripts element.
fn append_scripts_code(lua_code: &mut String, frame: &crate::xml::FrameXml) {
    if let Some(scripts) = frame.scripts() {
        lua_code.push_str(&generate_scripts_code(scripts));
    }
}

/// Create textures and fontstrings from the frame's Layers.
fn create_layer_children(env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str) -> Result<(), LoadError> {
    for layers in frame.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");
            for (texture, is_mask) in layer.textures() {
                create_texture_from_xml(env, texture, name, draw_layer, is_mask)?;
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
    use crate::xml::FrameElement;
    match child {
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

/// Recursively create child frames and assign parentKey references.
fn create_child_frames(env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str) -> Result<(), LoadError> {
    // Use all_frame_elements() to handle multiple <Frames> sections in the XML
    for child in frame.all_frame_elements() {
        create_single_child_frame(env, child, name)?;
    }
    // ScrollChild children are parented to the ScrollFrame just like regular children
    if let Some(scroll_child) = frame.scroll_child() {
        create_frame_elements(env, &scroll_child.children, name)?;
    }
    Ok(())
}

/// Create a single child frame from a FrameElement and assign parentKey.
fn create_single_child_frame(
    env: &LoaderEnv<'_>,
    child: &crate::xml::FrameElement,
    parent_name: &str,
) -> Result<(), LoadError> {
    let (child_frame, child_type) = match frame_element_to_type(child) {
        Some(pair) => pair,
        None => return Ok(()),
    };
    let child_name = create_frame_from_xml(env, child_frame, child_type, Some(parent_name))?;
    if let (Some(actual_child_name), Some(parent_key)) =
        (child_name, &child_frame.parent_key)
    {
        let lua_code = format!("{}.{} = {}", parent_name, parent_key, actual_child_name);
        env.exec(&lua_code).ok();
    }
    Ok(())
}

/// Create frames from a list of FrameElement, assigning parentKey references.
fn create_frame_elements(
    env: &LoaderEnv<'_>,
    elements: &[crate::xml::FrameElement],
    parent_name: &str,
) -> Result<(), LoadError> {
    for child in elements {
        let (child_frame, child_type) = match frame_element_to_type(child) {
            Some(pair) => pair,
            None => continue,
        };
        let child_name = create_frame_from_xml(env, child_frame, child_type, Some(parent_name))?;

        // Assign parentKey so the parent can reference the child.
        // The Lua assignment triggers __newindex which syncs to Rust children_keys.
        if let (Some(actual_child_name), Some(parent_key)) =
            (child_name.clone(), &child_frame.parent_key)
        {
            let lua_code = format!(
                "\n                    {}.{} = {}\n                    ",
                parent_name, parent_key, actual_child_name
            );
            env.exec(&lua_code).ok();
        }
    }
    Ok(())
}

/// Apply animation groups from the frame and its inherited templates.
fn apply_animation_groups(env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str, inherits: &str) -> Result<(), LoadError> {
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
fn exec_animation_groups(env: &LoaderEnv<'_>, anims: &crate::xml::AnimationsXml, name: &str) {
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
    if let Err(e) = env.exec(&anim_code) {
        eprintln!("[AnimSetup] error: {}", e);
    }
}

/// Initialize tables expected by action bar OnLoad handlers.
/// Frames with `numButtons` KeyValue are action bars that need `actionButtons = {}`.
fn init_action_bar_tables(env: &LoaderEnv<'_>, name: &str) {
    let code = format!(
        r#"do local f = {}
        if f and f.numButtons and not f.actionButtons then
            f.actionButtons = {{}}
        end end"#,
        name
    );
    let _ = env.exec(&code);
}

/// Fire OnLoad and OnShow lifecycle scripts after the frame is fully configured.
/// Both handlers are wrapped in pcall to match WoW's C++ engine behavior where
/// script errors are caught and displayed, not propagated.
fn fire_lifecycle_scripts(env: &LoaderEnv<'_>, name: &str) {
    // In WoW, OnLoad fires at the end of frame creation from XML.
    // Templates often use method="OnLoad" which calls self:OnLoad().
    let onload_code = format!(
        r#"
        local frame = {}
        local handler = frame:GetScript("OnLoad")
        if handler then
            local ok, err = pcall(handler, frame)
            if not ok then
                print("[OnLoad] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
            end
        elseif type(frame.OnLoad) == "function" then
            local ok, err = pcall(frame.OnLoad, frame)
            if not ok then
                print("[OnLoad] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
            end
        end
        "#,
        name, name = name
    );
    if let Err(e) = env.exec(&onload_code) {
        eprintln!("[OnLoad] {} error: {}", name, e);
    }

    // In WoW, OnShow fires when a frame becomes visible, including at creation if visible.
    let onshow_code = format!(
        r#"
        local frame = {}
        if frame:IsVisible() then
            local handler = frame:GetScript("OnShow")
            if handler then
                local ok, err = pcall(handler, frame)
                if not ok then
                    print("[OnShow] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
                end
            elseif type(frame.OnShow) == "function" then
                local ok, err = pcall(frame.OnShow, frame)
                if not ok then
                    print("[OnShow] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
                end
            end
        end
        "#,
        name, name = name
    );
    if let Err(e) = env.exec(&onshow_code) {
        eprintln!("[OnShow] {} error: {}", name, e);
    }
}
