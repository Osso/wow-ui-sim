//! Frame creation from XML definitions.

use crate::lua_api::LoaderEnv;

use super::button::{apply_button_text, apply_button_textures};
use super::error::LoadError;
use super::helpers::{escape_lua_string, generate_anchors_code, generate_scripts_code, get_size_values, lua_global_ref, rand_id};
use super::xml_frame_extras::{apply_animation_groups, apply_bar_texture, init_action_bar_tables};
use super::xml_lifecycle::fire_lifecycle_scripts;
use super::xml_fontstring::create_fontstring_from_xml;
use super::xml_texture::create_texture_from_xml;

/// Create a frame from XML definition.
/// Returns the name of the created frame (or None if skipped).
///
/// `intrinsic_base` is set when the XML element is an intrinsic type (e.g.
/// `<ContainedAlertFrame>`) whose registered template should be implicitly
/// inherited before any explicit `inherits` attribute.
pub fn create_frame_from_xml(
    env: &LoaderEnv<'_>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_override: Option<&str>,
    intrinsic_base: Option<&str>,
) -> Result<Option<String>, LoadError> {
    // Register virtual/intrinsic frames (templates) in the template registry.
    // Top-level virtual frames are templates only (not instantiated).
    // Child virtual frames (with parent_override) are still created — WoW's
    // engine creates them as children and optionally registers them as templates
    // when they have a name.
    if frame.is_virtual == Some(true) || frame.intrinsic == Some(true) {
        if let Some(ref name) = frame.name {
            crate::xml::register_template(name, widget_type, frame.clone());
        }
        if parent_override.is_none() {
            return Ok(None);
        }
    }

    let name = match resolve_frame_name(frame, parent_override) {
        Some(n) => n,
        None => return Ok(None),
    };

    let inherited_parent_buf = resolve_parent(frame, parent_override);
    let explicit_parent = parent_override
        .or(frame.parent.as_deref())
        .or(inherited_parent_buf.as_deref());
    let parent = explicit_parent.unwrap_or("UIParent");

    // Prepend intrinsic base template to the inherits chain so the intrinsic
    // type's mixin, scripts, and children are applied before user templates.
    let explicit_inherits = frame.inherits.as_deref().unwrap_or("");
    let inherits_buf;
    let inherits = match intrinsic_base {
        Some(base) if !explicit_inherits.is_empty() => {
            inherits_buf = format!("{}, {}", base, explicit_inherits);
            &inherits_buf
        }
        Some(base) => base,
        None => explicit_inherits,
    };

    let lua_code = build_frame_lua_code(widget_type, &name, explicit_parent, inherits, frame, parent);
    exec_create_frame_code(env, &lua_code, &name)?;
    apply_intrinsic_property(env, intrinsic_base, &name);
    create_children_and_finalize(env, frame, &name, inherits)?;
    Ok(Some(name))
}

/// Build the Lua code that creates a frame and sets all XML-driven properties.
fn build_frame_lua_code(
    widget_type: &str, name: &str, explicit_parent: Option<&str>,
    inherits: &str, frame: &crate::xml::FrameXml, parent: &str,
) -> String {
    let mut lua_code = build_create_frame_code(widget_type, name, explicit_parent, inherits);
    append_parent_key_code(&mut lua_code, frame, parent);
    append_mixins_code(&mut lua_code, frame, inherits);
    append_size_code(&mut lua_code, frame, inherits);
    append_anchors_code(&mut lua_code, frame, inherits, parent);
    append_frame_strata_code(&mut lua_code, frame, inherits);
    append_frame_level_code(&mut lua_code, frame, inherits);
    append_hidden_code(&mut lua_code, frame, inherits);
    append_toplevel_code(&mut lua_code, frame, inherits);
    append_alpha_code(&mut lua_code, frame, inherits);
    append_enable_mouse_code(&mut lua_code, frame, inherits);
    append_clamped_to_screen_code(&mut lua_code, frame, inherits);
    append_set_all_points_code(&mut lua_code, frame, inherits);
    append_key_values_code(&mut lua_code, frame, inherits);
    append_xml_attributes_code(&mut lua_code, frame);
    append_id_code(&mut lua_code, frame);
    append_scripts_code(&mut lua_code, frame);
    lua_code
}

