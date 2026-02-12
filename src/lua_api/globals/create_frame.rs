//! CreateFrame implementation for creating WoW frames from Lua.

use super::super::frame::FrameHandle;
use super::super::SimState;
use super::template::{apply_templates_from_registry, fire_on_load};
use crate::loader::helpers::lua_global_ref;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Create the CreateFrame Lua function.
pub fn create_frame_function(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let (frame_type, name, parent_id, template, id) = parse_create_frame_args(lua, &args, &state_clone)?;
        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame_id = register_new_frame(&state_clone, widget_type, name.clone(), parent_id);

        // Apply the 5th argument (frame ID) if provided
        if let Some(frame_lua_id) = id {
            if let Some(frame) = state_clone.borrow_mut().widgets.get_mut(frame_id) {
                frame.user_id = frame_lua_id;
            }
        }

        // Create default children for widget types that always need them
        create_widget_type_defaults(&mut state_clone.borrow_mut(), frame_id, widget_type);

        // ItemButton has intrinsic children (Count, icon, etc.) from WoW's intrinsic template
        if frame_type == "ItemButton" {
            create_item_button_intrinsics(&mut state_clone.borrow_mut(), frame_id);
        }

        let ud = create_frame_userdata(lua, &state_clone, frame_id, name.as_deref())?;

        // WoW registers named button's default texture children as globals:
        // ButtonNameNormalTexture, ButtonNamePushedTexture, etc.
        if matches!(widget_type, WidgetType::Button | WidgetType::CheckButton)
            && let Some(ref btn_name) = name {
                register_button_child_globals(lua, &state_clone, frame_id, btn_name)?;
            }

        // ItemButton intrinsic template defines mixin="ItemButtonMixin"
        if frame_type == "ItemButton" {
            let frame_key = format!("__frame_{}", frame_id);
            let code = format!(
                "do local f = {} if f and ItemButtonMixin then Mixin(f, ItemButtonMixin) end end",
                lua_global_ref(&frame_key)
            );
            let _ = lua.load(&code).exec();
        }

        // Apply intrinsic template if the frame type is registered as one.
        // This handles types like ContainedAlertFrame, EventButton, etc. whose
        // intrinsic XML definition (mixin, scripts, children) should be applied
        // automatically when created via CreateFrame/CreateFramePool.
        let intrinsic_entry = crate::xml::get_template(&frame_type);
        let ref_name = name.unwrap_or_else(|| format!("__frame_{}", frame_id));
        if let Some(entry) = &intrinsic_entry {
            // Use the canonical template name (PascalCase from XML definition),
            // not the raw Lua input which may be all-caps (e.g. "DROPDOWNBUTTON").
            let canonical = &entry.name;
            apply_templates_from_registry(lua, &ref_name, canonical);
            // Set frame.intrinsic = "TypeName" BEFORE user templates, so OnLoad
            // handlers (e.g. ValidateIsDropdownButtonIntrinsic) can see it.
            let code = format!(
                "{}.intrinsic = \"{}\"",
                lua_global_ref(&ref_name),
                canonical
            );
            let _ = lua.load(&code).exec();
        }

        // Apply user-specified templates from the registry
        if let Some(ref tmpl) = template {
            apply_templates_from_registry(lua, &ref_name, tmpl);

            // If any template in the chain defines parentArray, insert this frame
            // into its parent's array.  This handles dynamic CreateFrame calls like
            // CreateFrame("Frame", nil, self, "PreMatchArenaUnitFrameTemplate")
            // where PreMatchArenaUnitFrameTemplate has parentArray="preMatchUnitFrames".
            if parent_id.is_some() {
                apply_parent_array_from_template(lua, tmpl, frame_id, &ref_name);
            }
        }


        // Fire OnLoad on the frame itself (WoW fires OnLoad before CreateFrame returns).
        // Skip when the XML loader is in charge — it fires OnLoad via
        // fire_lifecycle_scripts after all inline properties (KeyValues, scripts,
        // layers) are applied.  Without this, OnLoad fires before inline content
        // is set, causing nil-reference errors (e.g. ActionBar numButtons).
        let suppress_depth: i32 = lua.globals().get("__suppress_create_frame_onload")
            .unwrap_or(0);
        if suppress_depth <= 0 {
            fire_on_load(lua, &ref_name);
        }

        Ok(ud)
    })?;
    Ok(create_frame)
}

