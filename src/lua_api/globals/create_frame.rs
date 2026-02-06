//! CreateFrame implementation for creating WoW frames from Lua.

use super::super::frame::FrameHandle;
use super::super::SimState;
use super::template::apply_templates_from_registry;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Create the CreateFrame Lua function.
pub fn create_frame_function(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let (frame_type, name, parent_id, template) = parse_create_frame_args(lua, &args, &state_clone)?;
        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame_id = register_new_frame(&state_clone, widget_type, name.clone(), parent_id);

        // Create default children for widget types that always need them
        create_widget_type_defaults(&mut state_clone.borrow_mut(), frame_id, widget_type);

        let ud = create_frame_userdata(lua, &state_clone, frame_id, name.as_deref())?;

        // Apply templates from the registry (if template specified)
        if let Some(tmpl) = template {
            let ref_name = name.unwrap_or_else(|| format!("__frame_{}", frame_id));
            apply_templates_from_registry(lua, &ref_name, &tmpl);
        }

        Ok(ud)
    })?;
    Ok(create_frame)
}

/// Parse the arguments to CreateFrame: (frameType, name, parent, template).
fn parse_create_frame_args(
    lua: &Lua,
    args: &mlua::MultiValue,
    state: &Rc<RefCell<SimState>>,
) -> Result<(String, Option<String>, Option<u64>, Option<String>)> {
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

    // Default to UIParent if no parent specified
    if parent_id.is_none() {
        parent_id = state.borrow().widgets.get_id_by_name("UIParent");
    }

    // Handle $parent/$Parent name substitution
    let name = name_raw.map(|n| substitute_parent_name(n, parent_id, state));

    Ok((frame_type, name, parent_id, template))
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
fn register_new_frame(
    state: &Rc<RefCell<SimState>>,
    widget_type: WidgetType,
    name: Option<String>,
    parent_id: Option<u64>,
) -> u64 {
    let frame = Frame::new(widget_type, name, parent_id);
    let frame_id = frame.id;

    let mut state = state.borrow_mut();
    state.widgets.register(frame);

    if let Some(pid) = parent_id {
        state.widgets.add_child(pid, frame_id);

        // Inherit strata and level from parent (like wowless does)
        let parent_props = state.widgets.get(pid).map(|p| (p.frame_strata, p.frame_level));
        if let Some((parent_strata, parent_level)) = parent_props {
            if let Some(f) = state.widgets.get_mut(frame_id) {
                f.frame_strata = parent_strata;
                f.frame_level = parent_level + 1;
            }
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

    // Text fontstring for button label
    let mut text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
    text_fs.anchors.push(crate::widget::Anchor {
        point: crate::widget::AnchorPoint::Center,
        relative_to: None,
        relative_to_id: Some(frame_id as usize),
        relative_point: crate::widget::AnchorPoint::Center,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    text_fs.draw_layer = crate::widget::DrawLayer::Overlay;
    let text_id = text_fs.id;
    state.widgets.register(text_fs);
    state.widgets.add_child(frame_id, text_id);

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

/// Create a child widget of the given type, register it, and add it as a child. Returns the ID.
fn create_child_widget(state: &mut SimState, widget_type: WidgetType, parent_id: u64) -> u64 {
    let child = Frame::new(widget_type, None, Some(parent_id));
    let child_id = child.id;
    state.widgets.register(child);
    state.widgets.add_child(parent_id, child_id);
    child_id
}