/// Set the `intrinsic` property on intrinsic frames (e.g. frame.intrinsic = "DropdownButton").
fn apply_intrinsic_property(env: &LoaderEnv<'_>, intrinsic_base: Option<&str>, name: &str) {
    if let Some(base) = intrinsic_base {
        let code = format!("{}.intrinsic = \"{}\"", lua_global_ref(name), base);
        env.exec(&code).ok();
    }
}

/// Create child frames, layer children, animations, and apply button/bar textures.
fn create_children_and_finalize(
    env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str, inherits: &str,
) -> Result<(), LoadError> {
    create_child_frames(env, frame, name)?;
    create_layer_children(env, frame, name)?;
    apply_animation_groups(env, frame, name, inherits)?;
    apply_button_textures(env, frame, name)?;
    apply_button_text(env, frame, name, inherits)?;
    apply_bar_texture(env, frame, name)?;
    init_action_bar_tables(env, name);
    fire_lifecycle_scripts(env, name);
    Ok(())
}

/// Execute the CreateFrame Lua code with OnLoad suppression.
///
/// Suppresses OnLoad during CreateFrame so the XML loader controls when it fires.
/// Template children created during CreateFrame have their OnLoad deferred until
/// instance-level KeyValues (e.g. layoutIndex) are applied in the Lua chunk.
/// Uses a depth counter to handle recursive create_frame_from_xml calls correctly.
fn exec_create_frame_code(env: &LoaderEnv<'_>, lua_code: &str, name: &str) -> Result<(), LoadError> {
    env.exec("__suppress_create_frame_onload = (__suppress_create_frame_onload or 0) + 1").ok();
    let exec_result = env.exec(lua_code).map_err(|e| {
        LoadError::Lua(format!("Failed to create frame {}: {}", name, e))
    });
    env.exec("__suppress_create_frame_onload = __suppress_create_frame_onload - 1").ok();
    exec_result?;
    crate::lua_api::globals::template::fire_deferred_child_onloads(env.lua());
    Ok(())
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

/// Resolve the parent for a frame, checking inherited templates when the frame
/// itself has no explicit `parent` attribute (e.g. ClassPowerBarFrame defines
/// `parent="PlayerFrame"` which propagates to PaladinPowerBarFrame).
///
/// Returns `Some(parent_name)` from the template chain, or `None` if no
/// template provides a parent.  The caller should prefer `parent_override`
/// and `frame.parent` first.
fn resolve_parent(frame: &crate::xml::FrameXml, parent_override: Option<&str>) -> Option<String> {
    if parent_override.is_some() || frame.parent.is_some() {
        return None; // Already have an explicit parent, no need to search templates.
    }
    frame.inherits.as_deref().and_then(|inherits| {
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|entry| entry.frame.parent.clone())
    })
}

/// Build the initial `CreateFrame(...)` Lua code.
fn build_create_frame_code(widget_type: &str, name: &str, parent: Option<&str>, inherits: &str) -> String {
    let inherits_arg = if inherits.is_empty() {
        "nil".to_string()
    } else {
        format!("\"{}\"", inherits)
    };
    // Engine-root frames (e.g. UIParent) are pre-created without a parent.
    // When XML defines them, name == default parent, which would self-parent.
    // Reuse the existing engine frame instead.
    if let Some(p) = parent {
        if name == p {
            return format!(
                r#"
        local frame = _G["{name}"]
        "#,
            );
        }
    }
    let parent_arg = match parent {
        Some(p) => format!("{} or UIParent", lua_global_ref(p)),
        // Lua CreateFrame defaults nil parent to UIParent, so pass UIParent
        // here and orphan the frame with SetParent(nil) afterwards.
        None => "UIParent".to_string(),
    };
    let orphan_code = if parent.is_none() {
        // In WoW, top-level XML frames without a parent attribute are created
        // as orphans (no parent). Our Lua CreateFrame always defaults to
        // UIParent, so we create with UIParent then immediately orphan.
        "\n        frame:SetParent(nil)"
    } else {
        ""
    };
    format!(
        r#"
        local frame = CreateFrame("{widget_type}", "{name}", {parent_arg}, {inherits_arg}){orphan_code}
        "#,
    )
}

