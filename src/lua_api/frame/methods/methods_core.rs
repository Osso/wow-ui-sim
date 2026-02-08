//! Core frame methods: GetName, SetSize, Show/Hide, strata/level, mouse, scale, rect.

use super::FrameHandle;
use super::methods_helpers::{calculate_frame_height, calculate_frame_width};
use crate::lua_api::layout::compute_frame_rect;
use crate::lua_api::SimState;
use mlua::{Lua, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Read screen dimensions from SimState.
fn screen_dims(state: &SimState) -> (f32, f32) {
    (state.screen_width, state.screen_height)
}

/// Add core frame methods to FrameHandle UserData.
pub fn add_core_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_identity_methods(methods);
    add_size_methods(methods);
    add_rect_methods(methods);
    add_visibility_methods(methods);
    add_strata_level_methods(methods);
    add_mouse_input_methods(methods);
    add_scale_methods(methods);
    add_region_query_methods(methods);
}

/// Identity methods: GetName, GetObjectType
fn add_identity_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetName() — returns nil for unnamed frames (WoW behavior)
    methods.add_method("GetName", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).and_then(|f| f.name.clone()))
    });

    // GetDebugName() — returns name, or parentKey path, or object type
    methods.add_method("GetDebugName", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(ref name) = frame.name {
                return Ok(name.clone());
            }
            // Try parentKey from parent's children_keys
            if let Some(pid) = frame.parent_id {
                if let Some(parent) = state.widgets.get(pid) {
                    for (key, &cid) in &parent.children_keys {
                        if cid == this.id {
                            let parent_name = parent.name.as_deref().unwrap_or("?");
                            return Ok(format!("{}.{}", parent_name, key));
                        }
                    }
                }
            }
            return Ok(format!("[{}]", frame.widget_type.as_str()));
        }
        Ok("[Unknown]".to_string())
    });

    // GetObjectType()
    methods.add_method("GetObjectType", |_, this, ()| {
        let state = this.state.borrow();
        let obj_type = state
            .widgets
            .get(this.id)
            .map(|f| f.widget_type.as_str())
            .unwrap_or("Frame");
        Ok(obj_type.to_string())
    });

    // IsObjectType(type) - Check if object is or inherits from a type
    methods.add_method("IsObjectType", |_, this, type_name: String| {
        use crate::widget::WidgetType;
        let state = this.state.borrow();
        let wt = state
            .widgets
            .get(this.id)
            .map(|f| f.widget_type)
            .unwrap_or(WidgetType::Frame);
        Ok(widget_type_is_a(wt, &type_name))
    });
}

/// Size methods: GetWidth, GetHeight, GetSize, SetWidth, SetHeight, SetSize
fn add_size_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetWidth() - returns explicit width or calculates from anchors
    methods.add_method("GetWidth", |_, this, ()| {
        let state = this.state.borrow();
        Ok(calculate_frame_width(&state.widgets, this.id))
    });

    // GetHeight() - returns explicit height or calculates from anchors
    methods.add_method("GetHeight", |_, this, ()| {
        let state = this.state.borrow();
        Ok(calculate_frame_height(&state.widgets, this.id))
    });

    // GetSize() -> width, height (with anchor calculation)
    methods.add_method("GetSize", |_, this, ()| {
        let state = this.state.borrow();
        let width = calculate_frame_width(&state.widgets, this.id);
        let height = calculate_frame_height(&state.widgets, this.id);
        Ok((width, height))
    });

    // SetSize(width, height)
    methods.add_method("SetSize", |_, this, (width, height): (f32, f32)| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.set_size(width, height);
        }
        Ok(())
    });

    // SetWidth(width)
    methods.add_method("SetWidth", |_, this, width: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.width = width;
        }
        Ok(())
    });

    // SetHeight(height)
    methods.add_method("SetHeight", |_, this, height: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.height = height;
        }
        Ok(())
    });
}