/// Parse the arguments to CreateFrame: (frameType, name, parent, template, id).
#[allow(clippy::type_complexity)]
fn parse_create_frame_args(
    lua: &Lua,
    args: &mlua::MultiValue,
    state: &Rc<RefCell<SimState>>,
) -> Result<(String, Option<String>, Option<u64>, Option<String>, Option<i32>)> {
    let mut args_iter = args.iter();

    let frame_type: String = args_iter
        .next()
        .and_then(|v| lua.coerce_string(v.clone()).ok().flatten())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Frame".to_string());

    let name_raw: Option<String> = args_iter
        .next()
        .and_then(|v| lua.coerce_string(v.clone()).ok().flatten())
        .map(|s| s.to_string_lossy().to_string());

    let parent_arg = args_iter.next();
    let mut parent_id: Option<u64> = parent_arg.and_then(|v| {
        if let Value::UserData(ud) = v {
            ud.borrow::<FrameHandle>().ok().map(|h| h.id)
        } else {
            None
        }
    });

    let template: Option<String> = args_iter
        .next()
        .and_then(|v| lua.coerce_string(v.clone()).ok().flatten())
        .map(|s| s.to_string_lossy().to_string());

    // 5th arg: numeric frame ID
    let id: Option<i32> = args_iter.next().and_then(|v| match v {
        Value::Integer(n) => Some(*n as i32),
        Value::Number(n) => Some(*n as i32),
        _ => None,
    });

    // Default to UIParent if no parent specified
    if parent_id.is_none() {
        parent_id = state.borrow().widgets.get_id_by_name("UIParent");
    }

    // Handle $parent/$Parent name substitution
    let name = name_raw.map(|n| substitute_parent_name(n, parent_id, state));

    Ok((frame_type, name, parent_id, template, id))
}

/// Replace $parent/$Parent placeholders in a frame name with the actual parent name.
fn substitute_parent_name(
    name: String,
    parent_id: Option<u64>,
    state: &Rc<RefCell<SimState>>,
) -> String {
    if !name.contains("$parent") && !name.contains("$Parent") {
        return name;
    }
    if let Some(pid) = parent_id {
        let state = state.borrow();
        if let Some(parent_name) = state.widgets.get(pid).and_then(|f| f.name.clone()) {
            return name.replace("$parent", &parent_name)
                .replace("$Parent", &parent_name);
        }
    }
    name.replace("$parent", "").replace("$Parent", "")
}

/// Register a new frame in the widget registry and set up parent-child relationship.
/// If a named frame already exists, orphan the old one (remove from parent's children and hide).
fn register_new_frame(
    state: &Rc<RefCell<SimState>>,
    widget_type: WidgetType,
    name: Option<String>,
    parent_id: Option<u64>,
) -> u64 {
    let mut frame = Frame::new(widget_type, name.clone(), parent_id);

    // Attribute frame to the addon currently being loaded, or inherit from parent.
    {
        let s = state.borrow();
        frame.owner_addon = s.loading_addon_index.or_else(|| {
            parent_id.and_then(|pid| s.widgets.get(pid).and_then(|p| p.owner_addon))
        });
    }

    let frame_id = frame.id;

    let mut state = state.borrow_mut();

    // If a frame with this name already exists, orphan it (WoW behavior: old frame
    // becomes unreachable via global, but still exists in the registry).
    let old_same_name = name.as_ref()
        .and_then(|n| state.widgets.get_id_by_name(n));
    if let Some(old_id) = old_same_name {
        orphan_old_frame(&mut state.widgets, old_id);
    }

    state.widgets.register(frame);

    // Migrate children AFTER register so the new frame exists in the registry.
    if let Some(old_id) = old_same_name {
        migrate_children_to_new_frame(&mut state.widgets, old_id, frame_id);
    }

    if let Some(pid) = parent_id {
        state.widgets.add_child(pid, frame_id);

        // Inherit strata and level from parent (like wowless does)
        let parent_props = state.widgets.get(pid).map(|p| (p.frame_strata, p.frame_level));
        if let Some((parent_strata, parent_level)) = parent_props
            && let Some(f) = state.widgets.get_mut(frame_id) {
                f.frame_strata = parent_strata;
                f.frame_level = parent_level + 1;
            }
    }

    frame_id
}

/// Create the Lua userdata handle and register it in globals.
fn create_frame_userdata(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    name: Option<&str>,
) -> Result<mlua::AnyUserData> {
    let handle = FrameHandle {
        id: frame_id,
        state: Rc::clone(state),
    };
    let ud = lua.create_userdata(handle)?;

    if let Some(n) = name {
        lua.globals().set(n, ud.clone())?;
    }

    let frame_key = format!("__frame_{}", frame_id);
    lua.globals().set(frame_key.as_str(), ud.clone())?;

    Ok(ud)
}