/// Append parentKey assignment so sibling frames can reference this frame.
///
/// Handles `$parent` prefix in parentKey (e.g. `$parent.CloseButton`)
/// which navigates up from the direct parent before setting the key.
fn append_parent_key_code(lua_code: &mut String, frame: &crate::xml::FrameXml, parent: &str) {
    if let Some(parent_key) = &frame.parent_key {
        let parent_ref = lua_global_ref(parent);
        if let Some(key) = parent_key.strip_prefix("$parent.") {
            lua_code.push_str(&format!(
                r#"
        do local __pk = {}:GetParent(); if __pk then __pk.{} = frame end end
        "#,
                parent_ref, key
            ));
        } else {
            lua_code.push_str(&format!(
                r#"
        {}.{} = frame
        "#,
                parent_ref, parent_key
            ));
        }
    }
    append_parent_array_code(lua_code, frame, parent);
}

/// Append parentArray insertion when the attribute is directly on this frame.
///
/// Template-inherited parentArray is handled by `apply_parent_array_from_template`
/// inside `CreateFrame`, so we only handle the direct-attribute case here.
fn append_parent_array_code(lua_code: &mut String, frame: &crate::xml::FrameXml, parent: &str) {
    if let Some(parent_array) = &frame.parent_array {
        let parent_ref = lua_global_ref(parent);
        lua_code.push_str(&format!(
            "\n        {parent_ref}.{parent_array} = {parent_ref}.{parent_array} or {{}}\n        \
             table.insert({parent_ref}.{parent_array}, frame)\n        ",
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

/// Append `frame:SetToplevel(true)` if the frame has toplevel="true" (directly or via template).
fn append_toplevel_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let toplevel = frame.toplevel.or_else(|| {
        resolve_from_templates(inherits, |f| f.toplevel)
    });
    if toplevel == Some(true) {
        lua_code.push_str("\n        frame:SetToplevel(true)\n        ");
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

/// Resolve clampedToScreen from the frame and templates, then call SetClampedToScreen.
fn append_clamped_to_screen_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let mut clamped = frame.clamped_to_screen;
    if clamped.is_none() && !inherits.is_empty() {
        for entry in &crate::xml::get_template_chain(inherits) {
            if let Some(c) = entry.frame.clamped_to_screen {
                clamped = Some(c);
            }
        }
    }
    if let Some(c) = clamped {
        lua_code.push_str(&format!(
            "\n        frame:SetClampedToScreen({})\n        ",
            if c { "true" } else { "false" }
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
            let sub_level = layer.texture_sub_level.unwrap_or(0);
            for (texture, is_mask, is_line) in layer.textures() {
                create_texture_from_xml(env, texture, name, draw_layer, is_mask, is_line, sub_level)?;
            }
            for fontstring in layer.font_strings() {
                create_fontstring_from_xml(env, fontstring, name, draw_layer, sub_level)?;
            }
        }
    }
    Ok(())
}

/// Map a FrameElement variant to its (FrameXml, widget_type, intrinsic_name) triple.
/// `intrinsic_name` is Some when the XML element is an intrinsic type whose template
/// should be implicitly inherited (e.g. `<ContainedAlertFrame>` inherits "ContainedAlertFrame").
/// Returns None for unsupported element types.
fn frame_element_to_type(child: &crate::xml::FrameElement) -> Option<(&crate::xml::FrameXml, &'static str, Option<&'static str>)> {
    use crate::xml::FrameElement;
    match child {
        FrameElement::Frame(f) => Some((f, "Frame", None)),
        FrameElement::Button(f) => Some((f, "Button", None)),
        FrameElement::DropdownButton(f) => Some((f, "Button", Some("DropdownButton"))),
        FrameElement::DropDownToggleButton(f) => Some((f, "Button", Some("DropDownToggleButton"))),
        FrameElement::EventButton(f) => Some((f, "Button", Some("EventButton"))),
        FrameElement::ContainedAlertFrame(f) => Some((f, "Button", Some("ContainedAlertFrame"))),
        FrameElement::ItemButton(f) => Some((f, "ItemButton", None)),
        FrameElement::CheckButton(f) => Some((f, "CheckButton", None)),
        FrameElement::EditBox(f)
        | FrameElement::EventEditBox(f) => Some((f, "EditBox", None)),
        FrameElement::ScrollFrame(f)
        | FrameElement::EventScrollFrame(f) => Some((f, "ScrollFrame", None)),
        FrameElement::Slider(f) => Some((f, "Slider", None)),
        FrameElement::StatusBar(f) => Some((f, "StatusBar", None)),
        FrameElement::Cooldown(f) => Some((f, "Cooldown", None)),
        FrameElement::GameTooltip(f) => Some((f, "GameTooltip", None)),
        FrameElement::ColorSelect(f) => Some((f, "ColorSelect", None)),
        FrameElement::Model(f)
        | FrameElement::DressUpModel(f) => Some((f, "Model", None)),
        FrameElement::ModelScene(f) => Some((f, "ModelScene", None)),
        FrameElement::PlayerModel(f)
        | FrameElement::CinematicModel(f) => Some((f, "PlayerModel", None)),
        FrameElement::MessageFrame(f)
        | FrameElement::ScrollingMessageFrame(f) => Some((f, "MessageFrame", None)),
        FrameElement::SimpleHTML(f) => Some((f, "SimpleHTML", None)),
        FrameElement::Minimap(f) => Some((f, "Minimap", None)),
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
        | FrameElement::MapScene(f)
        | FrameElement::Line(f)
        | FrameElement::Browser(f)
        | FrameElement::MovieFrame(f)
        | FrameElement::WorldFrame(f) => Some((f, "Frame", None)),
        FrameElement::ScopedModifier(_) => None,
    }
}

/// Recursively create child frames and assign parentKey references.
fn create_child_frames(env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str) -> Result<(), LoadError> {
    // Use all_frame_elements() to handle multiple <Frames> sections in the XML
    // and standalone frame-type children outside <Frames> wrappers
    let elements = frame.all_frame_elements();
    for child in &elements {
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
    let (child_frame, child_type, intrinsic) = match frame_element_to_type(child) {
        Some(triple) => triple,
        None => return Ok(()),
    };
    let child_name = create_frame_from_xml(env, child_frame, child_type, Some(parent_name), intrinsic)?;
    if let (Some(actual_child_name), Some(parent_key)) =
        (child_name, &child_frame.parent_key)
    {
        let lua_code = format!("{}.{} = {}", lua_global_ref(parent_name), parent_key, lua_global_ref(&actual_child_name));
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
        let (child_frame, child_type, intrinsic) = match frame_element_to_type(child) {
            Some(triple) => triple,
            None => continue,
        };
        let child_name = create_frame_from_xml(env, child_frame, child_type, Some(parent_name), intrinsic)?;

        // Assign parentKey so the parent can reference the child.
        // The Lua assignment triggers __newindex which syncs to Rust children_keys.
        if let (Some(actual_child_name), Some(parent_key)) =
            (child_name.clone(), &child_frame.parent_key)
        {
            let lua_code = format!(
                "\n                    {}.{} = {}\n                    ",
                lua_global_ref(parent_name), parent_key, lua_global_ref(&actual_child_name)
            );
            env.exec(&lua_code).ok();
        }
    }
    Ok(())
}

/// Append `frame:SetFrameStrata(strata)` if the frame has a frameStrata attribute.
fn append_frame_strata_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let strata = frame.frame_strata.clone().or_else(|| {
        resolve_string_from_templates(inherits, |f| f.frame_strata.clone())
    });
    if let Some(s) = strata {
        lua_code.push_str(&format!("\n        frame:SetFrameStrata(\"{}\")\n        ", s));
    }
}

/// Append `frame:SetFrameLevel(level)` if the frame has a frameLevel attribute.
fn append_frame_level_code(lua_code: &mut String, frame: &crate::xml::FrameXml, inherits: &str) {
    let level = frame.frame_level.or_else(|| {
        resolve_from_templates(inherits, |f| f.frame_level)
    });
    if let Some(l) = level {
        lua_code.push_str(&format!("\n        frame:SetFrameLevel({})\n        ", l));
    }
}

/// Search template chain for the first Some value returned by the extractor.
fn resolve_from_templates<T: Copy>(inherits: &str, extract: fn(&crate::xml::FrameXml) -> Option<T>) -> Option<T> {
    if inherits.is_empty() { return None; }
    crate::xml::get_template_chain(inherits)
        .iter()
        .find_map(|entry| extract(&entry.frame))
}

/// Search template chain for the first Some(String) returned by the extractor.
fn resolve_string_from_templates(inherits: &str, extract: fn(&crate::xml::FrameXml) -> Option<String>) -> Option<String> {
    if inherits.is_empty() { return None; }
    crate::xml::get_template_chain(inherits)
        .iter()
        .find_map(|entry| extract(&entry.frame))
}