/// Compute effective scale by walking up the parent chain.
fn effective_scale(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    let mut scale = 1.0f32;
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        if let Some(f) = widgets.get(cid) {
            scale *= f.scale;
            current_id = f.parent_id;
        } else {
            break;
        }
    }
    scale
}

/// Rect/position methods: GetRect, GetScaledRect, GetLeft, GetRight, GetTop, GetBottom,
/// GetCenter, GetBounds.
///
/// WoW coordinate system: origin at bottom-left, Y increases upward.
/// `compute_frame_rect` returns top-left origin (screen coords, Y-down), so we convert.
fn add_rect_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_rect_full_methods(methods);
    add_rect_edge_methods(methods);
}

/// GetRect, GetScaledRect, GetBounds
fn add_rect_full_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetRect() -> left, bottom, width, height (unscaled, bottom-left origin)
    methods.add_method("GetRect", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        let bottom = screen_height - rect.y - rect.height;
        Ok((rect.x, bottom, rect.width, rect.height))
    });

    // GetScaledRect() -> left, bottom, width, height (scaled by effective scale)
    methods.add_method("GetScaledRect", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        let scale = effective_scale(&state.widgets, this.id);
        let left = rect.x * scale;
        let bottom = (screen_height - rect.y - rect.height) * scale;
        Ok((left, bottom, rect.width * scale, rect.height * scale))
    });

    // GetBounds() -> left, bottom, width, height (same as GetRect in practice)
    methods.add_method("GetBounds", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        let bottom = screen_height - rect.y - rect.height;
        Ok((rect.x, bottom, rect.width, rect.height))
    });
}

/// GetLeft, GetRight, GetTop, GetBottom, GetCenter
fn add_rect_edge_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetLeft", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        Ok(rect.x)
    });

    methods.add_method("GetRight", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        Ok(rect.x + rect.width)
    });

    methods.add_method("GetTop", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        Ok(screen_height - rect.y)
    });

    methods.add_method("GetBottom", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        Ok(screen_height - rect.y - rect.height)
    });

    methods.add_method("GetCenter", |_, this, ()| {
        let state = this.state.borrow();
        let (screen_width, screen_height) = screen_dims(&state);
        let rect = compute_frame_rect(
            &state.widgets, this.id, screen_width, screen_height,
        );
        let cx = rect.x + rect.width / 2.0;
        let cy = screen_height - rect.y - rect.height / 2.0;
        Ok((cx, cy))
    });
}

/// Fire OnShow on a frame and recursively on its visible children.
pub(crate) fn fire_on_show_recursive(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    id: u64,
) -> mlua::Result<()> {
    // Fire OnShow on this frame
    if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, id, "OnShow") {
        if let Some(frame_ud) = crate::lua_api::script_helpers::get_frame_ref(lua, id) {
            if let Err(e) = handler.call::<()>(frame_ud) {
                crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
            }
        }
    }

    // Collect visible children (borrow scoped)
    let children: Vec<u64> = {
        let st = state.borrow();
        st.widgets
            .get(id)
            .map(|f| {
                f.children
                    .iter()
                    .filter(|&&cid| st.widgets.get(cid).map(|c| c.visible).unwrap_or(false))
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    };

    for child_id in children {
        fire_on_show_recursive(lua, state, child_id)?;
    }

    Ok(())
}

/// Visibility methods: Show, Hide, IsVisible, IsShown, SetShown
fn add_visibility_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Show()
    methods.add_method("Show", |lua, this, ()| {
        let was_hidden = {
            let state = this.state.borrow();
            state
                .widgets
                .get(this.id)
                .map(|f| !f.visible)
                .unwrap_or(false)
        };

        {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = true;
            }
        }

        // Fire OnShow if transitioning from hidden to visible
        if was_hidden {
            fire_on_show_recursive(lua, &this.state, this.id)?;
        }
        Ok(())
    });

    // Hide()
    methods.add_method("Hide", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.visible = false;
        }
        Ok(())
    });

    // IsVisible() - true only if this frame AND all ancestors are shown
    methods.add_method("IsVisible", |_, this, ()| {
        let state = this.state.borrow();
        let mut id = this.id;
        loop {
            match state.widgets.get(id) {
                Some(f) if f.visible => match f.parent_id {
                    Some(pid) => id = pid,
                    None => return Ok(true),
                },
                _ => return Ok(false),
            }
        }
    });

    // IsShown() - true if this frame's own visible flag is set (ignores parents)
    methods.add_method("IsShown", |_, this, ()| {
        let state = this.state.borrow();
        let visible = state
            .widgets
            .get(this.id)
            .map(|f| f.visible)
            .unwrap_or(false);
        Ok(visible)
    });

    // SetShown(shown) - show/hide based on boolean
    methods.add_method("SetShown", |_, this, shown: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.visible = shown;
        }
        Ok(())
    });
}

