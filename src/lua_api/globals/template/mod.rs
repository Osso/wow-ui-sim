//! Template application from the XML template registry.
//!
//! This module provides functionality to apply XML templates from the registry
//! when CreateFrame is called with a template name.

pub(crate) mod direct;
mod elements;

use crate::loader::helpers::generate_set_point_code;
use crate::loader::helpers_anim::generate_animation_group_code;
use crate::lua_api::SimState;
use crate::xml::{get_template_chain, FrameElement, FrameXml, TemplateEntry};
use mlua::Lua;
use std::cell::RefCell;
use std::rc::Rc;

/// Extract the FrameXml, widget type, and optional intrinsic name from a FrameElement.
fn frame_element_type(element: &FrameElement) -> Option<(&FrameXml, &'static str, Option<&'static str>)> {
    match element {
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
        | FrameElement::Minimap(f)
        | FrameElement::MovieFrame(f)
        | FrameElement::WorldFrame(f) => Some((f, "Frame", None)),
        FrameElement::ScopedModifier(_) => None,
    }
}

/// Apply templates from the registry to a frame.
///
/// This generates Lua code to create child frames, textures, and fontstrings
/// defined in the template chain (including inherited templates).
pub fn apply_templates_from_registry(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame_name: &str,
    template_names: &str,
) {
    let chain = get_template_chain(template_names);
    if chain.is_empty() {
        return;
    }

    let mut all_child_names = Vec::new();
    for entry in &chain {
        let child_names = apply_single_template(lua, state, frame_name, entry);
        all_child_names.extend(child_names);
    }

    // When __suppress_create_frame_onload is set (XML loading mode), defer child
    // OnLoad firing so the XML loader can apply instance-level KeyValues first.
    // Child OnLoad handlers may reference parent properties set via KeyValues
    // (e.g. ArenaEnemyPetFrame accesses parent.layoutIndex).
    let suppress_depth: i32 = lua
        .globals()
        .get("__suppress_create_frame_onload")
        .unwrap_or(0);

    if suppress_depth > 0 {
        // Store names for the XML loader to fire after KeyValues are applied
        let deferred: mlua::Table = lua
            .globals()
            .get("__deferred_child_onloads")
            .unwrap_or_else(|_| lua.create_table().unwrap());
        for child_name in &all_child_names {
            let len = deferred.raw_len();
            let _ = deferred.raw_set(len + 1, child_name.as_str());
        }
        let _ = lua.globals().set("__deferred_child_onloads", deferred);
    } else {
        // Fire OnLoad for all child frames created during template application.
        // This is deferred until after ALL templates in the chain are applied,
        // because child OnLoad handlers may depend on KeyValues from later templates.
        for child_name in &all_child_names {
            fire_on_load(lua, child_name);
        }
    }
}

/// Fire all deferred child OnLoad scripts that were queued during template
/// application while `__suppress_create_frame_onload` was active.
pub fn fire_deferred_child_onloads(lua: &Lua) {
    let Ok(deferred) = lua.globals().get::<mlua::Table>("__deferred_child_onloads") else {
        return;
    };
    let names: Vec<String> = deferred
        .sequence_values::<String>()
        .filter_map(|r| r.ok())
        .collect();
    let _ = lua.globals().set("__deferred_child_onloads", mlua::Value::Nil);
    for name in &names {
        fire_on_load(lua, name);
    }
}

