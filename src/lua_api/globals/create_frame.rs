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
        let mut args_iter = args.into_iter();

        let frame_type: String = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Frame".to_string());

        let name_raw: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        let parent_arg = args_iter.next();
        let parent_id: Option<u64> = parent_arg.as_ref().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });

        // Get template parameter (4th argument)
        let template: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        // Get parent ID (default to UIParent)
        let parent_id = parent_id.or_else(|| {
            let state = state_clone.borrow();
            state.widgets.get_id_by_name("UIParent")
        });

        // Handle $parent/$Parent name substitution
        let name: Option<String> = name_raw.map(|n| {
            if n.contains("$parent") || n.contains("$Parent") {
                if let Some(pid) = parent_id {
                    let state = state_clone.borrow();
                    if let Some(parent_name) = state.widgets.get(pid).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n.replace("$parent", "").replace("$Parent", "")
                }
            } else {
                n
            }
        });

        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame = Frame::new(widget_type, name.clone(), parent_id);
        let frame_id = frame.id;

        let mut state = state_clone.borrow_mut();
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

        // Create default children for widget types that always need them
        // These are fundamental to the widget type, not template-specific
        create_widget_type_defaults(&mut state, frame_id, widget_type);

        drop(state); // Release borrow before Lua operations

        // Create userdata handle
        let handle = FrameHandle {
            id: frame_id,
            state: Rc::clone(&state_clone),
        };

        let ud = lua.create_userdata(handle)?;

        // Store reference in globals if named
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        // Store reference for event dispatch
        let frame_key = format!("__frame_{}", frame_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        // Apply templates from the registry (if template specified)
        if let Some(ref tmpl) = template {
            if let Some(ref frame_name) = name {
                apply_templates_from_registry(lua, frame_name, tmpl);
            }
        }

        Ok(ud)
    })?;
    Ok(create_frame)
}

/// Create default children for widget types that fundamentally need them.
/// This is separate from templates - these are intrinsic to the widget type.
fn create_widget_type_defaults(state: &mut SimState, frame_id: u64, widget_type: WidgetType) {
    match widget_type {
        WidgetType::Button | WidgetType::CheckButton => {
            // Buttons implicitly have mouse enabled in WoW
            if let Some(frame) = state.widgets.get_mut(frame_id) {
                frame.mouse_enabled = true;
            }
            // Buttons always have these texture slots available
            let normal_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let normal_id = normal_tex.id;
            state.widgets.register(normal_tex);
            state.widgets.add_child(frame_id, normal_id);

            let pushed_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let pushed_id = pushed_tex.id;
            state.widgets.register(pushed_tex);
            state.widgets.add_child(frame_id, pushed_id);

            let highlight_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let highlight_id = highlight_tex.id;
            state.widgets.register(highlight_tex);
            state.widgets.add_child(frame_id, highlight_id);

            let disabled_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let disabled_id = disabled_tex.id;
            state.widgets.register(disabled_tex);
            state.widgets.add_child(frame_id, disabled_id);

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
                btn.children_keys
                    .insert("NormalTexture".to_string(), normal_id);
                btn.children_keys
                    .insert("PushedTexture".to_string(), pushed_id);
                btn.children_keys
                    .insert("HighlightTexture".to_string(), highlight_id);
                btn.children_keys
                    .insert("DisabledTexture".to_string(), disabled_id);
                btn.children_keys.insert("Text".to_string(), text_id);
            }
        }

        WidgetType::GameTooltip => {
            // Initialize tooltip data and set TOOLTIP strata
            state.tooltips.insert(frame_id, crate::lua_api::tooltip::TooltipData::default());
            if let Some(frame) = state.widgets.get_mut(frame_id) {
                frame.frame_strata = crate::widget::FrameStrata::Tooltip;
                frame.has_fixed_frame_strata = true;
            }
        }

        WidgetType::Slider => {
            // Sliders have Low/High/Text fontstrings and ThumbTexture
            let low_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let low_id = low_fs.id;
            state.widgets.register(low_fs);
            state.widgets.add_child(frame_id, low_id);

            let high_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let high_id = high_fs.id;
            state.widgets.register(high_fs);
            state.widgets.add_child(frame_id, high_id);

            let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let text_id = text_fs.id;
            state.widgets.register(text_fs);
            state.widgets.add_child(frame_id, text_id);

            let thumb_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let thumb_id = thumb_tex.id;
            state.widgets.register(thumb_tex);
            state.widgets.add_child(frame_id, thumb_id);

            if let Some(slider) = state.widgets.get_mut(frame_id) {
                slider.children_keys.insert("Low".to_string(), low_id);
                slider.children_keys.insert("High".to_string(), high_id);
                slider.children_keys.insert("Text".to_string(), text_id);
                slider
                    .children_keys
                    .insert("ThumbTexture".to_string(), thumb_id);
            }
        }

        _ => {}
    }
}