/// Strata and level methods
fn add_strata_level_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_alpha_methods(methods);
    add_strata_methods(methods);
    add_level_methods(methods);

    // SetToplevel(toplevel) - Mark frame as toplevel (raises on click)
    methods.add_method("SetToplevel", |_, _this, _toplevel: bool| Ok(()));

    // IsToplevel()
    methods.add_method("IsToplevel", |_, _this, ()| Ok(false));

    // NOTE: Raise() and Lower() methods are handled in __index metamethod
    // to allow custom properties with these names to take precedence.
}

/// Alpha transparency methods.
fn add_alpha_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetAlpha", |_, this, alpha: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.alpha = alpha.clamp(0.0, 1.0);
        }
        Ok(())
    });

    methods.add_method("GetAlpha", |_, this, ()| {
        let state = this.state.borrow();
        let alpha = state.widgets.get(this.id).map(|f| f.alpha).unwrap_or(1.0);
        Ok(alpha)
    });

    // GetEffectiveAlpha() - walk parent chain multiplying alpha values
    methods.add_method("GetEffectiveAlpha", |_, this, ()| {
        let state = this.state.borrow();
        let mut alpha = 1.0f32;
        let mut current_id = Some(this.id);
        while let Some(id) = current_id {
            if let Some(f) = state.widgets.get(id) {
                alpha *= f.alpha;
                current_id = f.parent_id;
            } else {
                break;
            }
        }
        Ok(alpha)
    });

    // SetAlphaFromBoolean(flag) - set alpha to 1.0 if true, 0.0 if false
    methods.add_method("SetAlphaFromBoolean", |_, this, flag: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.alpha = if flag { 1.0 } else { 0.0 };
        }
        Ok(())
    });
}

/// Frame strata methods (major draw order).
fn add_strata_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFrameStrata", |_, this, strata: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id)
            && let Some(s) = crate::widget::FrameStrata::from_str(&strata) {
                frame.frame_strata = s;
                frame.has_fixed_frame_strata = true;
            }
        Ok(())
    });

    methods.add_method("GetFrameStrata", |_, this, ()| {
        let state = this.state.borrow();
        let strata = state
            .widgets
            .get(this.id)
            .map(|f| f.frame_strata.as_str())
            .unwrap_or("MEDIUM");
        Ok(strata.to_string())
    });

    methods.add_method("SetFixedFrameStrata", |_, this, fixed: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.has_fixed_frame_strata = fixed;
        }
        Ok(())
    });

    methods.add_method("HasFixedFrameStrata", |_, this, ()| {
        let state = this.state.borrow();
        let fixed = state
            .widgets
            .get(this.id)
            .map(|f| f.has_fixed_frame_strata)
            .unwrap_or(false);
        Ok(fixed)
    });
}