/// Apply a single template entry to a frame, returning names of created children.
fn apply_single_template(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame_name: &str,
    entry: &TemplateEntry,
) -> Vec<String> {
    let template = &entry.frame;

    // Apply mixin (must be before children and scripts) — stays in Lua
    apply_mixin(lua, &template.combined_mixin(), frame_name);

    // Look up frame_id for direct Rust property setting
    let frame_id = state.borrow().widgets.get_id_by_name(frame_name);

    // Apply key values from template (handles multiple <KeyValues> blocks) — stays in Lua
    for key_values in template.all_key_values() {
        apply_key_values(lua, key_values, frame_name);
    }

    // Direct Rust property setting (bypasses Lua compilation)
    if let Some(fid) = frame_id {
        direct::set_size(state, fid, template);
        direct::set_anchors(state, fid, template, frame_name);
        direct::set_all_points(state, fid, template);
        direct::set_hidden(state, fid, template);
    }

    // Apply layers (textures and fontstrings)
    // subst_parent = frame_name at the template root level
    apply_layers(lua, template, frame_name, frame_name);

    // Apply button textures (NormalTexture, PushedTexture, etc.)
    apply_button_textures(lua, template, frame_name, frame_name);

    // Apply StatusBar BarTexture
    if let Some(bar) = template.bar_texture() {
        elements::create_bar_texture_from_template(lua, bar, frame_name, frame_name);
    }

    // Apply Slider ThumbTexture
    if let Some(thumb) = template.thumb_texture() {
        elements::create_thumb_texture_from_template(lua, thumb, frame_name, frame_name);
    }

    // Apply ButtonText and EditBox FontString
    apply_button_text(lua, template, frame_name, frame_name);
    elements::apply_button_text_attribute(lua, template, frame_name);
    apply_editbox_fontstring(lua, template, frame_name, frame_name);
    apply_button_fonts(lua, template, frame_name);
    apply_animation_groups(lua, template, frame_name);

    // Create child frames defined in the template
    let mut child_names = create_child_frames(lua, state, template, frame_name, frame_name);

    // Create ScrollChild children
    if let Some(scroll_child) = template.scroll_child() {
        child_names.extend(create_scroll_child_frames(lua, state, &scroll_child.children, frame_name, frame_name));
    }

    // Apply scripts from template (after children, so OnLoad can reference them)
    if let Some(scripts) = template.scripts() {
        elements::apply_scripts_from_template(lua, scripts, frame_name);
    }

    child_names
}