/// Register button's default texture children as Lua globals.
/// In WoW, named buttons get globals like `ButtonNameNormalTexture`, etc.
fn register_button_child_globals(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    button_name: &str,
) -> Result<()> {
    let keys: Vec<(String, u64)> = {
        let st = state.borrow();
        let Some(btn) = st.widgets.get(frame_id) else { return Ok(()) };
        ["NormalTexture", "PushedTexture", "HighlightTexture", "DisabledTexture", "Text"]
            .iter()
            .filter_map(|key| {
                btn.children_keys.get(*key).map(|&id| (key.to_string(), id))
            })
            .collect()
    };
    let globals = lua.globals();
    for (key, child_id) in keys {
        let global_name = format!("{}{}", button_name, key);
        let handle = FrameHandle { id: child_id, state: Rc::clone(state) };
        let ud = lua.create_userdata(handle)?;
        globals.set(global_name.as_str(), ud)?;
    }
    Ok(())
}

/// Create default children for widget types that fundamentally need them.
/// This is separate from templates - these are intrinsic to the widget type.
fn create_widget_type_defaults(state: &mut SimState, frame_id: u64, widget_type: WidgetType) {
    match widget_type {
        WidgetType::Button | WidgetType::CheckButton => {
            create_button_defaults(state, frame_id);
        }
        WidgetType::GameTooltip => {
            create_tooltip_defaults(state, frame_id);
        }
        WidgetType::SimpleHTML => {
            state.simple_htmls.insert(frame_id, crate::lua_api::simple_html::SimpleHtmlData::default());
        }
        WidgetType::MessageFrame => {
            state.message_frames.insert(frame_id, crate::lua_api::message_frame::MessageFrameData::default());
        }
        WidgetType::Slider => {
            create_slider_defaults(state, frame_id);
        }
        WidgetType::EditBox => {
            if let Some(frame) = state.widgets.get_mut(frame_id) {
                frame.mouse_enabled = true;
            }
        }
        _ => {}
    }
}

/// Create default texture slots and text fontstring for Button/CheckButton.
fn create_button_defaults(state: &mut SimState, frame_id: u64) {
    if let Some(frame) = state.widgets.get_mut(frame_id) {
        frame.mouse_enabled = true;
    }

    let normal_id = create_child_widget(state, WidgetType::Texture, frame_id);
    let pushed_id = create_child_widget(state, WidgetType::Texture, frame_id);
    let highlight_id = create_child_widget(state, WidgetType::Texture, frame_id);
    let disabled_id = create_child_widget(state, WidgetType::Texture, frame_id);

    // Button textures fill the parent by default (SetAllPoints equivalent)
    for tex_id in [normal_id, pushed_id, highlight_id, disabled_id] {
        if let Some(tex) = state.widgets.get_mut(tex_id) {
            add_fill_parent_anchors(tex, frame_id);
        }
    }

    // Button state textures render below child regions (icon, etc.) in WoW.
    // Set to Background layer so they don't occlude Border/Artwork children.
    for tex_id in [normal_id, pushed_id, disabled_id] {
        if let Some(tex) = state.widgets.get_mut(tex_id) {
            tex.draw_layer = crate::widget::DrawLayer::Background;
        }
    }

    // HighlightTexture is only visible on hover in WoW, uses additive blending
    if let Some(highlight) = state.widgets.get_mut(highlight_id) {
        highlight.draw_layer = crate::widget::DrawLayer::Highlight;
        highlight.visible = false;
        highlight.blend_mode = crate::render::BlendMode::Additive;
    }

    // Text fontstring for button label — fill parent so it inherits the
    // button's width and renders at Overlay layer (above child textures
    // like three-slice Background-layer Left/Right/Center).
    let text_id = create_child_widget(state, WidgetType::FontString, frame_id);
    if let Some(text_fs) = state.widgets.get_mut(text_id) {
        add_fill_parent_anchors(text_fs, frame_id);
        text_fs.draw_layer = crate::widget::DrawLayer::Overlay;
    }

    if let Some(btn) = state.widgets.get_mut(frame_id) {
        btn.children_keys.insert("NormalTexture".to_string(), normal_id);
        btn.children_keys.insert("PushedTexture".to_string(), pushed_id);
        btn.children_keys.insert("HighlightTexture".to_string(), highlight_id);
        btn.children_keys.insert("DisabledTexture".to_string(), disabled_id);
        btn.children_keys.insert("Text".to_string(), text_id);
    }
}