/// Frame level methods (draw order within strata).
fn add_level_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFrameLevel", |_, this, level: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.frame_level = level;
            frame.has_fixed_frame_level = true;
        }
        Ok(())
    });

    methods.add_method("GetFrameLevel", |_, this, ()| {
        let state = this.state.borrow();
        let level = state
            .widgets
            .get(this.id)
            .map(|f| f.frame_level)
            .unwrap_or(0);
        Ok(level)
    });

    methods.add_method("SetFixedFrameLevel", |_, this, fixed: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.has_fixed_frame_level = fixed;
        }
        Ok(())
    });

    methods.add_method("HasFixedFrameLevel", |_, this, ()| {
        let state = this.state.borrow();
        let fixed = state
            .widgets
            .get(this.id)
            .map(|f| f.has_fixed_frame_level)
            .unwrap_or(false);
        Ok(fixed)
    });
}

/// Mouse and input methods
fn add_mouse_input_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetID(id) - Set frame ID (used for tab ordering, etc.)
    methods.add_method("SetID", |_, this, id: i32| {
        this.state.borrow_mut().widgets.get_mut_silent(this.id).map(|f| f.user_id = id);
        Ok(())
    });

    // GetID() - Get frame ID
    methods.add_method("GetID", |_, this, ()| {
        Ok(this.state.borrow().widgets.get(this.id).map(|f| f.user_id).unwrap_or(0))
    });

    // GetMapID() / SetMapID() - Map canvas frame stubs
    methods.add_method("GetMapID", |_, _this, ()| Ok(0));
    methods.add_method("SetMapID", |_, _this, _map_id: i32| Ok(()));

    // EnableMouse(enable)
    methods.add_method("EnableMouse", |_, this, enable: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.mouse_enabled = enable;
        }
        Ok(())
    });

    // IsMouseEnabled()
    methods.add_method("IsMouseEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .widgets
            .get(this.id)
            .map(|f| f.mouse_enabled)
            .unwrap_or(false);
        Ok(enabled)
    });

    // EnableMouseWheel(enable) - enable mouse wheel events
    methods.add_method("EnableMouseWheel", |_, _this, _enable: bool| Ok(()));

    // IsMouseWheelEnabled()
    methods.add_method("IsMouseWheelEnabled", |_, _this, ()| Ok(false));

    // EnableKeyboard(enable) - enable keyboard input for frame
    methods.add_method("EnableKeyboard", |_, this, enable: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(f) = state.widgets.get_mut(this.id) {
            f.keyboard_enabled = enable;
        }
        Ok(())
    });

    // IsKeyboardEnabled()
    methods.add_method("IsKeyboardEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .widgets
            .get(this.id)
            .map(|f| f.keyboard_enabled)
            .unwrap_or(false);
        Ok(enabled)
    });

    // RegisterForMouse(button1, button2, ...) - register mouse buttons for click
    methods.add_method("RegisterForMouse", |_, _this, _args: mlua::MultiValue| Ok(()));

    add_mouse_motion_methods(methods);
}

/// Mouse motion and click enabled methods.
fn add_mouse_motion_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("EnableMouseMotion", |_, this, enable: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.mouse_motion_enabled = enable;
        }
        Ok(())
    });

    methods.add_method("IsMouseMotionEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .widgets
            .get(this.id)
            .map(|f| f.mouse_motion_enabled)
            .unwrap_or(false);
        Ok(enabled)
    });

    methods.add_method("SetMouseMotionEnabled", |_, this, enable: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.mouse_motion_enabled = enable;
        }
        Ok(())
    });

    methods.add_method("SetMouseClickEnabled", |_, this, enable: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.mouse_enabled = enable;
        }
        Ok(())
    });

    methods.add_method("IsMouseClickEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .widgets
            .get(this.id)
            .map(|f| f.mouse_enabled)
            .unwrap_or(false);
        Ok(enabled)
    });
}