/// Apply key values from a template to a frame.
fn apply_key_values(
    lua: &Lua,
    key_values: &crate::xml::KeyValuesXml,
    frame_name: &str,
) {
    let frame_ref = lua_global_ref(frame_name);
    for kv in &key_values.values {
        let value = format_key_value(&kv.value, kv.value_type.as_deref());
        let code = format!(
            "do local f = {} if f then f.{} = {} end end",
            frame_ref, kv.key, value
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
///
/// `subst_parent` is the name used for `$parent` substitution in child names.
/// For anonymous frames, this propagates from the nearest named ancestor.
fn apply_layers(lua: &Lua, template: &FrameXml, frame_name: &str, subst_parent: &str) {
    for layers in template.layers() {
        for layer in &layers.layers {
            let draw_layer = layer.level.as_deref().unwrap_or("ARTWORK");
            for (texture, is_mask, is_line) in layer.textures() {
                elements::create_texture_from_template(lua, texture, frame_name, subst_parent, draw_layer, is_mask, is_line);
            }
            for fontstring in layer.font_strings() {
                elements::create_fontstring_from_template(lua, fontstring, frame_name, subst_parent, draw_layer);
            }
        }
    }
}

/// Apply button textures (NormalTexture, PushedTexture, etc.) from a template.
fn apply_button_textures(lua: &Lua, template: &FrameXml, frame_name: &str, subst_parent: &str) {
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
            elements::create_button_texture_from_template(lua, tex, frame_name, subst_parent, parent_key, setter);
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
        if name == "EventFrameMixin" || name == "CallbackRegistryMixin" {
            // Initialize callbackTables immediately so TriggerEvent works even
            // before OnLoad fires (Show() during creation can trigger OnShow).
            post_init.push_str(
                "if f.OnLoad_Intrinsic then pcall(f.OnLoad_Intrinsic, f) end ",
            );
        }
    }
    let code = format!(
        "do local f = {} if f then {} {} end end",
        lua_global_ref(frame_name),
        parts.join(" "),
        post_init,
    );
    let _ = lua.load(&code).exec();
}

/// Fire OnLoad on a frame.
///
/// Only fires handlers registered via `SetScript` (from `<Scripts>` XML tags)
/// and `OnLoad_Intrinsic` (from intrinsic mixins like EventFrameMixin).
/// Does NOT call `frame.OnLoad` as a fallback — in WoW, the C++ engine only
/// calls registered script handlers, not mixin table fields. Mixin OnLoad
/// methods are invoked via `<Scripts><OnLoad method="OnLoad"/></Scripts>` which
/// generates a `SetScript("OnLoad", function(self) self:OnLoad() end)` call.
pub(crate) fn fire_on_load(lua: &Lua, frame_name: &str) {
    let frame_ref = lua_global_ref(frame_name);
    // Fire intrinsic OnLoad_Intrinsic first (e.g. EventFrameMixin) — in WoW,
    // intrinsic scripts always fire regardless of what user templates set.
    let code = format!(
        r#"
        local frame = {frame_ref}
        if frame then
            if type(frame.OnLoad_Intrinsic) == "function" then
                local ok, err = pcall(frame.OnLoad_Intrinsic, frame)
                if not ok then
                    return tostring(err)
                end
            end
            local handler = frame:GetScript("OnLoad")
            if handler then
                local ok, err = pcall(handler, frame)
                if not ok then
                    return tostring(err)
                end
            end
        end
        "#
    );
    match lua.load(&code).eval::<Option<String>>() {
        Ok(Some(err)) => eprintln!("[fire_on_load] {} error: {}", frame_name, err),
        Err(e) => eprintln!("[fire_on_load] {} eval error: {}", frame_name, e),
        _ => {}
    }
}

/// Create child frames from template XML.
fn create_child_frames(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame: &FrameXml,
    parent_name: &str,
    subst_parent: &str,
) -> Vec<String> {
    let mut all_names = Vec::new();
    let elements = frame.all_frame_elements();
    for child in &elements {
        let Some((child_frame, child_type, intrinsic)) = frame_element_type(child) else {
            continue;
        };
        let names = create_child_frame_from_template(
            lua, state, child_frame, child_type, intrinsic, parent_name, subst_parent,
        );
        all_names.extend(names);
    }
    all_names
}

/// Create a child frame from template XML.
/// Returns names of this frame AND all nested descendants (for deferred OnLoad).
///
/// `subst_parent` is the name used for `$parent` substitution. For anonymous
/// frames (no name attribute), `$parent` propagates from the nearest named
/// ancestor rather than using the auto-generated frame name.
fn create_child_frame_from_template(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    intrinsic: Option<&str>,
    parent_name: &str,
    subst_parent: &str,
) -> Vec<String> {
    let is_named = frame.name.is_some();
    let child_name = frame
        .name
        .as_ref()
        .map(|n| n.replace("$parent", subst_parent))
        .unwrap_or_else(|| format!("__tpl_{}", rand_id()));

    // For named children, their name becomes the new $parent for descendants.
    // For anonymous children, propagate the current subst_parent.
    let child_subst = if is_named { &child_name } else { subst_parent };

    let code = build_create_child_code(frame, widget_type, parent_name, &child_name);
    if let Err(e) = lua.load(&code).exec() {
        eprintln!(
            "[template] Failed to create child '{}' (type={}) under '{}': {}",
            child_name, widget_type, parent_name, e
        );
    }

    // Apply intrinsic template (e.g. DropdownButton) and set the intrinsic property.
    if let Some(intrinsic_name) = intrinsic {
        apply_templates_from_registry(lua, state, &child_name, intrinsic_name);
        let code = format!(
            "{}.intrinsic = \"{}\"",
            lua_global_ref(&child_name),
            intrinsic_name
        );
        let _ = lua.load(&code).exec();
    }

    // Apply inherited templates AFTER CreateFrame but BEFORE inline content.
    // CreateFrame is called without the template arg so it doesn't fire OnLoad
    // prematurely. Templates set up mixin/scripts, then inline content adds
    // layers/children, and finally the deferred OnLoad fires with everything
    // in place.
    let inherits = frame.inherits.as_deref().unwrap_or("");
    if !inherits.is_empty() {
        apply_templates_from_registry(lua, state, &child_name, inherits);
    }

    let nested_names = apply_inline_frame_content(lua, state, frame, &child_name, child_subst);

    let mut all_names = nested_names;
    all_names.push(child_name);
    all_names
}

/// Build Lua code to create a child frame with size, anchors, visibility, and parentKey.
///
/// The `inherits` template is NOT passed to CreateFrame here — it's applied
/// separately in `create_child_frame_from_template` so that inline XML content
/// (layers, children) is fully created before OnLoad fires.
fn build_create_child_code(
    frame: &crate::xml::FrameXml,
    widget_type: &str,
    parent_name: &str,
    child_name: &str,
) -> String {
    let mut code = format!(
        r#"
        local parent = {}
        if parent then
            local child = CreateFrame("{}", "{}", parent, nil)
        "#,
        lua_global_ref(parent_name),
        widget_type,
        escape_lua_string(child_name),
    );

    append_child_size_and_anchors(&mut code, frame, parent_name);
    append_child_parent_refs(&mut code, frame);
    code.push_str(&format!("            _G[\"{}\"] = child\n", escape_lua_string(child_name)));
    code.push_str("        end\n");
    code
}

/// Append anchors, setAllPoints, and hidden to child frame code.
/// Size is NOT set here — it's applied later in apply_inline_frame_content
/// so that template defaults are set first, then inline size overrides.
fn append_child_size_and_anchors(code: &mut String, frame: &FrameXml, parent_name: &str) {
    if let Some(anchors) = frame.anchors() {
        code.push_str(&generate_set_point_code(anchors, "child", "parent", parent_name, "nil"));
    }
    if frame.set_all_points == Some(true) {
        code.push_str("            child:SetAllPoints(true)\n");
    }
    let hidden = frame.hidden.or_else(|| {
        let inherits = frame.inherits.as_deref().unwrap_or("");
        if inherits.is_empty() {
            return None;
        }
        for entry in &get_template_chain(inherits) {
            if entry.frame.hidden.is_some() {
                return entry.frame.hidden;
            }
        }
        None
    });
    if hidden == Some(true) {
        code.push_str("            child:Hide()\n");
    }
}

/// Append parentKey and parentArray assignment (with template chain resolution).
fn append_child_parent_refs(code: &mut String, frame: &FrameXml) {
    if let Some(parent_key) = &resolve_inherited_field(frame, |f| f.parent_key.as_ref()) {
        let key = escape_lua_string(parent_key);
        code.push_str(&format!("            parent[\"{key}\"] = child\n"));
    }
    if let Some(parent_array) = &resolve_inherited_field(frame, |f| f.parent_array.as_ref()) {
        let arr = escape_lua_string(parent_array);
        code.push_str(&format!(
            "            parent[\"{arr}\"] = parent[\"{arr}\"] or {{}}\n\
             table.insert(parent[\"{arr}\"], child)\n"
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
    state: &Rc<RefCell<SimState>>,
    children: &[FrameElement],
    parent_name: &str,
    subst_parent: &str,
) -> Vec<String> {
    let mut all_names = Vec::new();
    for child in children {
        let Some((child_frame, child_type, intrinsic)) = frame_element_type(child) else {
            continue;
        };
        let names = create_child_frame_from_template(
            lua, state, child_frame, child_type, intrinsic, parent_name, subst_parent,
        );
        all_names.extend(names);
    }
    all_names
}

/// Apply inline content from a FrameXml to an already-created frame.
fn apply_inline_frame_content(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame: &crate::xml::FrameXml,
    frame_name: &str,
    subst_parent: &str,
) -> Vec<String> {
    apply_mixin(lua, &frame.combined_mixin(), frame_name);
    apply_inline_key_values(lua, frame, frame_name);
    // Re-apply inline size — templates may override the size set in build_create_child_code.
    let fid = state.borrow().widgets.get_id_by_name(frame_name);
    if let Some(fid) = fid {
        direct::set_size_partial(state, fid, frame);
    }
    apply_layers(lua, frame, frame_name, subst_parent);

    if let Some(thumb) = frame.thumb_texture() {
        elements::create_thumb_texture_from_template(lua, thumb, frame_name, subst_parent);
    }
    if let Some(bar) = frame.bar_texture() {
        elements::create_bar_texture_from_template(lua, bar, frame_name, subst_parent);
    }

    apply_inline_button_textures(lua, frame, frame_name, subst_parent);
    apply_button_text(lua, frame, frame_name, subst_parent);
    elements::apply_button_text_attribute(lua, frame, frame_name);
    apply_editbox_fontstring(lua, frame, frame_name, subst_parent);
    apply_animation_groups(lua, frame, frame_name);

    let mut nested_names = create_child_frames(lua, state, frame, frame_name, subst_parent);
    if let Some(scroll_child) = frame.scroll_child() {
        nested_names.extend(create_scroll_child_frames(lua, state, &scroll_child.children, frame_name, subst_parent));
    }

    if let Some(scripts) = frame.scripts() {
        elements::apply_scripts_from_template(lua, scripts, frame_name);
    }

    nested_names
}

/// Apply animation groups from a FrameXml to an already-created frame.
fn apply_animation_groups(lua: &Lua, frame: &FrameXml, frame_name: &str) {
    let Some(anims) = frame.animations() else { return };
    let mut code = format!("local frame = {}\n", lua_global_ref(frame_name));
    for group in &anims.animations {
        if group.is_virtual == Some(true) {
            continue;
        }
        code.push_str(&generate_animation_group_code(group, "frame"));
    }
    let _ = lua.load(&code).exec();
}

/// Apply KeyValues from inline frame content (handles multiple `<KeyValues>` blocks).
fn apply_inline_key_values(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let frame_ref = lua_global_ref(frame_name);
    for key_values in frame.all_key_values() {
        for kv in &key_values.values {
            let value = format_key_value(&kv.value, kv.value_type.as_deref());
            let code = format!(
                "do local f = {} if f then f.{} = {} end end",
                frame_ref, kv.key, value
            );
            let _ = lua.load(&code).exec();
        }
    }
}

/// Apply button textures from inline frame content.
fn apply_inline_button_textures(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str, subst_parent: &str) {
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
            elements::create_button_texture_from_template(lua, tex, frame_name, subst_parent, parent_key, setter);
        }
    }
}

/// Create ButtonText fontstring from template.
fn apply_button_text(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str, subst_parent: &str) {
    let Some(fs) = frame.button_text() else { return };
    elements::create_fontstring_from_template(lua, fs, frame_name, subst_parent, "OVERLAY");
    // Only apply SetAllPoints when the ButtonText has no explicit anchors.
    // Templates like ChatTabTemplate define explicit anchors (e.g. CENTER 0 -5)
    // that would be wiped by SetAllPoints.
    let has_anchors = fs.anchors.as_ref().is_some_and(|a| !a.anchors.is_empty());
    let text_ref = if let Some(ref pk) = fs.parent_key {
        format!("p[\"{}\"]", escape_lua_string(pk))
    } else {
        "select(p:GetNumRegions(), p:GetRegions())".to_string()
    };
    let set_all_points = if has_anchors { "" } else { "if t then t:SetAllPoints(p) end " };
    let code = format!(
        "do local p = {} if p then \
         local t = {text_ref} \
         {set_all_points}\
         if not p.Text then p.Text = t end \
         end end",
        lua_global_ref(frame_name)
    );
    let _ = lua.load(&code).exec();
}

/// Apply NormalFont/HighlightFont/DisabledFont from template XML.
fn apply_button_fonts(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str) {
    let frame_ref = lua_global_ref(frame_name);
    for (setter, font_ref) in frame.button_fonts() {
        let Some(font_ref) = font_ref else { continue };
        let Some(style) = font_ref.style.as_deref().or(font_ref.inherits.as_deref()) else { continue };
        let code = format!(
            "do local f={frame_ref} local fo={style} if f and fo then f:{setter}(fo) \
             if f.Text and f.Text.SetFontObject then f.Text:SetFontObject(fo) end end end"
        );
        let _ = lua.load(&code).exec();
    }
}

/// Create EditBox FontString child from template.
fn apply_editbox_fontstring(lua: &Lua, frame: &crate::xml::FrameXml, frame_name: &str, subst_parent: &str) {
    let Some(fs) = frame.font_string_child() else { return };
    elements::create_fontstring_from_template(lua, fs, frame_name, subst_parent, "OVERLAY");
}

/// Get size values from a SizeXml.
pub(super) fn get_size_values(size: &crate::xml::SizeXml) -> (Option<f32>, Option<f32>) {
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

/// Return a Lua expression that references a frame by its global name.
///
/// Frame names like `$TankMarkerCheckButton` contain characters that aren't
/// valid in Lua identifiers, so we always use `_G["name"]` instead of bare names.
pub(super) fn lua_global_ref(name: &str) -> String {
    format!("_G[\"{}\"]", escape_lua_string(name))
}

/// Generate a unique ID (delegates to shared atomic counter).
fn rand_id() -> u64 {
    crate::loader::helpers::rand_id()
}