/// Create default tooltip state and set TOOLTIP strata.
fn create_tooltip_defaults(state: &mut SimState, frame_id: u64) {
    state.tooltips.insert(frame_id, crate::lua_api::tooltip::TooltipData::default());
    if let Some(frame) = state.widgets.get_mut(frame_id) {
        frame.frame_strata = crate::widget::FrameStrata::Tooltip;
        frame.has_fixed_frame_strata = true;
    }
}

/// Create default fontstrings and thumb texture for Slider.
fn create_slider_defaults(state: &mut SimState, frame_id: u64) {
    let low_id = create_child_widget(state, WidgetType::FontString, frame_id);
    let high_id = create_child_widget(state, WidgetType::FontString, frame_id);
    let text_id = create_child_widget(state, WidgetType::FontString, frame_id);
    let thumb_id = create_child_widget(state, WidgetType::Texture, frame_id);

    if let Some(slider) = state.widgets.get_mut(frame_id) {
        slider.children_keys.insert("Low".to_string(), low_id);
        slider.children_keys.insert("High".to_string(), high_id);
        slider.children_keys.insert("Text".to_string(), text_id);
        slider.children_keys.insert("ThumbTexture".to_string(), thumb_id);
    }
}

/// Add TOPLEFT+BOTTOMRIGHT anchors to fill the parent (equivalent to SetAllPoints).
fn add_fill_parent_anchors(frame: &mut Frame, parent_id: u64) {
    use crate::widget::{Anchor, AnchorPoint};
    frame.anchors.push(Anchor {
        point: AnchorPoint::TopLeft,
        relative_to: None,
        relative_to_id: Some(parent_id as usize),
        relative_point: AnchorPoint::TopLeft,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    frame.anchors.push(Anchor {
        point: AnchorPoint::BottomRight,
        relative_to: None,
        relative_to_id: Some(parent_id as usize),
        relative_point: AnchorPoint::BottomRight,
        x_offset: 0.0,
        y_offset: 0.0,
    });
}

/// Remove old frame from its parent's children and hide it.
fn orphan_old_frame(widgets: &mut crate::widget::WidgetRegistry, old_id: u64) {
    if let Some(old_frame) = widgets.get(old_id)
        && let Some(old_parent_id) = old_frame.parent_id
            && let Some(old_parent) = widgets.get_mut(old_parent_id) {
                old_parent.children.retain(|&c| c != old_id);
            }
    if let Some(old_frame) = widgets.get_mut(old_id) {
        old_frame.visible = false;
    }
}

/// Move all children from an old frame to a new replacement frame.
///
/// When a named frame is re-created (e.g. UIParent defined in XML replaces the
/// pre-built one), frames that were parented to the old version need to be
/// reparented to the new one so they remain in the live visibility tree.
fn migrate_children_to_new_frame(
    widgets: &mut crate::widget::WidgetRegistry,
    old_id: u64,
    new_id: u64,
) {
    let children: Vec<u64> = widgets.get(old_id)
        .map(|f| f.children.clone())
        .unwrap_or_default();
    for &child_id in &children {
        if let Some(child) = widgets.get_mut(child_id) {
            child.parent_id = Some(new_id);
        }
    }
    // Move children_keys too (e.g. NineSlice for tooltips)
    let keys: std::collections::HashMap<String, u64> = widgets.get(old_id)
        .map(|f| f.children_keys.clone())
        .unwrap_or_default();
    if let Some(new_frame) = widgets.get_mut(new_id) {
        new_frame.children.extend(&children);
        for (k, v) in keys {
            new_frame.children_keys.entry(k).or_insert(v);
        }
    }
    if let Some(old_frame) = widgets.get_mut(old_id) {
        old_frame.children.clear();
        old_frame.children_keys.clear();
    }
}

/// Create a child widget of the given type, register it, and add it as a child. Returns the ID.
fn create_child_widget(state: &mut SimState, widget_type: WidgetType, parent_id: u64) -> u64 {
    let child = Frame::new(widget_type, None, Some(parent_id));
    let child_id = child.id;
    state.widgets.register(child);
    state.widgets.add_child(parent_id, child_id);
    // Inherit strata and level from parent
    let parent_props = state.widgets.get(parent_id).map(|p| (p.frame_strata, p.frame_level));
    if let Some((parent_strata, parent_level)) = parent_props {
        if let Some(f) = state.widgets.get_mut(child_id) {
            f.frame_strata = parent_strata;
            f.frame_level = parent_level + 1;
        }
    }
    child_id
}

/// Create intrinsic children for ItemButton (from WoW's intrinsic="true" template).
/// ItemButton defines: icon (Texture), Count (FontString), Stock (FontString),
/// searchOverlay, ItemContextOverlay, IconBorder, IconOverlay, IconOverlay2 (Textures).
fn create_item_button_intrinsics(state: &mut SimState, frame_id: u64) {
    // icon texture (BORDER layer, fills parent)
    let icon_id = create_child_widget(state, WidgetType::Texture, frame_id);
    if let Some(tex) = state.widgets.get_mut(icon_id) {
        tex.draw_layer = crate::widget::DrawLayer::Border;
        add_fill_parent_anchors(tex, frame_id);
    }

    // Count fontstring (ARTWORK layer, hidden, anchored BOTTOMRIGHT)
    let count_id = create_child_widget(state, WidgetType::FontString, frame_id);
    if let Some(fs) = state.widgets.get_mut(count_id) {
        fs.draw_layer = crate::widget::DrawLayer::Artwork;
        fs.visible = false;
        fs.justify_h = crate::widget::TextJustify::Right;
        fs.anchors.push(crate::widget::Anchor {
            point: crate::widget::AnchorPoint::BottomRight,
            relative_to: None,
            relative_to_id: Some(frame_id as usize),
            relative_point: crate::widget::AnchorPoint::BottomRight,
            x_offset: -5.0,
            y_offset: -2.0,
        });
    }

    // Stock fontstring (ARTWORK layer, hidden)
    let stock_id = create_child_widget(state, WidgetType::FontString, frame_id);
    if let Some(fs) = state.widgets.get_mut(stock_id) {
        fs.draw_layer = crate::widget::DrawLayer::Artwork;
        fs.visible = false;
    }

    // IconBorder, IconOverlay, IconOverlay2 (OVERLAY layer, hidden)
    let icon_border_id = create_hidden_overlay(state, frame_id);
    let icon_overlay_id = create_hidden_overlay(state, frame_id);
    let icon_overlay2_id = create_hidden_overlay(state, frame_id);

    // searchOverlay (OVERLAY layer, hidden, fills parent)
    let search_overlay_id = create_child_widget(state, WidgetType::Texture, frame_id);
    if let Some(tex) = state.widgets.get_mut(search_overlay_id) {
        tex.draw_layer = crate::widget::DrawLayer::Overlay;
        tex.visible = false;
        add_fill_parent_anchors(tex, frame_id);
    }

    // ItemContextOverlay (OVERLAY layer, hidden)
    let context_overlay_id = create_hidden_overlay(state, frame_id);

    if let Some(btn) = state.widgets.get_mut(frame_id) {
        btn.children_keys.insert("icon".to_string(), icon_id);
        btn.children_keys.insert("Count".to_string(), count_id);
        btn.children_keys.insert("Stock".to_string(), stock_id);
        btn.children_keys.insert("IconBorder".to_string(), icon_border_id);
        btn.children_keys.insert("IconOverlay".to_string(), icon_overlay_id);
        btn.children_keys.insert("IconOverlay2".to_string(), icon_overlay2_id);
        btn.children_keys.insert("searchOverlay".to_string(), search_overlay_id);
        btn.children_keys.insert("ItemContextOverlay".to_string(), context_overlay_id);
    }
}

/// Check the template chain for a `parentArray` attribute and insert the frame
/// into its parent's Lua array if found.
fn apply_parent_array_from_template(lua: &Lua, template_names: &str, _frame_id: u64, ref_name: &str) {
    let chain = crate::xml::get_template_chain(template_names);
    for entry in &chain {
        if let Some(parent_array) = &entry.frame.parent_array {
            let frame_ref = lua_global_ref(ref_name);
            let code = format!(
                "do local child = {frame_ref}\n\
                 if child then\n\
                     local parent = child:GetParent()\n\
                     if parent then\n\
                         parent[\"{parent_array}\"] = parent[\"{parent_array}\"] or {{}}\n\
                         table.insert(parent[\"{parent_array}\"], child)\n\
                     end\n\
                 end\nend",
            );
            let _ = lua.load(&code).exec();
            break;
        }
    }
}

/// Create a hidden overlay texture child (OVERLAY layer, hidden, centered on parent).
fn create_hidden_overlay(state: &mut SimState, parent_id: u64) -> u64 {
    let id = create_child_widget(state, WidgetType::Texture, parent_id);
    if let Some(tex) = state.widgets.get_mut(id) {
        tex.draw_layer = crate::widget::DrawLayer::Overlay;
        tex.visible = false;
        tex.anchors.push(crate::widget::Anchor {
            point: crate::widget::AnchorPoint::Center,
            relative_to: None,
            relative_to_id: Some(parent_id as usize),
            relative_point: crate::widget::AnchorPoint::Center,
            x_offset: 0.0,
            y_offset: 0.0,
        });
    }
    id
}