/// Scale methods
fn add_scale_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetScale() - get frame's scale
    methods.add_method("GetScale", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.scale).unwrap_or(1.0))
    });

    // SetScale(scale) - set frame's scale factor (affects visible size)
    methods.add_method("SetScale", |_, this, scale: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(f) = state.widgets.get_mut(this.id) {
            f.scale = scale;
        }
        Ok(())
    });

    // GetEffectiveScale() - get product of all ancestor scales * this frame's scale
    methods.add_method("GetEffectiveScale", |_, this, ()| {
        let state = this.state.borrow();
        let mut scale = 1.0f32;
        let mut current_id = Some(this.id);
        while let Some(id) = current_id {
            if let Some(f) = state.widgets.get(id) {
                scale *= f.scale;
                current_id = f.parent_id;
            } else {
                break;
            }
        }
        Ok(scale)
    });

    // SetIgnoreParentScale(ignore) - set whether frame ignores parent scale
    methods.add_method("SetIgnoreParentScale", |_, _this, _ignore: bool| Ok(()));

    // GetIgnoreParentScale() - get whether frame ignores parent scale
    methods.add_method("GetIgnoreParentScale", |_, _this, ()| Ok(false));

    // SetIgnoreParentAlpha(ignore) - set whether frame ignores parent alpha
    methods.add_method("SetIgnoreParentAlpha", |_, _this, _ignore: bool| Ok(()));

    // GetIgnoreParentAlpha() - get whether frame ignores parent alpha
    methods.add_method("GetIgnoreParentAlpha", |_, _this, ()| Ok(false));
}

/// Region/frame query methods: IsRectValid, IsObjectLoaded, IsMouseOver, etc.
fn add_region_query_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // IsRectValid() - true if the frame has at least one anchor
    methods.add_method("IsRectValid", |_, this, ()| {
        let state = this.state.borrow();
        let valid = state
            .widgets
            .get(this.id)
            .map(|f| !f.anchors.is_empty())
            .unwrap_or(false);
        Ok(valid)
    });

    // IsObjectLoaded() - always true in the simulator
    methods.add_method("IsObjectLoaded", |_, _this, ()| Ok(true));

    // IsMouseOver() - stub returning true (in the simulator, click handlers assume mouse-over)
    methods.add_method("IsMouseOver", |_, _this, _args: mlua::MultiValue| Ok(true));

    // StopAnimating() - stub
    methods.add_method("StopAnimating", |_, _this, ()| Ok(()));

    // GetSourceLocation() - no debug info in simulator
    methods.add_method("GetSourceLocation", |_, _this, ()| Ok(Value::Nil));

    // Intersects(region) - stub returning false
    methods.add_method("Intersects", |_, _this, _region: Value| Ok(false));

    // IsDrawLayerEnabled(layer) - stub returning true
    methods.add_method("IsDrawLayerEnabled", |_, _this, _layer: String| Ok(true));

    // SetDrawLayerEnabled(layer, enabled) - stub
    methods.add_method(
        "SetDrawLayerEnabled",
        |_, _this, (_layer, _enabled): (String, bool)| Ok(()),
    );
}

/// Check if a widget type is or inherits from the given type name.
/// WoW type hierarchy:
/// - Region: base of all
/// - Frame extends Region
/// - Button extends Frame
/// - CheckButton extends Button
/// - GameTooltip extends Frame
/// - EditBox, ScrollFrame, Slider, StatusBar, etc. extend Frame
/// - FontString, Texture extend Region (not Frame)
fn widget_type_is_a(wt: crate::widget::WidgetType, type_name: &str) -> bool {
    use crate::widget::WidgetType;
    // Exact match
    if wt.as_str().eq_ignore_ascii_case(type_name) {
        return true;
    }
    // Check parent types
    match type_name.to_ascii_lowercase().as_str() {
        "region" => true, // Everything is a Region
        "frame" => !matches!(wt, WidgetType::FontString | WidgetType::Texture),
        "button" => matches!(wt, WidgetType::Button | WidgetType::CheckButton),
        _ => false,
    }
}
